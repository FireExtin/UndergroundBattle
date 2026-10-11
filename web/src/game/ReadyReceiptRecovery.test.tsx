import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, View } from './types';
import { useGame } from './useGame';

const seat = { roomId: testView.roomId, seat: 0, token: 'ready-receipt-test', inviteCode: 'INVITE' };
const json = (value: unknown, status = 200) => new Response(JSON.stringify(value), { status });
type Command = { commandId: string; expectedVersion: number; action: Action };
const readyView = (version: number, ready: boolean): View => ({ ...testView, version, serverNowMs: 1000,
  players: [{ ...testView.players[0], ready }],
  legalActions: [{ id: 'ready', kind: 'ready', label: ready ? '取消准备' : '准备' }] });
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); saveSession(seat); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('ready toggle receipt and disconnect contracts', () => {
  it('retries a committed ready toggle with the exact original tuple and keeps a newer poll over its old receipt', async () => {
    const commands: Command[] = [];
    let toggles = 0;
    let server = readyView(1, false);
    let receipt: View | undefined;
    let releaseReceipt: (() => void) | undefined;
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible');
    vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return Promise.resolve(json(testCatalog));
      if (!url.endsWith('/commands')) return Promise.resolve(json(server));
      commands.push(JSON.parse(String(init?.body)));
      if (commands.length === 1) {
        toggles++; receipt = readyView(2, true); server = receipt;
        return Promise.reject(new Error('committed response lost'));
      }
      if (commands.length === 2) return new Promise<Response>(resolve => { releaseReceipt = () => resolve(json(receipt)); });
      toggles++; server = readyView(6, false); return Promise.resolve(json(server));
    }));
    const hook = renderHook(useGame); await act(async () => {});
    let request: Promise<void> | undefined;
    await act(async () => { request = hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(2);
    expect(commands[1]).toEqual(commands[0]);
    expect(commands[0]).toMatchObject({ expectedVersion: 1, action: { kind: 'game', action: { kind: 'ready' } } });
    expect(toggles).toBe(1);
    server = readyView(5, true); // A peer revision is newer than the saved acknowledgement.
    act(() => { document.dispatchEvent(new Event('visibilitychange')); });
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(hook.result.current.view?.version).toBe(5);
    await act(async () => { releaseReceipt!(); await request; });
    expect(hook.result.current.view?.version).toBe(5);
    expect(hook.result.current.view?.players[0].ready).toBe(true);
    expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
    expect(commands).toHaveLength(2); expect(toggles).toBe(1);
    // Only a subsequent explicit click may create a fresh ID and toggle back.
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(3); expect(commands[2].commandId).not.toBe(commands[0].commandId);
    expect(commands[2].expectedVersion).toBe(5); expect(toggles).toBe(2);
    expect(hook.result.current.view?.players[0].ready).toBe(false);
  });

  it('keeps an unknown ready result through refresh, further clicks and remount without a new identity or implicit pause', async () => {
    const commands: Command[] = [];
    let recover = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        if (!recover) throw new Error('offline after possible commit');
        return json(readyView(2, true));
      }
      return url.endsWith('/catalog') ? json(testCatalog) : json(readyView(commands.length ? 4 : 1, commands.length > 0));
    }));
    let hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    const original = commands[0];
    expect(commands).toHaveLength(2); expect(hook.result.current.uncertain).toBe(true);
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...original });
    await act(async () => { await hook.result.current.act({ kind: 'ready' }); });
    const beforeRefresh = commands.length;
    act(() => { hook.result.current.refresh(); });
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(commands).toHaveLength(beforeRefresh + 2); // Both lost-ACK attempts keep the original tuple.
    expect(commands.every(command => JSON.stringify(command) === JSON.stringify(original))).toBe(true);
    const beforeUnmount = commands.length;
    hook.unmount(); // Closing this UI has no protocol action.
    expect(commands).toHaveLength(beforeUnmount);
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...original });
    recover = true; hook = renderHook(useGame); await act(async () => {});
    expect(commands).toHaveLength(beforeUnmount + 1);
    expect(commands.at(-1)).toEqual(original);
    expect(commands.every(command => command.action.kind === 'game' && command.action.action?.kind === 'ready')).toBe(true);
    expect(hook.result.current.view?.version).toBe(4); // Original receipt v2 cannot undo current v4.
    expect(hook.result.current.uncertain).toBe(false); expect(readPending(seat)).toBeNull();
  });

  it('does not pause a running game when reads lose connection, the tab becomes hidden or the UI unmounts', async () => {
    const commands: Command[] = [];
    let offline = false;
    let visibility: DocumentVisibilityState = 'visible';
    vi.spyOn(document, 'visibilityState', 'get').mockImplementation(() => visibility);
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) { commands.push(JSON.parse(String(init?.body))); return json(testView); }
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (offline) throw new Error('disconnected');
      return json({ ...testView, status: 'playing', serverNowMs: 1000, canPause: true, legalActions: [] });
    }));
    const hook = renderHook(useGame); await act(async () => {});
    offline = true;
    await act(async () => { await vi.advanceTimersByTimeAsync(1500); });
    expect(hook.result.current.connection).toBe('offline');
    expect(hook.result.current.view?.pause).toBeUndefined();
    visibility = 'hidden';
    act(() => { document.dispatchEvent(new Event('visibilitychange')); });
    hook.unmount();
    expect(commands).toEqual([]); expect(readPending(seat)).toBeNull();
  });
});
