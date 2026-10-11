import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getState, pollState, readPending, saveSession, sendCommand } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, Region, View } from './types';
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
  it.each([{ mode: 'duel' as const, count: 3 }, { mode: 'teams' as const, count: 5 }])(
    'accepts the complete $count-slot $mode projection and confirms its command receipt', async ({ mode, count }) => {
      const regions: Region[] = Array.from({ length: count }, (_, index) => index === 1
        ? { id: `empty-region:${index}`, index, cardId: '', name: '空位', threshold: 0, points: 0,
          influence: [0, 0], characters: [], iconsByTeam: [{ investigation: 0, combat: 0, influence: 0 },
            { investigation: 0, combat: 0, influence: 0 }], skipConfrontation: true }
        : { id: `region-instance:${index}`, index, cardId: 'DQJC112', name: '纽约', threshold: 4, points: 4,
          influence: [0, 0], characters: [] });
      const view: View = { ...readyView(3), mode, status: 'playing', worldDeckCount: 0, regions,
        legalActions: [{ id: 'pass', kind: 'pass', label: '让过' }] };
      const before = { ...view, version: 2 };
      const commands: Command[] = [];
      vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
        if (url.endsWith('/catalog')) return json(testCatalog);
        if (url.endsWith('/commands')) { commands.push(JSON.parse(String(init?.body))); return json(view); }
        return json(before);
      }));
      expect(await getState(seat)).toEqual(before);
      expect(await pollState(seat, 1, new AbortController().signal)).toEqual(before);
      expect(await sendCommand(seat, 2, { kind: 'pass' }, 'slot-projection-receipt')).toEqual(view);
      const hook = renderHook(useGame); await act(async () => {});
      await act(async () => { await hook.result.current.act({ kind: 'pass' }); });
      expect(commands).toHaveLength(2);
      expect(commands[1].expectedVersion).toBe(2);
      expect(hook.result.current.view?.regions).toEqual(regions);
      expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
    });

  it.each(invalidReplies)('keeps the original pending command after a successful %s response, even when state reads succeed', async (_name, invalidReply) => {
    const commands: Command[] = [];
    let recover = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (!url.endsWith('/commands')) return json(readyView(commands.length ? 5 : 1));
      const command = JSON.parse(String(init?.body)); commands.push(command);
      return recover ? json(readyView(command.commandId === commands[0].commandId ? 2 : command.expectedVersion + 1)) : invalidReply();
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

  it.each([4, 0])('retains the original expected-v5 command after a full ACK v%i while reads reach v9', async ackVersion => {
    const commands: Command[] = [];
    let recover = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (!url.endsWith('/commands')) return json(readyView(commands.length ? 9 : 5));
      const command = JSON.parse(String(init?.body)); commands.push(command);
      return json(readyView(!recover ? ackVersion : command.commandId === commands[0].commandId ? 6 : 10));
    }));
    const hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    const original = commands[0];
    expect(hook.result.current.uncertain).toBe(true);
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...original });
    expect(original.expectedVersion).toBe(5);
    expect(commands).toHaveLength(2); expect(commands[1]).toEqual(original);
    expect(hook.result.current.view?.version).toBe(9);
    recover = true;
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(original);
    expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
    expect(hook.result.current.view?.version).toBe(9);
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(4); expect(commands[3].commandId).not.toBe(original.commandId);
    expect(commands[3].expectedVersion).toBe(9);
  });

  it.each([{ ackVersion: 5, kind: 'pauseRoom' }, { ackVersion: 6, kind: 'ready' }])(
    'confirms a valid $kind receipt v$ackVersion for expected-v5 without replacing the v9 table', async ({ ackVersion, kind }) => {
      const commands: Command[] = [];
      let release: (() => void) | undefined;
      vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
        if (url.endsWith('/catalog')) return Promise.resolve(json(testCatalog));
        const table = { ...readyView(commands.length ? 9 : 5), status: kind === 'pauseRoom' ? 'playing' as const : 'lobby' as const };
        if (!url.endsWith('/commands')) return Promise.resolve(json(table));
        commands.push(JSON.parse(String(init?.body)));
        return new Promise<Response>(resolve => { release = () => resolve(json({ ...table, version: ackVersion,
          ...(kind === 'pauseRoom' ? { pause: { pausedAtMs: 1000, pausedBy: 0 } } : {}) })); });
      }));
      const hook = renderHook(useGame); await act(async () => {});
      let request: Promise<void> | undefined;
      await act(async () => { request = hook.result.current.act({ kind }); });
      expect(commands).toHaveLength(1); expect(commands[0].expectedVersion).toBe(5);
      act(() => { hook.result.current.refresh(); });
      await act(async () => { await vi.advanceTimersByTimeAsync(0); });
      expect(hook.result.current.view?.version).toBe(9);
      await act(async () => { release!(); await request; });
      expect(hook.result.current.view?.version).toBe(9);
      expect(commands).toHaveLength(1);
      expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
    });
});
