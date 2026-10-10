import { act, cleanup, fireEvent, render, renderHook, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, readSavedSeats, savePending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import { useGame } from './useGame';
import { GameApp } from './GameApp';
import type { SavedSession } from './types';

const first: SavedSession = { roomId: testView.roomId, seat: 0, token: 'token-A', inviteCode: 'INVITE' };
const peer: SavedSession = { ...first, seat: 1, token: 'token-B' };
const legacy = { roomId: first.roomId, commandId: 'legacy-X', expectedVersion: 7, action: { kind: 'game', action: { kind: 'ready' } } };
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const viewFor = (seat: SavedSession, version = 7) => ({ ...testView, roomId: seat.roomId, you: `p${seat.seat}`, version, serverNowMs: 1000 });
const actorFor = (init?: RequestInit) => new Headers(init?.headers).get('Authorization');
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('independent review recovery regressions', () => {
  it('never dispatches a seatless legacy command when two hooks restore around suspended state reads and a changed shared session', async () => {
    saveSession(first); localStorage.setItem('hegemony.pending.v1', JSON.stringify(legacy));
    const reads = new Map<string, (response: Response) => void>();
    const commands: { actor: string | null; body: unknown }[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      const actor = actorFor(init);
      if (url.endsWith('/commands')) {
        commands.push({ actor, body: JSON.parse(String(init?.body)) });
        return json(viewFor(actor === 'Bearer token-B' ? peer : first, 8));
      }
      return new Promise<Response>(resolve => { reads.set(actor!, resolve); });
    }));
    const firstHook = renderHook(useGame); await act(async () => {});
    expect(reads.has('Bearer token-A')).toBe(true);
    saveSession(peer);
    const peerHook = renderHook(useGame); await act(async () => {});
    expect(reads.has('Bearer token-B')).toBe(true);
    // Release B first: the old client would submit X under B before A ever reaches the service.
    await act(async () => { reads.get('Bearer token-B')!(json(viewFor(peer))); });
    await act(async () => { reads.get('Bearer token-A')!(json(viewFor(first))); });
    expect(commands).toEqual([]);
    expect(firstHook.result.current.view?.you).toBe('p0'); expect(peerHook.result.current.view?.you).toBe('p1');
    expect(firstHook.result.current.error).toContain('原座位'); expect(peerHook.result.current.error).toContain('原座位');
    await act(async () => { await peerHook.result.current.act({ kind: 'ready' }); peerHook.result.current.retryPending(); });
    expect(commands).toEqual([]);
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(JSON.stringify(legacy));
    expect([...Array(localStorage.length)].map((_, i) => localStorage.key(i)).some(key => key?.startsWith('hegemony.pending.v2.'))).toBe(false);
  });

  it('offers a read-only latest-table action for an unbound record instead of a command confirmation', async () => {
    saveSession(first); localStorage.setItem('hegemony.pending.v1', JSON.stringify(legacy));
    const paths: string[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string) => { paths.push(url); return url.endsWith('/catalog') ? json(testCatalog) : json(viewFor(first)); }));
    render(<GameApp />); await act(async () => {});
    expect(screen.getByText(/旧行动缺少原座位记录，不能安全自动恢复/)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '确认上一行动' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看最新牌桌' }));
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(paths.filter(path => path.includes('/state'))).toHaveLength(2);
    expect(paths.some(path => path.endsWith('/commands'))).toBe(false);
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(JSON.stringify(legacy));
  });

  it.each([
    ['invalid_seat_token', 401, 'read'], ['room_not_found', 404, 'read'], ['unsupported_room_version', 410, 'read'],
    ['invalid_seat_token', 401, 'command'], ['room_not_found', 404, 'command'], ['unsupported_room_version', 410, 'command'],
  ] as const)('retires all rejected actor records on %s (%i) from a %s path and permits recovery of the surviving actors', async (code, status, rejectionPath) => {
    const other = { ...first, roomId: 'other-room', inviteCode: 'OTHER', token: 'other-token' };
    saveSession(other); saveSession(peer); saveSession(first);
    const commands = [
      { ...legacy, seat: 0, commandId: 'actor-first' }, { ...legacy, seat: 0, commandId: 'actor-second' },
      { ...legacy, roomId: other.roomId, seat: 0, commandId: 'other-room-command' },
      { ...legacy, seat: 1, commandId: 'other-seat-command' },
    ];
    commands.forEach(command => savePending(command));
    let rejected = false; const submitted: { actor: string | null; commandId: string }[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url === '/api/rooms') return json({ ...other, roomId: 'new-room', view: { ...viewFor(other), roomId: 'new-room' } });
      const actor = actorFor(init);
      if (actor === 'Bearer token-A' && !rejected && (rejectionPath === 'read' || url.endsWith('/commands'))) {
        rejected = true; return json({ error: code, message: 'definitive rejection' }, status);
      }
      if (url.endsWith('/commands')) submitted.push({ actor, commandId: JSON.parse(String(init?.body)).commandId });
      return json(viewFor(actor === 'Bearer other-token' ? other : actor === 'Bearer token-B' ? peer : first));
    }));
    const hook = renderHook(useGame); await act(async () => {});
    expect(hook.result.current.session).toBeNull();
    expect(readPending(first)).toBeNull();
    expect(readPending(other)).toEqual(commands[2]); expect(readPending(peer)).toEqual(commands[3]);
    if (code !== 'unsupported_room_version') expect(readSavedSeats()).toEqual([other, peer]);
    await act(async () => { hook.result.current.resume(other); });
    expect(hook.result.current.view?.roomId).toBe(other.roomId); expect(readPending(other)).toBeNull();
    await act(async () => { hook.result.current.resume(peer); });
    expect(hook.result.current.view?.you).toBe('p1'); expect(readPending(peer)).toBeNull();
    expect(submitted).toEqual([{ actor: 'Bearer other-token', commandId: commands[2].commandId }, { actor: 'Bearer token-B', commandId: commands[3].commandId }]);
    await act(async () => { hook.result.current.leave(); });
    await act(async () => { await hook.result.current.create('new', 'duel', 'watchers'); });
    expect(hook.result.current.session?.roomId).toBe('new-room'); expect(hook.result.current.uncertain).toBe(false);
  });

  it('does not turn an acknowledged legacy receipt into uncertainty when both migration and retirement writes fail', async () => {
    saveSession(first); const original = { ...legacy, seat: 0 };
    localStorage.setItem('hegemony.pending.v1', JSON.stringify(original));
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('quota'); });
    const commands: { commandId: string; expectedVersion: number; action: { kind: string; action?: { kind: string } } }[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) {
        commands.push(JSON.parse(String(init?.body)));
        return json(viewFor(first, commands.at(-1)?.commandId === original.commandId ? 8 : 9));
      }
      return url.endsWith('/catalog') ? json(testCatalog) : json(viewFor(first, commands.length ? 9 : 7));
    }));
    let hook = renderHook(useGame); await act(async () => {});
    expect(commands).toHaveLength(1); expect(commands[0].commandId).toBe(original.commandId);
    expect(hook.result.current.uncertain).toBe(false);
    await act(async () => { await hook.result.current.act({ kind: 'deck', option: 'watchers' }); });
    expect(commands).toHaveLength(2); expect(commands[1].commandId).not.toBe(original.commandId);
    expect(commands[1].action).toEqual({ kind: 'game', action: { kind: 'deck', option: 'watchers' } });
    expect(hook.result.current.uncertain).toBe(false);
    expect(localStorage.getItem('hegemony.pending.v1')).toBe(JSON.stringify(original));
    hook.unmount(); hook = renderHook(useGame); await act(async () => {});
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]); // Fresh page confirms the original identity.
    expect(hook.result.current.uncertain).toBe(false); expect(hook.result.current.view?.version).toBe(9);
  });
});
