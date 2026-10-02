import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { readSavedSeats, savePending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import { newerView, useGame } from './useGame';

const session = { roomId: testView.roomId, inviteCode: 'INVITE', token: 'opaque-token', seat: 0 };
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
afterEach(() => { vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); });
describe('room lifecycle and reconciliation', () => {
  it('never regresses a table when a delayed command returns behind a newer poll', () => {
    expect(newerView({ ...testView, version: 12 }, { ...testView, version: 9 }).version).toBe(12);
    expect(newerView(testView, { ...testView, version: 2 }).version).toBe(2);
    expect(newerView({ ...testView, version: 12 }, { ...testView, you: 'p1', version: 9 }).you).toBe('p1');
  });
  it('creates a room with a curated deck and preserves only this seat for reload', async () => {
    const fetchMock = vi.fn(async (url: string) => url === '/api/catalog' ? json(testCatalog) : url === '/api/rooms' ? json({ ...session, view: testView }) : url.endsWith('/events') ? new Response(null) : json(testView));
    vi.stubGlobal('fetch', fetchMock);
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.catalog).toEqual(testCatalog));
    await act(async () => { await result.current.create('甲', 'teams', 'watchers'); });
    const call = fetchMock.mock.calls.find(([url]) => url === '/api/rooms');
    expect(call).toBeDefined(); expect(result.current.session).toEqual(session);
    expect(JSON.parse(localStorage.getItem('hegemony.session.v1')!)).toEqual(session);
    act(() => result.current.leave());
    expect(result.current.session).toBeNull(); expect(result.current.resumeAvailable).toBe(true);
    expect(JSON.parse(localStorage.getItem('hegemony.session.v1')!)).toEqual(session);
    act(() => result.current.resume());
    expect(result.current.session).toEqual(session);
  });
  it('restores persisted chooser state and reconciles a visible 409 instead of hiding it', async () => {
    saveSession(session);
    const choice = { id: 'restore-choice', kind: 'mulligan', title: '再调度', description: '选择手牌', playerId: 'p0', options: [], allowDecline: true };
    const state = { ...testView, status: 'playing', pendingChoice: choice };
    vi.stubGlobal('fetch', vi.fn(async (url: string) => url === '/api/catalog' ? json(testCatalog) : url.endsWith('/events') ? new Response(null) : url.endsWith('/commands') ? json({ error: 'stale', view: { ...state, version: 4 } }, 409) : json(state)));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.view?.pendingChoice?.id).toBe('restore-choice'));
    await act(async () => { await result.current.act({ kind: 'pass' }); });
    expect(result.current.view?.version).toBe(4);
    expect(result.current.error).toContain('已同步最新状态'); expect(result.current.uncertain).toBe(false);
  });
  it('keeps an explicit return to lobby after reload and preserves the old seat when creating another room', async () => {
    saveSession(session);
    const next = { roomId: 'new-duel-room', inviteCode: 'DUEL', token: 'new-opaque-token', seat: 0 };
    const oldView = { ...testView, mode: 'teams' };
    const nextView = { ...testView, roomId: next.roomId, inviteCode: next.inviteCode, mode: 'duel' };
    const requests: string[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string) => {
      requests.push(url);
      return url === '/api/catalog' ? json(testCatalog) : url === '/api/rooms' ? json({ ...next, view: nextView })
        : json(url.includes(next.roomId) ? nextView : oldView);
    }));
    const first = renderHook(useGame);
    await waitFor(() => expect(first.result.current.view?.mode).toBe('teams'));
    act(() => first.result.current.leave());
    expect(first.result.current.session).toBeNull(); first.unmount();
    const reloaded = renderHook(useGame);
    await waitFor(() => expect(reloaded.result.current.catalog).toEqual(testCatalog));
    expect(reloaded.result.current.session).toBeNull();
    expect(reloaded.result.current.resumeAvailable).toBe(true);
    await act(async () => { await reloaded.result.current.create('甲', 'duel', 'watchers'); });
    expect(reloaded.result.current.view?.mode).toBe('duel');
    expect(readSavedSeats()).toEqual([session, next]);
    act(() => reloaded.result.current.leave());
    act(() => reloaded.result.current.resume(session));
    await waitFor(() => expect(reloaded.result.current.view?.mode).toBe('teams'));
    expect(reloaded.result.current.session).toEqual(session);
    expect(requests.filter(url => url === '/api/rooms')).toHaveLength(1);
  });
  it('does not let an old in-flight state read pull a player back after returning to the lobby', async () => {
    saveSession(session); let release: ((response: Response) => void) | undefined;
    vi.stubGlobal('fetch', vi.fn((url: string) => url === '/api/catalog' ? Promise.resolve(json(testCatalog))
      : new Promise<Response>(resolve => { release = resolve; })));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(release).toBeDefined());
    act(() => result.current.leave());
    await act(async () => { release!(json(testView)); });
    expect(result.current.session).toBeNull(); expect(result.current.view).toBeNull();
    expect(result.current.savedSeats).toEqual([session]);
  });
  it('keeps an uncertain command for its old room and refuses to discard it when opening another table', async () => {
    saveSession(session);
    vi.stubGlobal('fetch', vi.fn(async (url: string) => {
      if (url.endsWith('/commands')) throw new Error('lost acknowledgement');
      return url === '/api/catalog' ? json(testCatalog) : json(testView);
    }));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.view).not.toBeNull());
    await act(async () => { await result.current.act({ kind: 'pass' }); });
    const original = localStorage.getItem('hegemony.pending.v1');
    act(() => result.current.leave());
    await act(async () => { await result.current.create('新桌', 'duel', 'watchers'); });
    expect(result.current.session).toBeNull();
    expect(result.current.error).toContain('旧牌桌还有待确认行动');
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(original);
    expect(result.current.savedSeats).toEqual([session]);
  });
  it('retries a lost acknowledgement with the same identity and blocks unresolved new commands', async () => {
    saveSession(session); const commands: Record<string, unknown>[] = []; let fail = true;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) { commands.push(JSON.parse(String(init?.body))); if (fail) throw new Error('connection lost'); return json({ ...testView, version: 2 }); }
      return url === '/api/catalog' ? json(testCatalog) : url.endsWith('/events') ? new Response(null) : json(testView);
    }));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.view).not.toBeNull());
    await act(async () => { await result.current.act({ kind: 'pass' }); });
    expect(commands).toHaveLength(2); expect(commands[0]).toEqual(commands[1]);
    expect(result.current.uncertain).toBe(true); expect(result.current.error).toContain('结果暂未确认');
    fail = false;
    await act(async () => { await result.current.act({ kind: 'deploy', cardId: 'never-send-new' }); });
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]);
    expect(result.current.uncertain).toBe(false); expect(localStorage.getItem('hegemony.pending.v1')).toBeNull();
  });
  it('keeps an uncertain command bound to its original seat when the same browser has two seats in one room', async () => {
    const other = { ...session, seat: 1, token: 'other-seat-token' };
    saveSession(other); saveSession(session);
    const commands: { actor: string | undefined; body: unknown }[] = []; let fail = true;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      const actor = (init?.headers as Record<string, string> | undefined)?.Authorization;
      if (url.endsWith('/commands')) {
        commands.push({ actor, body: JSON.parse(String(init?.body)) });
        if (fail) throw new Error('lost acknowledgement');
        return json({ ...testView, version: 2 });
      }
      return url === '/api/catalog' ? json(testCatalog) : json({ ...testView, you: actor === 'Bearer other-seat-token' ? 'p1' : 'p0' });
    }));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.view).not.toBeNull());
    await act(async () => { await result.current.act({ kind: 'pass' }); });
    const original = localStorage.getItem('hegemony.pending.v1');
    expect(JSON.parse(original!).seat).toBe(0);
    act(() => result.current.leave());
    act(() => result.current.resume(other));
    expect(result.current.session).toBeNull(); expect(result.current.error).toContain('原席位');
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(original); expect(commands).toHaveLength(2);
    fail = false;
    act(() => result.current.resume(session));
    await waitFor(() => expect(result.current.uncertain).toBe(false));
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]);
    expect(commands.every(command => command.actor === 'Bearer opaque-token')).toBe(true);
    expect(result.current.view?.you).toBe('p0'); expect(localStorage.getItem('hegemony.pending.v1')).toBeNull();
  });
  it('confirms an interrupted persisted command after reload without regressing the current state', async () => {
    saveSession(session); const command = { roomId: session.roomId, commandId: 'persisted-original', expectedVersion: 1, action: { kind: 'pass' } };
    savePending(command); const commands: Record<string, unknown>[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) { commands.push(JSON.parse(String(init?.body))); return json({ ...testView, version: 2 }); }
      return url === '/api/catalog' ? json(testCatalog) : url.endsWith('/events') ? new Response(null) : json({ ...testView, version: 9 });
    }));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(commands).toHaveLength(1));
    await waitFor(() => expect(result.current.uncertain).toBe(false));
    expect(commands[0]).toMatchObject({ commandId: 'persisted-original', expectedVersion: 1, action: { kind: 'pass' } });
    expect(result.current.view?.version).toBe(9);
  });
  it('polls the latest version, avoids overlapping requests and resumes when visible', async () => {
    vi.useFakeTimers(); saveSession(session);
    const polls: string[] = []; let release: ((response: Response) => void) | undefined;
    vi.stubGlobal('fetch', vi.fn(async (url: string) => {
      if (url.includes('?afterVersion=')) {
        polls.push(url);
        if (polls.length === 1) return new Promise<Response>(resolve => { release = resolve; });
        return new Response(null, { status: 204 });
      }
      return url === '/api/catalog' ? json(testCatalog) : json({ ...testView, version: 9 });
    }));
    let result: ReturnType<typeof renderHook<ReturnType<typeof useGame>, unknown>>['result'];
    await act(async () => { ({ result } = renderHook(useGame)); await vi.advanceTimersByTimeAsync(0); });
    expect(result!.current.connection).toBe('online');
    await act(async () => { await vi.advanceTimersByTimeAsync(1500); });
    expect(polls).toEqual([`/api/rooms/${session.roomId}/state?afterVersion=9`]);
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible');
    await act(async () => { document.dispatchEvent(new Event('visibilitychange')); await vi.advanceTimersByTimeAsync(0); });
    expect(polls).toHaveLength(1);
    await act(async () => { release!(json({ ...testView, version: 8 })); await vi.advanceTimersByTimeAsync(0); });
    expect(result!.current.view?.version).toBe(9);
    await act(async () => { document.dispatchEvent(new Event('visibilitychange')); await vi.advanceTimersByTimeAsync(0); });
    expect(polls).toHaveLength(2); expect(polls[1]).toContain('afterVersion=9');
    expect(result!.current.connection).toBe('online');
  });
  it('preserves the occupied seat and pending command when the private Site requires login', async () => {
    saveSession(session);
    savePending({ roomId: session.roomId, commandId: 'preserve-after-login', expectedVersion: 1, action: { kind: 'pass' } });
    vi.stubGlobal('fetch', vi.fn(async (url: string) => url === '/api/catalog' ? json(testCatalog) : json({ message: 'Sign in required' }, 403)));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.connection).toBe('offline'));
    expect(result.current.session).toEqual(session);
    expect(JSON.parse(localStorage.getItem('hegemony.session.v1')!)).toEqual(session);
    expect(JSON.parse(localStorage.getItem('hegemony.pending.v1')!).commandId).toBe('preserve-after-login');
    expect(result.current.error).toContain('需要重新登录');
  });
  it('clears a seat only when the game service explicitly confirms an invalid seat token', async () => {
    saveSession(session);
    vi.stubGlobal('fetch', vi.fn(async (url: string) => url === '/api/catalog' ? json(testCatalog) : json({ error: 'invalid_seat_token', message: 'invalid' }, 401)));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.session).toBeNull());
    expect(localStorage.getItem('hegemony.session.v1')).toBeNull();
    expect(result.current.error).toContain('无法恢复');
  });
  it('keeps the original command after a lost ACK followed by a login gate, then reconciles its receipt', async () => {
    saveSession(session); const commands: Record<string, unknown>[] = []; let blocked = true;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        if (commands.length === 1) throw new Error('lost committed ACK');
        return blocked ? json({ message: 'Sign in required' }, 403) : json({ ...testView, version: 2 });
      }
      return url === '/api/catalog' ? json(testCatalog) : json({ ...testView, version: 3 });
    }));
    const { result } = renderHook(useGame);
    await waitFor(() => expect(result.current.view?.version).toBe(3));
    await act(async () => { await result.current.act({ kind: 'pass' }); });
    expect(commands).toHaveLength(2); expect(commands[0]).toEqual(commands[1]);
    expect(result.current.uncertain).toBe(true);
    expect(localStorage.getItem('hegemony.pending.v1')).not.toBeNull();
    blocked = false;
    await act(async () => { await result.current.act({ kind: 'deploy', cardId: 'do-not-send-new' }); });
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]);
    expect(result.current.view?.version).toBe(3); expect(result.current.uncertain).toBe(false);
    expect(localStorage.getItem('hegemony.pending.v1')).toBeNull();
  });
});
