import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getState, pollState, readPending, saveSession, sendCommand } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, View } from './types';
import { useGame } from './useGame';

const seat = { roomId: testView.roomId, seat: 0, token: 'ack-validation-seat', inviteCode: 'INVITE' };
const json = (value: unknown) => new Response(JSON.stringify(value), { status: 200 });
const readyView = (version: number): View => ({ ...testView, version, serverNowMs: 1000,
  players: [{ ...testView.players[0], ready: version > 1 }] });
const invalidReplies: [string, () => Response][] = [
  ['empty JSON', () => json({})],
  ['null JSON', () => json(null)],
  ['HTML', () => new Response('<html>Sign in</html>', { status: 200 })],
  ['another room', () => json({ ...readyView(2), roomId: 'another-room' })],
  ['another seat', () => json({ ...readyView(2), you: 'p1' })],
  ['negative version', () => json({ ...readyView(2), version: -1 })],
  ['fractional version', () => json({ ...readyView(2), version: 1.5 })],
  ['headers without a table', () => json({ roomId: seat.roomId, you: 'p0', version: 2 })],
  ['missing required collection', () => json({ ...readyView(2), players: undefined })],
  ['null player', () => json({ ...readyView(2), players: [null] })],
  ['null region', () => json({ ...readyView(2), regions: [null] })],
  ['null legal action', () => json({ ...readyView(2), legalActions: [null] })],
  ['missing response members', () => json({ ...readyView(2), responseWindow: { members: null } })],
  ['missing region characters', () => json({ ...readyView(2), regions: [{ ...testView.regions[0],
    id: 'region-a', index: 0, cardId: 'region-card', name: '地区', threshold: 3, points: 1,
    influence: [0, 0], characters: null }] })],
];
type Command = { commandId: string; expectedVersion: number; action: Action };

beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); saveSession(seat); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('command acknowledgement validation', () => {
  it.each(invalidReplies)('keeps the original pending command after a successful %s response, even when state reads succeed', async (_name, invalidReply) => {
    const commands: Command[] = [];
    let recover = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (!url.endsWith('/commands')) return json(readyView(commands.length ? 5 : 1));
      commands.push(JSON.parse(String(init?.body)));
      return recover ? json(readyView(2)) : invalidReply();
    }));
    let hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    const original = commands[0];
    expect(hook.result.current.uncertain).toBe(true);
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...original });
    expect(commands).toHaveLength(2);
    expect(commands[1]).toEqual(original);
    expect(hook.result.current.view?.version).toBe(5);
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(4);
    act(() => { hook.result.current.refresh(); });
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(commands).toHaveLength(6);
    hook.unmount();
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...original });
    recover = true; hook = renderHook(useGame); await act(async () => {});
    expect(commands).toHaveLength(7);
    expect(commands.every(command => JSON.stringify(command) === JSON.stringify(original))).toBe(true);
    // An authentic older receipt confirms the command without replacing the newer table.
    expect(hook.result.current.view?.version).toBe(5);
    expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(8);
    expect(commands[7].commandId).not.toBe(original.commandId);
    expect(commands[7].expectedVersion).toBe(5);
  });

  it.each(invalidReplies)('rejects %s at each successful table API boundary', async (_name, invalidReply) => {
    vi.stubGlobal('fetch', vi.fn(async () => invalidReply()));
    await expect(sendCommand(seat, 1, { kind: 'ready' }, 'original-command')).rejects.toMatchObject({ status: 0 });
    await expect(getState(seat)).rejects.toMatchObject({ status: 0 });
    await expect(pollState(seat, 1, new AbortController().signal)).rejects.toMatchObject({ status: 0 });
  });

  it('keeps a definite 409 rejection definite while discarding its malformed view and reading the table', async () => {
    const commands: Command[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (!url.endsWith('/commands')) return json(readyView(commands.length ? 5 : 1));
      commands.push(JSON.parse(String(init?.body)));
      return new Response(JSON.stringify({ error: 'version_conflict', message: '版本冲突',
        view: { ...readyView(9), players: [null] } }), { status: 409 });
    }));
    const hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(1);
    expect(hook.result.current.view?.version).toBe(5);
    expect(hook.result.current.view?.players[0].ready).toBe(true);
    expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
  });
});
