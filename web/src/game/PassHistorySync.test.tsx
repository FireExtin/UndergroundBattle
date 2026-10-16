import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { GameApp } from './GameApp';
import { saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import type { Action, PassRecord, View } from './types';

const sessions = [0,1].map(seat => ({ roomId: testView.roomId, inviteCode: 'LOCAL-ONLY', seat, token: `pass-history-synthetic-seat-${seat}` }));
const pass = { id: 'pass', kind: 'pass', label: '让过' };
const json = (value: unknown) => new Response(JSON.stringify(value));
const advance = async (ms: number) => { await act(async () => { await vi.advanceTimersByTimeAsync(ms); }); };
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); history.replaceState({}, '', '/'); vi.useFakeTimers(); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('pass history follows the authenticated acknowledgement and refresh pipeline', () => {
  it.each(['manual', 'auto-empty', 'auto-response'] as const)('%s remains visible to the other client and after both remount, without altering the command', async mode => {
    let server: View = { ...testView, status: 'playing', phase: 'action', step: 'team:0', hand: [], serverNowMs: 0,
      legalActions: [pass], players: [testView.players[0], { ...testView.players[0], id: 'p1', seat: 1, name: '乙', team: 1 }] };
    if (mode === 'auto-response') server = { ...server, stack: [{ id: 'effect', label: '公开效果', controller: 'p1' }],
      responseWindow: { id: 'response:1', stackTopId: 'effect', holderTeam: 0, canBegin: false, members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 10000 }] } };
    const commands: { action: Action; commandId: string; expectedVersion: number }[] = [];
    vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith('/catalog')) return json(testCatalog);
      const seat = new Headers(init?.headers).get('Authorization') === `Bearer ${sessions[1].token}` ? 1 : 0;
      if (url.endsWith('/commands')) {
        const command = JSON.parse(String(init?.body)); commands.push(command);
        const record: PassRecord = { id: '2:0', version: 2, playerId: 'p0', playerName: '甲', turn: 1,
          phase: 'action', step: 'team:0', region: null, reason: mode === 'auto-response' ? 'response' : 'pass',
          ...(mode === 'auto-response' ? { responseWindowId: 'response:1' } : {}) };
        server = { ...server, version: 2, phase: 'end', step: 'end', legalActions: [], responseWindow: null, stack: [], passHistory: [record] };
      }
      return json({ ...server, you: `p${seat}` });
    }));
    saveSession(sessions[0]); let app = render(<GameApp />); await advance(0);
    expect(screen.queryByLabelText('最近让过')).not.toBeInTheDocument();
    if (mode === 'manual') fireEvent.click(screen.getByRole('button', { name: /^让过$/ }));
    else if (mode === 'auto-empty') fireEvent.click(screen.getByRole('switch', { name: '无可用行动时自动让过' }));
    else fireEvent.click(screen.getByRole('button', { name: '一键连续让过' }));
    await advance(600);
    expect(commands).toHaveLength(1);
    expect(commands[0].action).toEqual(mode === 'auto-response' ? { kind: 'passResponse', windowId: 'response:1' } : { kind: 'game', action: { kind: 'pass' } });
    const expected = `甲 · 第 1 回合 · 行动阶段 · 团队 1 行动 · 全局 · ${mode === 'auto-response' ? '响应让过' : '让过'}`;
    expect(screen.getByLabelText('最近让过')).toHaveTextContent(expected);
    expect(screen.getByLabelText('最近让过')).not.toHaveTextContent('结束阶段');
    for (const session of [sessions[1], sessions[0]]) {
      app.unmount(); saveSession(session); app = render(<GameApp />); await advance(0);
      expect(screen.getByLabelText('最近让过')).toHaveTextContent(expected);
      expect(screen.getByLabelText('牌桌行动记录').querySelectorAll('[data-pass-record]')).toHaveLength(1);
    }
    expect(commands).toHaveLength(1);
  });
});
