import { act, cleanup, fireEvent, render, renderHook, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { saveSession } from './api';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, DECK_DRAFT_STORAGE_PREFIX, readDeckLibrary, removeDeckDraft, saveDeckDraft, saveDeckLibrary, type DeckStorage } from './deckLibrary';
import { testCatalog, testView } from './testFixtures';
import type { SavedSession, View } from './types';
import { useGame } from './useGame';

// Two independent instances share the real jsdom localStorage. Fetch is a
// deterministic transport double: no Worker, public room or game state is used.
const firstSeat: SavedSession = { roomId: 'shared-first', inviteCode: 'FIRST', seat: 0, token: 'first-token' };
const secondSeat: SavedSession = { roomId: 'shared-second', inviteCode: 'SECOND', seat: 0, token: 'second-token' };
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const roomView = (seat: SavedSession): View => ({ ...testView, roomId: seat.roomId, you: `p${seat.seat}` });
type SentCommand = { commandId: string; expectedVersion: number; action: { kind: string; option?: string } };
const commandBody = (init?: RequestInit): SentCommand => JSON.parse(String(init?.body));
async function mountSeat(seat: SavedSession) {
  saveSession(seat);
  const hook = renderHook(useGame);
  await act(async () => {});
  expect(hook.result.current.view?.roomId).toBe(seat.roomId);
  return hook;
}
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); });
afterEach(() => {
  cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals();
  localStorage.clear(); sessionStorage.clear();
});

describe('shared pending recovery across separately mounted hooks', () => {
  it('keeps the original same-seat receipt recoverable after a stale peer submits and confirms another action', async () => {
    const commands: SentCommand[] = [];
    let acknowledgeReady = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) {
        const command = commandBody(init); commands.push(command);
        if (command.action.kind === 'ready' && !acknowledgeReady) return json({ message: 'ACK unavailable' }, 503);
        return json({ ...roomView(firstSeat), version: 2 });
      }
      return url.endsWith('/catalog') ? json(testCatalog) : json(roomView(firstSeat));
    }));
    const first = await mountSeat(firstSeat);
    const stalePeer = await mountSeat(firstSeat); // Mounted before first creates pending.
    await act(async () => { await first.result.current.act({ kind: 'ready' }); });
    const original = commands[0];
    expect(commands).toEqual([original, original]); // Existing lost-ACK retry identity.
    expect(first.result.current.uncertain).toBe(true);
    await act(async () => { await stalePeer.result.current.act({ kind: 'deck', option: 'watchers' }); });
    const beforeReload = commands.filter(command => command.commandId === original.commandId).length;
    first.unmount(); stalePeer.unmount();
    acknowledgeReady = true;
    await mountSeat(firstSeat);
    expect(commands.filter(command => command.commandId === original.commandId)).toHaveLength(beforeReload + 1);
    expect(commands.filter(command => command.commandId === original.commandId).every(command => JSON.stringify(command) === JSON.stringify(original))).toBe(true);
    expect(commands.at(-1)).toEqual(original);
  });

  it('preserves both original receipts for two rooms and restores each with its original body', async () => {
    const commands: { room: string; body: SentCommand }[] = [];
    let acknowledge = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      const seat = url.includes(secondSeat.roomId) ? secondSeat : firstSeat;
      if (url.endsWith('/commands')) {
        commands.push({ room: seat.roomId, body: commandBody(init) });
        return acknowledge ? json({ ...roomView(seat), version: 2 }) : json({ message: 'ACK unavailable' }, 503);
      }
      return url.endsWith('/catalog') ? json(testCatalog) : json(roomView(seat));
    }));
    const first = await mountSeat(firstSeat);
    const second = await mountSeat(secondSeat);
    await act(async () => { await first.result.current.act({ kind: 'ready' }); });
    await act(async () => { await second.result.current.act({ kind: 'ready' }); });
    const originalFirst = commands.find(command => command.room === firstSeat.roomId)!;
    const originalSecond = commands.find(command => command.room === secondSeat.roomId)!;
    expect(first.result.current.uncertain).toBe(true); expect(second.result.current.uncertain).toBe(true);
    first.unmount(); second.unmount(); acknowledge = true;
    const restoredFirst = await mountSeat(firstSeat);
    expect(commands.filter(command => command.room === firstSeat.roomId)).toEqual([originalFirst, originalFirst, originalFirst]);
    restoredFirst.unmount();
    const restoredSecond = await mountSeat(secondSeat);
    expect(commands.filter(command => command.room === secondSeat.roomId)).toEqual([originalSecond, originalSecond, originalSecond]);
    expect(restoredSecond.result.current.uncertain).toBe(false);
  });

  it.each([200, 409])('an older instance finishing with HTTP %s cannot clear another room’s unconfirmed receipt', async status => {
    const commands: { room: string; body: SentCommand }[] = [];
    let finishFirst: ((response: Response) => void) | undefined;
    let acknowledgeSecond = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      const seat = url.includes(secondSeat.roomId) ? secondSeat : firstSeat;
      if (url.endsWith('/commands')) {
        commands.push({ room: seat.roomId, body: commandBody(init) });
        if (seat === firstSeat) return new Promise<Response>(resolve => { finishFirst = resolve; });
        return acknowledgeSecond ? json({ ...roomView(seat), version: 2 }) : json({ message: 'ACK unavailable' }, 503);
      }
      return url.endsWith('/catalog') ? json(testCatalog) : json(roomView(seat));
    }));
    const first = await mountSeat(firstSeat);
    const second = await mountSeat(secondSeat);
    let firstRequest: Promise<void> | undefined;
    await act(async () => { firstRequest = first.result.current.act({ kind: 'ready' }); });
    expect(finishFirst).toBeTypeOf('function');
    await act(async () => { await second.result.current.act({ kind: 'ready' }); });
    const originalSecond = commands.find(command => command.room === secondSeat.roomId)!;
    expect(second.result.current.uncertain).toBe(true);
    await act(async () => {
      finishFirst!(status === 200 ? json({ ...roomView(firstSeat), version: 2 })
        : json({ error: 'version_conflict', view: { ...roomView(firstSeat), version: 2 } }, 409));
      await firstRequest;
    });
    first.unmount(); second.unmount(); acknowledgeSecond = true;
    await mountSeat(secondSeat);
    expect(commands.filter(command => command.room === secondSeat.roomId)).toEqual([originalSecond, originalSecond, originalSecond]);
  });
});

