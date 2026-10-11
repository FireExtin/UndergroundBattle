import { act, cleanup, fireEvent, render, renderHook, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, saveSession } from './api';
import { GameApp } from './GameApp';
import { testCatalog, testView } from './testFixtures';
import type { Action, SavedSession, View } from './types';
import { useGame } from './useGame';

const seat: SavedSession = { roomId: testView.roomId, seat: 0, token: 'interaction-test-seat', inviteCode: 'INVITE' };
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
type Command = { commandId: string; expectedVersion: number; action: Action };
const body = (init?: RequestInit): Command => JSON.parse(String(init?.body));
const playing: View = { ...testView, status: 'playing', serverNowMs: 1000, legalActions: [{ id: 'pass', kind: 'pass', label: '让过' }] };
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); saveSession(seat); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('visible version conflict recovery', () => {
  it('offers a latest-table read after a rejected action and failed sync without claiming success or repeating the command', async () => {
    const commands: Command[] = [];
    let recover = false;
    const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) { commands.push(body(init)); return json({ error: 'version_conflict' }, 409); }
      if (recover) return json({ ...playing, version: 2 });
      return commands.length ? json({ message: 'sync unavailable' }, 503) : json(playing);
    });
    vi.stubGlobal('fetch', fetchMock);
    render(<GameApp />); await act(async () => {});
    await act(async () => { fireEvent.click(screen.getByRole('button', { name: '让过' })); });
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('请查看最新牌桌后重新选择行动');
    expect(alert).not.toHaveTextContent('已同步');
    expect(readPending(seat)).toBeNull();
    expect(screen.queryByRole('button', { name: '确认上一行动' })).not.toBeInTheDocument();
    expect(commands).toHaveLength(1);
    recover = true;
    fireEvent.click(within(alert).getByRole('button', { name: '查看最新牌桌' }));
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(fetchMock.mock.calls.filter(([url]) => url.includes('/state'))).toHaveLength(3);
    expect(fetchMock.mock.calls.at(-1)?.[0]).toContain('afterVersion=1');
    expect(commands).toHaveLength(1); // A confirmed rejection never becomes an automatic new command.
    expect(readPending(seat)).toBeNull();
  });

  const invalidViews: [string, Partial<View>][] = [
    ['wrong room', { roomId: 'another-room' }], ['wrong seat', { you: 'p1' }],
    ['negative version', { version: -1 }], ['fractional version', { version: 1.5 }],
  ];
  it.each(invalidViews)('reads an authenticated state when the conflict view has a %s', async (_name, invalid) => {
    const commands: Command[] = [];
    const reads: string[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) { commands.push(body(init)); return json({ error: 'version_conflict', view: { ...playing, version: 2, ...invalid } }, 409); }
      reads.push(url); return json(commands.length ? { ...playing, version: 3 } : playing);
    }));
    const hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'pass' }); });
    expect(reads).toHaveLength(2);
    expect(hook.result.current.view).toEqual({ ...playing, version: 3 });
    expect(hook.result.current.error).toContain('已同步最新状态');
    expect(hook.result.current.uncertain).toBe(false);
    expect(commands).toHaveLength(1); expect(readPending(seat)).toBeNull();
  });

  it('keeps the last authenticated view and offers manual refresh when both the conflict view and fallback read are unusable', async () => {
    let sent = 0;
    vi.stubGlobal('fetch', vi.fn(async (url: string) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) { sent++; return json({ error: 'version_conflict', view: { ...playing, you: 'p1', version: 2 } }, 409); }
      return sent ? json({ message: 'offline' }, 503) : json(playing);
    }));
    const hook = renderHook(useGame); await act(async () => {});
    await act(async () => { await hook.result.current.act({ kind: 'pass' }); });
    expect(hook.result.current.view).toEqual(playing);
    expect(hook.result.current.error).not.toContain('已同步');
    expect(hook.result.current.error).toContain('查看最新牌桌');
    expect(sent).toBe(1); expect(hook.result.current.uncertain).toBe(false);
  });

  it('preserves a newer peer poll over a delayed conflict view and never rebases the original request', async () => {
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible');
    const commands: Command[] = [];
    let release: ((response: Response) => void) | undefined;
    vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
      if (url.endsWith('/commands')) { commands.push(body(init)); return new Promise<Response>(resolve => { release = resolve; }); }
      return Promise.resolve(url.endsWith('/catalog') ? json(testCatalog) : json({ ...playing, version: commands.length ? 5 : 3 }));
    }));
    const hook = renderHook(useGame); await act(async () => {});
    let request: Promise<void> | undefined;
    await act(async () => { request = hook.result.current.act({ kind: 'pass' }); });
    act(() => { document.dispatchEvent(new Event('visibilitychange')); });
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    expect(hook.result.current.view?.version).toBe(5);
    await act(async () => { release!(json({ error: 'version_conflict', view: { ...playing, version: 4 } }, 409)); await request; });
    expect(hook.result.current.view?.version).toBe(5);
    expect(hook.result.current.error).toContain('已同步最新状态');
    expect(commands).toHaveLength(1); expect(commands[0].expectedVersion).toBe(3);
    expect(readPending(seat)).toBeNull();
  });

  it('blocks double ready submissions and leaves an independently stale peer to confirm ready explicitly after its normal 409', async () => {
    const peer = { ...seat, seat: 1, token: 'interaction-test-peer' };
    let version = 1;
    const ready = [false, false];
    const commands: { actor: number; command: Command; status: number }[] = [];
    let acknowledgeFirst: (() => void) | undefined;
    const viewFor = (actor: number): View => ({ ...testView, version, serverNowMs: 1000, you: `p${actor}`,
      players: [testView.players[0], { ...testView.players[0], id: 'p1', seat: 1, name: '队友' }].map((player, index) => ({ ...player, ready: ready[index] })),
      legalActions: [{ id: 'ready', kind: 'ready', label: ready[actor] ? '取消准备' : '准备' }] });
    vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
      const actor = new Headers(init?.headers).get('Authorization') === `Bearer ${peer.token}` ? 1 : 0;
      if (url.endsWith('/catalog')) return Promise.resolve(json(testCatalog));
      if (!url.endsWith('/commands')) return Promise.resolve(json(viewFor(actor)));
      const command = body(init);
      const status = command.expectedVersion === version ? 200 : 409;
      commands.push({ actor, command, status });
      if (status === 409) return Promise.resolve(json({ error: 'version_conflict', view: viewFor(actor) }, status));
      ready[actor] = !ready[actor]; version++;
      const result = json(viewFor(actor));
      if (actor === 0) return new Promise<Response>(resolve => { acknowledgeFirst = () => resolve(result); });
      return Promise.resolve(result);
    }));
    const first = renderHook(useGame); await act(async () => {});
    saveSession(peer); const second = renderHook(useGame); await act(async () => {});
    let request: Promise<void> | undefined;
    await act(async () => { request = first.result.current.act({ kind: 'ready' }); void first.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(1);
    expect(commands[0].command.action).toEqual({ kind: 'game', action: { kind: 'ready' } });
    await act(async () => { await second.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(2); expect(commands[1].status).toBe(409);
    expect(second.result.current.error).toContain('你当前尚未准备');
    expect(second.result.current.uncertain).toBe(false); expect(readPending(peer)).toBeNull();
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: 0, ...commands[0].command });
    expect(ready).toEqual([true, false]);
    await act(async () => { acknowledgeFirst!(); await request; });
    expect(commands).toHaveLength(2);
    await act(async () => { await second.result.current.act({ kind: 'ready' }); });
    expect(commands).toHaveLength(3); expect(commands[2].status).toBe(200);
    expect(commands[2].command.expectedVersion).toBe(2);
    expect(commands[2].command.commandId).not.toBe(commands[1].command.commandId);
    expect(ready).toEqual([true, true]);
    expect(readPending(seat)).toBeNull(); expect(readPending(peer)).toBeNull();
  });
});
