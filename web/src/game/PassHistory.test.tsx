import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { LatestPass, TableRecords, passRecordText } from './PassHistory';
import { Table } from './Table';
import { testCatalog, testView } from './testFixtures';
import type { PassRecord, View } from './types';

const record: PassRecord = { id: '8:0', version: 8, playerId: 'p1', playerName: '乙', turn: 2,
  phase: 'confrontation', step: 'region:0:combat:before', region: { index: 0, instanceId: 'old-region', name: '旧港' }, reason: 'response', responseWindowId: 'response:1' };

describe('public pass history uses event-time context', () => {
  it('shows the same recorded actor, phase and original region to either seat after a phase or region changes', () => {
    const view: View = { ...testView, status: 'playing', version: 20, turn: 3, phase: 'end', step: 'end',
      players: [{ ...testView.players[0], name: '甲' }, { ...testView.players[0], id: 'p1', seat: 1, name: '乙的新昵称', team: 1 }],
      regions: [], hand: [], legalActions: [], passHistory: [record] };
    const app = render(<Table view={view} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    const expected = '乙 · 第 2 回合 · 对抗阶段 · 战斗 · 奖励前快速窗口 · 地区 1 · 旧港 · 响应让过';
    expect(screen.getByLabelText('最近让过')).toHaveTextContent(expected);
    expect(screen.getByLabelText('最近让过')).not.toHaveTextContent('乙的新昵称');
    fireEvent.click(screen.getByText('记录 · 1'));
    expect(within(screen.getByLabelText('牌桌行动记录')).getByText(expected)).toBeInTheDocument();
    app.rerender(<Table view={{ ...view, you: 'p1' }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByLabelText('最近让过')).toHaveTextContent(expected);
  });
  it.each([['start','prepare','开始阶段 · 重置与准备'], ['start','draw','开始阶段 · 抓牌'], ['action','team:1','行动阶段 · 团队 2 行动'], ['confrontation','mobility','对抗阶段 · 机动窗口'], ['end','end','结束阶段']])('marks %s/%s as global', (phase, step, label) => {
    expect(passRecordText({ ...record, phase, step, region: null, reason: 'pass' })).toBe(`乙 · 第 2 回合 · ${label} · 全局 · 让过`);
  });
  it('preserves response cancel/timeout reasons, escapes names, and does not guess unrecorded passes', () => {
    const app = render(<LatestPass history={undefined} />);
    expect(screen.queryByLabelText('最近让过')).not.toBeInTheDocument();
    app.rerender(<LatestPass history={[{ ...record, reason: 'timeout', playerName: '<script>bad()</script>' }]} />);
    expect(screen.getByLabelText('最近让过')).toHaveTextContent('<script>bad()</script>');
    expect(document.querySelector('script')).toBeNull();
    expect(screen.getByLabelText('最近让过')).toHaveTextContent('响应超时，自动让过');
    app.rerender(<LatestPass history={[{ ...record, reason: 'cancelResponse' }]} />);
    expect(screen.getByLabelText('最近让过')).toHaveTextContent('取消响应并让过');
  });
  it('merges public passes with ordinary records by revision and bounds the visible history', () => {
    const passHistory = Array.from({ length: 50 }, (_, index) => ({ ...record, version: index + 1, id: `${index + 1}:0`, playerName: `玩家${index}` }));
    render(<TableRecords view={{ ...testView, passHistory, log: [{ version: 51, text: '新动作' }] }} />);
    const entries = within(screen.getByLabelText('牌桌行动记录')).getAllByRole('listitem', { hidden: true });
    expect(entries).toHaveLength(50); expect(entries[0]).toHaveTextContent('新动作');
    expect(entries[1]).toHaveTextContent('玩家49'); expect(entries.at(-1)).toHaveTextContent('玩家1');
  });
});
