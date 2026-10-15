import { act, cleanup, fireEvent, render, renderHook, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, readSession, saveSession } from './api';
import { GameApp } from './GameApp';
import { playerStorage } from './playerStorage';
import { testCatalog, testChoice, testView } from './testFixtures';
import type { Action, SavedSession } from './types';
import { useGame } from './useGame';

const seat: SavedSession = { roomId: testView.roomId, seat: 0, token: 'storage-feedback-token', inviteCode: testView.inviteCode };
const json = (value: unknown, status = 200) => new Response(JSON.stringify(value), { status });
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('visible browser recovery persistence failures', () => {
  it('keeps a new seat in this page through lobby return and saves it on explicit retry without creating another room', async () => {
    let blocked = true;
    const write = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
      if (blocked && key === 'hegemony.seats.v1') throw new DOMException('full', 'QuotaExceededError');
      write.call(this, key, value);
    });
    let entries = 0;
    vi.stubGlobal('fetch', vi.fn(async (url: string) => {
      if (url.endsWith('/catalog')) return json({ ...testCatalog, entryIdempotency: true });
      if (url === '/api/rooms') { entries++; return json({ ...seat, view: testView }); }
      return json(testView);
    }));
    const hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.create('甲', 'duel', 'watchers'); });
    expect(hook.result.current.view?.roomId).toBe(seat.roomId);
    expect(hook.result.current.persistenceWarning).toContain('此席位'); expect(readSession()).toBeNull();
    expect(hook.result.current.savedSeats).toEqual([seat]);
    act(() => { hook.result.current.leave(); });
    expect(hook.result.current.resumeAvailable).toBe(true); expect(hook.result.current.savedSeats).toEqual([seat]);
    await act(async () => { hook.result.current.resume(hook.result.current.savedSeats[0]); });
    blocked = false;
    act(() => { hook.result.current.retryPersistence(); });
    expect(hook.result.current.persistenceWarning).toBe(''); expect(readSession()).toEqual(seat);
    expect(entries).toBe(1);
    hook.unmount();
    const restored = renderHook(useGame); await act(async () => {});
    expect(restored.result.current.session).toEqual(seat); expect(entries).toBe(1);
  });

  it('offers a visible retry when the original command only exists in memory and persists the same identity before refresh', async () => {
    saveSession(seat);
    let blocked = true;
    let online = false;
    const commands: { commandId: string; expectedVersion: number; action: Action }[] = [];
    const write = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
      if (blocked && key.startsWith('hegemony.pending.v2.')) throw new DOMException('full', 'QuotaExceededError');
      write.call(this, key, value);
    });
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        return online ? json({ ...testView, version: 2 }) : json({ message: 'lost receipt' }, 503);
      }
      return json(testView);
    }));
    let app = render(<GameApp />); await act(async () => {});
    await act(async () => { fireEvent.click(screen.getByRole('button', { name: '准备' })); });
    expect(commands).toHaveLength(2); expect(commands[1]).toEqual(commands[0]); expect(readPending(seat)).toBeNull();
    expect(screen.getByRole('status', { name: '浏览器保存提示' })).toHaveTextContent('上一行动的确认记录尚未写入浏览器');
    blocked = false;
    fireEvent.click(screen.getByRole('button', { name: '重试保存恢复信息' }));
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: seat.seat, ...commands[0] });
    expect(screen.queryByRole('status', { name: '浏览器保存提示' })).not.toBeInTheDocument();
    expect(commands).toHaveLength(2);
    app.unmount(); online = true;
    app = render(<GameApp />); await act(async () => {});
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]); expect(readPending(seat)).toBeNull();
  });

  it('does not apply a late entry from an unmounted page into the new player namespace', async () => {
    const ordinary = playerStorage();
    const newer = { ...seat, roomId: 'new-independent-room', token: 'new-independent-token' };
    let release: ((response: Response) => void) | undefined;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url === '/api/rooms') {
        const body = JSON.parse(String(init?.body));
        if (body.name === 'old-page') return new Promise<Response>(resolve => { release = resolve; });
        return json({ ...newer, view: { ...testView, roomId: newer.roomId } });
      }
      return json({ ...testView, roomId: newer.roomId });
    }));
    const old = renderHook(useGame); await act(async () => {});
    let request: Promise<void> | undefined;
    act(() => { request = old.result.current.create('old-page', 'duel', 'watchers'); });
    old.unmount();
    const current = renderHook(useGame); await act(async () => {});
    await act(async () => { current.result.current.startIndependentSession(); });
    const independent = playerStorage();
    await act(async () => { await current.result.current.create('new-page', 'duel', 'watchers'); });
    await act(async () => { release!(json({ ...seat, view: testView })); await request; });
    expect(readSession(ordinary)).toEqual(seat);
    expect(readSession(independent)).toEqual(newer);
    expect(current.result.current.session).toEqual(newer);
    expect(current.result.current.view?.roomId).toBe(newer.roomId);
  });

  it('keeps recovery notices and their retry controls reachable inside a pending choice dialog', async () => {
    saveSession(seat);
    const choosing = { ...testView, status: 'playing', pendingChoice: testChoice,
      legalActions: [{ id: 'choose', kind: 'choose', choiceId: testChoice.id, label: '确认选择' }] };
    let blocked = true;
    let online = false;
    const write = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
      if (blocked && key.startsWith('hegemony.pending.v2.')) throw new DOMException('full', 'QuotaExceededError');
      write.call(this, key, value);
    });
    const commands: unknown[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        return online ? json({ ...choosing, version: 2, pendingChoice: null, legalActions: [] }) : json({ message: 'lost receipt' }, 503);
      }
      return json(choosing);
    }));
    render(<GameApp />); await act(async () => {});
    fireEvent.click(screen.getByRole('button', { name: /角色乙/ }));
    await act(async () => { fireEvent.click(screen.getByRole('button', { name: '确认选择' })); });
    const dialog = within(screen.getByRole('dialog', { name: '待完成的选择' }));
    expect(dialog.getByRole('status', { name: '浏览器保存提示' })).toHaveTextContent('上一行动的确认记录尚未写入浏览器');
    blocked = false;
    fireEvent.click(dialog.getByRole('button', { name: '重试保存恢复信息' }));
    online = true;
    await act(async () => { fireEvent.click(dialog.getByRole('button', { name: '确认上一行动' })); });
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]);
    expect(screen.queryByRole('dialog', { name: '待完成的选择' })).not.toBeInTheDocument();
  });
});
