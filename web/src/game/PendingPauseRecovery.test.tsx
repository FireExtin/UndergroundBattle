import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, savePending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, View } from './types';
import { useGame } from './useGame';

const seat = { roomId: testView.roomId, seat: 0, token: 'pause-recovery-test', inviteCode: 'INVITE' };
const playing: View = { ...testView, status: 'playing', canPause: true, serverNowMs: 1000 };
const paused: View = { ...playing, version: playing.version + 1, pause: { pausedAtMs: 2000, pausedBy: 0 } };
const json = (value: unknown) => new Response(JSON.stringify(value));
type Command = { commandId: string; expectedVersion: number; action: Action };
beforeEach(() => {
  localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); saveSession(seat);
  vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible');
});
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('paused table with an unconfirmed command', () => {
  it('automatically confirms a committed pause after both acknowledgements are lost, then becomes idle', async () => {
    let state = playing;
    let recover = false;
    let commits = 0;
    const commands: Command[] = [];
    const reads: string[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        if (commands.length === 1) { state = paused; commits++; }
        if (!recover) throw new Error('pause acknowledgement unavailable');
        return json(paused);
      }
      reads.push(url);
      return url.includes('?') ? new Response(null, { status: 204 }) : json(state);
    }));
    const hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'pauseRoom' }); });
    expect(hook.result.current.view?.pause).toBeDefined();
    expect(hook.result.current.uncertain).toBe(true); expect(commands).toHaveLength(2);
    const original = commands[0];
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...original });
    recover = true;
    await act(async () => { await vi.advanceTimersByTimeAsync(1500); });
    expect(commands).toEqual([original, original, original]); expect(commits).toBe(1);
    expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
    const settledReads = reads.length;
    await act(async () => { await vi.advanceTimersByTimeAsync(60000); });
    expect(reads).toHaveLength(settledReads); expect(commands).toHaveLength(3);
  });

  it('continues receipt recovery on a reloaded paused table and foregrounds it immediately without resuming the game', async () => {
    const original = { roomId: seat.roomId, seat: 0, commandId: 'persisted-pause', expectedVersion: playing.version, action: { kind: 'pauseRoom' } };
    savePending(original);
    let visible: DocumentVisibilityState = 'hidden';
    vi.spyOn(document, 'visibilityState', 'get').mockImplementation(() => visible);
    let recover = false;
    const commands: Command[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        if (!recover) throw new Error('receipt unavailable');
      }
      return json(paused);
    }));
    const hook = renderHook(useGame); await act(async () => {});
    expect(hook.result.current.uncertain).toBe(true); expect(commands).toHaveLength(2);
    recover = true; visible = 'visible';
    act(() => { document.dispatchEvent(new Event('visibilitychange')); });
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(commands).toHaveLength(3);
    expect(commands.every(command => JSON.stringify(command) === JSON.stringify(commands[0]))).toBe(true);
    expect(commands[0]).toMatchObject({ commandId: original.commandId, expectedVersion: original.expectedVersion, action: { kind: 'pauseRoom' } });
    expect(hook.result.current.view?.pause).toBeDefined(); expect(hook.result.current.uncertain).toBe(false);
  });
});