describe('shared draft storage across separately mounted editors', () => {
  const catalog = { ...testCatalog, deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false } };
  const editor = () => render(<DeckLibrary catalog={catalog} storage={localStorage} />);
  const saveNamed = (instance: ReturnType<typeof editor>, name: string) => {
    const page = within(instance.container);
    fireEvent.change(page.getByRole('textbox', { name: '牌组名称' }), { target: { value: name } });
    fireEvent.click(page.getByRole('button', { name: '保存牌组' }));
  };
  it('retains separately added drafts when both editors mounted before either save', () => {
    const first = editor(); const second = editor();
    saveNamed(first, 'draft-a'); saveNamed(second, 'draft-b');
    expect(readDeckLibrary().drafts.map(draft => draft.name).sort()).toEqual(['draft-a', 'draft-b']);
  });
  it('does not revive a deleted draft when a stale editor saves an unrelated new draft', () => {
    const original = { ...createDeckDraft(catalog), name: 'delete-me' };
    expect(saveDeckLibrary([original])).toBeNull();
    const first = editor(); const stalePeer = editor();
    fireEvent.click(within(first.container).getByRole('button', { name: '删除牌组 delete-me' }));
    fireEvent.click(within(stalePeer.container).getByRole('button', { name: '新建空白牌组' }));
    saveNamed(stalePeer, 'keep-me');
    expect(readDeckLibrary().drafts.map(draft => draft.name)).toEqual(['keep-me']);
  });
  it('deletes only the requested draft while preserving another editor’s newly saved draft', () => {
    const original = { ...createDeckDraft(catalog), name: 'delete-me' };
    expect(saveDeckLibrary([original])).toBeNull();
    const stalePeer = editor(); const second = editor();
    fireEvent.click(within(second.container).getByRole('button', { name: '新建空白牌组' }));
    saveNamed(second, 'keep-me');
    fireEvent.click(within(stalePeer.container).getByRole('button', { name: '删除牌组 delete-me' }));
    expect(readDeckLibrary().drafts.map(draft => draft.name)).toEqual(['keep-me']);
  });
  it('keeps a deleted draft out of a stale editor save and offers an explicit new identity for the retained edits', () => {
    const original = { ...createDeckDraft(catalog), name: 'delete-me', cards: [{ cardId: 'JC125', count: 50 }] };
    expect(saveDeckLibrary([original])).toBeNull();
    const first = editor(); const stalePeer = editor();
    fireEvent.click(within(first.container).getByRole('button', { name: '删除牌组 delete-me' }));
    saveNamed(stalePeer, 'retained-edits');
    expect(within(stalePeer.container).getByRole('alert')).toHaveTextContent('已在另一页删除');
    expect(readDeckLibrary().drafts).toEqual([]);
    fireEvent.click(within(stalePeer.container).getByRole('button', { name: '另存为新草稿' }));
    const copy = readDeckLibrary().drafts[0];
    expect(copy.id).not.toBe(original.id); expect(copy.name).toBe('retained-edits'); expect(copy.cards).toEqual(original.cards);
  });
  it('retains both identities when another save commits between the first save’s check and write', () => {
    const first = { ...createDeckDraft(catalog), name: 'concurrent-a' };
    const second = { ...createDeckDraft(catalog), name: 'concurrent-b' };
    let interleaved = false;
    const storage: DeckStorage = {
      get length() { return localStorage.length; }, key: index => localStorage.key(index), getItem: key => localStorage.getItem(key),
      setItem: (key, value) => {
        if (!interleaved && key.startsWith(DECK_DRAFT_STORAGE_PREFIX)) { interleaved = true; expect(saveDeckDraft(second)).toBeNull(); }
        localStorage.setItem(key, value);
      },
    };
    expect(saveDeckDraft(first, storage)).toBeNull();
    expect(readDeckLibrary().drafts.map(draft => draft.name).sort()).toEqual(['concurrent-a', 'concurrent-b']);
  });
  it('a deletion arriving between a stale save’s check and write cannot be overwritten by the late save', () => {
    const original = { ...createDeckDraft(catalog), name: 'delete-during-save' };
    expect(saveDeckLibrary([original])).toBeNull();
    const storage: DeckStorage = {
      get length() { return localStorage.length; }, key: index => localStorage.key(index), getItem: key => localStorage.getItem(key),
      setItem: (key, value) => { expect(removeDeckDraft(original.id)).toBeNull(); localStorage.setItem(key, value); },
    };
    expect(saveDeckDraft({ ...original, name: 'late-edit' }, storage)).toContain('已在另一页删除');
    expect(readDeckLibrary().drafts).toEqual([]);
  });
});
