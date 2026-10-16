import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { GameApp } from './GameApp';
import { readPending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, View } from './types';

const seat = { roomId: testView.roomId, seat: 0, token: 'continuous-pass-recovery-test', inviteCode: 'INVITE' };
const pass = { id: 'pass', kind: 'pass', label: '让过' };
const json = (value: unknown) => new Response(JSON.stringify(value));
type Command = { commandId: string; expectedVersion: number; action: Action };
const playing = (): View => ({ ...testView, status: 'playing', phase: 'start', step: 'prepare', serverNowMs: 1000, legalActions: [pass] });
const advance = async (ms: number) => { await act(async () => { await vi.advanceTimersByTimeAsync(ms); }); };
const start = () => fireEvent.click(screen.getByRole('button', { name: '一键连续让过' }));
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); history.replaceState({}, '', '/'); vi.useFakeTimers(); saveSession(seat); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('continuous passes use the existing authenticated receipt pipeline', () => {
  it('submits distinct acknowledged passes with the original actor, stops at action, and remains off on refresh', async () => {
    let server = playing(); const commands: Command[] = []; const actors: string[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (!url.endsWith('/commands')) return json(server);
      commands.push(JSON.parse(String(init?.body))); actors.push(new Headers(init?.headers).get('Authorization') || '');
      server = { ...server, version: server.version + 1, phase: commands.length === 1 ? 'start' : 'action', step: commands.length === 1 ? 'draw' : 'team:0' };
      return json(server);
    }));
    let app = render(<GameApp />); await act(async () => {});
    fireEvent.change(screen.getByLabelText('连续让过的目标阶段'), { target: { value: 'action' } });
    start(); await advance(600); await advance(600);
    expect(commands).toHaveLength(2);
    expect(commands.map(command => command.expectedVersion)).toEqual([1, 2]);
    expect(new Set(commands.map(command => command.commandId)).size).toBe(2);
    expect(commands.every(command => JSON.stringify(command.action) === JSON.stringify({ kind: 'game', action: { kind: 'pass' } }))).toBe(true);
    expect(actors).toEqual([`Bearer ${seat.token}`, `Bearer ${seat.token}`]);
    expect(screen.getByLabelText('连续让过状态')).toHaveTextContent('已到达行动阶段');
    await advance(3000); expect(commands).toHaveLength(2);
    app.unmount(); app = render(<GameApp />); await act(async () => {}); await advance(2000);
    expect(screen.getByLabelText('连续让过状态')).toHaveTextContent('已关闭');
    expect(commands).toHaveLength(2); expect(readPending(seat)).toBeNull();
  });

  it('retains one passResponse identity after a lost acknowledgement, refreshes it, and never restarts the run', async () => {
    let server: View = { ...playing(), phase: 'action', stack: [{ id: 'effect', controller: 'p1', label: '对方响应' }],
      responseWindow: { id: 'response-1', stackTopId: 'effect', holderTeam: 0, canBegin: true, members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 10_000 }] } };
    const commands: Command[] = []; let recover = false;
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (!url.endsWith('/commands')) return json(server);
      commands.push(JSON.parse(String(init?.body)));
      if (!recover) {
        server = { ...server, version: 2, responseWindow: { ...server.responseWindow!, members: [{ playerId: 'p0', status: 'passed' }] } };
        throw new Error('committed response acknowledgement lost');
      }
      return json({ ...server, version: 2 });
    }));
    let app = render(<GameApp />); await act(async () => {}); start(); await advance(600);
    expect(commands).toHaveLength(2); expect(commands[1]).toEqual(commands[0]);
    expect(commands[0].action).toEqual({ kind: 'passResponse', windowId: 'response-1' });
    expect(readPending(seat)).toEqual({ roomId: seat.roomId, seat: seat.seat, ...commands[0] });
    expect(screen.getByLabelText('连续让过状态')).toHaveTextContent('尚未确认');
    app.unmount(); recover = true;
    server = { ...server, version: 3, responseWindow: { ...server.responseWindow!, id: 'response-2', members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 10_000 }] } };
    app = render(<GameApp />); await act(async () => {}); await advance(2500);
    expect(commands).toHaveLength(3); expect(commands[2]).toEqual(commands[0]);
    expect(readPending(seat)).toBeNull();
    expect(screen.getByLabelText('连续让过状态')).toHaveTextContent('已关闭');
  });

  it('stops on read disconnection while waiting for another player and stays off after reconnecting', async () => {
    let offline = false; const commands: Command[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      if (url.endsWith('/commands')) { commands.push(JSON.parse(String(init?.body))); return json(playing()); }
      if (offline) throw new Error('read offline');
      return json({ ...playing(), legalActions: [] });
    }));
    render(<GameApp />); await act(async () => {}); start(); offline = true;
    await advance(1500); expect(screen.getByLabelText('连续让过状态')).toHaveTextContent('连接中断');
    offline = false; await advance(4000);
    expect(screen.queryByRole('button', { name: '取消连续让过' })).not.toBeInTheDocument();
    expect(commands).toEqual([]);
  });
});
