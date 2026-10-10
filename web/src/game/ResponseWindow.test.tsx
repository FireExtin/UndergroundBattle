import { act, cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ResponseWindow } from './ResponseWindow';
import { testChoice, testView } from './testFixtures';
import type { View } from './types';

describe('public stack projection', () => {
  it('uses public target labels and server validity even when no matching board instance exists', () => {
    const view: View = { ...testView, status: 'playing', stack: [{ id: 'effect', label: '公开效果', controller: 'p0',
      targetSummaries: [{ instanceId: 'private-target', label: '暗藏者·甲', kind: 'hidden', valid: true, status: 'valid' }] }], legalActions: [] };
    const { rerender } = render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} />);
    expect(screen.getByText('暗藏者·甲')).toBeInTheDocument();
    expect(screen.getByText('有效目标')).toBeInTheDocument();
    expect(screen.queryByText('无知路人')).not.toBeInTheDocument();
    rerender(<ResponseWindow view={{ ...view, stack: [{ ...view.stack[0], resolutionState: 'resolving', targetSummaries: [{ ...view.stack[0].targetSummaries![0], status: 'guardAccepted' }] }] }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} />);
    expect(screen.getByText('已进入结算')).toBeInTheDocument();
    expect(screen.getByText('正在结算堆顶效果')).toBeInTheDocument();
  });
  it('shows private and public choice pauses separately from responding', () => {
    const view: View = { ...testView, status: 'playing', stack: [{ id: 'effect', label: '公开效果', controller: 'p0' }], pendingChoice: testChoice };
    const { rerender } = render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} />);
    expect(screen.getByText('请你完成选择：选择目标')).toBeInTheDocument();
    rerender(<ResponseWindow view={{ ...view, pendingChoice: null, waitingChoice: { playerId: 'p0', kind: 'target', title: '选择牺牲角色' } }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} />);
    expect(screen.getByText('等待 甲 完成选择：选择牺牲角色')).toBeInTheDocument();
  });
  it('keeps older stack views usable when target summaries and resolution state are absent', () => {
    render(<ResponseWindow view={{ ...testView, status: 'playing', stack: [{ id: 'legacy-effect', label: '旧服务效果', controller: 'p0', targetId: 'unknown-instance' }], legalActions: [{ id: 'pass', kind: 'pass', label: '让过' }] }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} />);
    expect(screen.getByText('旧服务效果')).toBeInTheDocument();
    expect(screen.getByText('你当前只能让过，交出优先权')).toBeInTheDocument();
    expect(screen.queryByLabelText('公开目标')).not.toBeInTheDocument();
    expect(screen.queryByText('目标已失效')).not.toBeInTheDocument();
  });
});

describe('server-owned response intent', () => {
  const fast = { id: 'fast', kind: 'activate', label: '响应行动', cardId: 'source-instance' };
  const view: View = { ...testView, status: 'playing', serverNowMs: 100_000,
    stack: [{ id: 'effect', label: '待结算效果', controller: 'p1' }], legalActions: [{ id: 'pass', kind: 'pass', label: '让过' }, fast],
    responseWindow: { id: 'window-one', stackTopId: 'effect', holderTeam: 0, canBegin: true,
      members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 105_000 }] } };
  beforeEach(() => vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval', 'performance'] }));
  afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });
  const advance = (ms: number) => act(() => vi.advanceTimersByTime(ms));

  it('freezes remaining response time and all intent controls for a saved paused table', () => {
    const submit = vi.fn();
    const { rerender } = render(<ResponseWindow view={{ ...view, serverNowMs: 102_000, pause: { pausedAtMs: 102_000, pausedBy: 1 } }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={submit} />);
    expect(screen.getByRole('timer')).toHaveTextContent('剩余 3 秒');
    advance(86_400_000); expect(screen.getByRole('timer')).toHaveTextContent('剩余 3 秒');
    fireEvent.click(screen.getByRole('button', { name: '连锁' }));
    fireEvent.click(screen.getByRole('button', { name: '不连锁，让过' }));
    expect(submit).not.toHaveBeenCalled();
    rerender(<ResponseWindow view={{ ...view, version: view.version + 1, serverNowMs: 86_502_000, responseWindow: { ...view.responseWindow!, id: 'resumed-window', members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 86_505_000 }] } }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={submit} />);
    expect(screen.getByRole('timer')).toHaveTextContent('剩余 3 秒');
    advance(1000); expect(screen.getByRole('timer')).toHaveTextContent('剩余 2 秒');
  });

  it('shows the server-sampled five seconds and waits for confirmation at zero without passing locally', () => {
    const submit = vi.fn();
    render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={submit} />);
    const timer = screen.getByRole('timer', { name: '连锁决定倒计时' });
    expect(timer).toHaveTextContent('剩余 5 秒');
    expect(screen.getByRole('button', { name: '连锁' })).toBeEnabled();
    advance(4500); expect(timer).toHaveTextContent('剩余 1 秒');
    advance(500); expect(timer).toHaveTextContent('剩余 0 秒');
    expect(screen.getByText('等待服务器确认响应决定')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '连锁' })).toBeDisabled();
    expect(screen.getByRole('button', { name: '不连锁，让过' })).toBeDisabled();
    expect(screen.getByRole('region', { name: '当前待结算效果与响应' })).toHaveAttribute('data-response-state', 'undecided');
    advance(20_000); expect(submit).not.toHaveBeenCalled();
  });

  it('begins an intent for this window without choosing or paying for a card', () => {
    const submit = vi.fn(); const select = vi.fn();
    const intentId = '11111111-2222-3333-4444-555555555555';
    vi.spyOn(crypto, 'randomUUID').mockReturnValue(intentId);
    render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={select} onAction={submit} />);
    expect(screen.queryByRole('button', { name: '查看响应牌' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '连锁' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ kind: 'beginResponse', windowId: 'window-one', intentId });
    expect(select).not.toHaveBeenCalled();
  });

  it('lets only this undecided member pass when its private begin qualification is false', () => {
    const submit = vi.fn();
    render(<ResponseWindow view={{ ...view, responseWindow: { ...view.responseWindow!, canBegin: false } }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={submit} />);
    expect(screen.getByRole('button', { name: '连锁' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: '不连锁，让过' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ kind: 'passResponse', windowId: 'window-one' });
  });

  it('rebases elapsed display on a fresh trusted server sample', () => {
    const { rerender } = render(<ResponseWindow view={view} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={vi.fn()} />);
    advance(2000);
    expect(screen.getByRole('timer')).toHaveTextContent('剩余 3 秒');
    rerender(<ResponseWindow view={{ ...view, serverNowMs: 104_000 }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={vi.fn()} />);
    expect(screen.getByRole('timer')).toHaveTextContent('剩余 1 秒');
    advance(1000); expect(screen.getByRole('timer')).toHaveTextContent('剩余 0 秒');
  });

  it('restores composing from the server intent, keeps selection untimed and cancels that same intent', () => {
    const submit = vi.fn(); const select = vi.fn();
    const composing: View = { ...view, responseWindow: { ...view.responseWindow!, myIntentId: 'restored-intent', members: [{ playerId: 'p0', status: 'composing' }] } };
    render(<ResponseWindow view={composing} busy={false} uncertain={false} connection="online" onSelectCard={select} onAction={submit} />);
    expect(screen.getByText('正在选择响应，确认前可取消')).toBeInTheDocument();
    expect(screen.queryByRole('timer')).not.toBeInTheDocument();
    advance(60_000);
    expect(screen.getByRole('button', { name: '取消并让过' })).toBeEnabled();
    fireEvent.click(screen.getByRole('button', { name: '查看响应牌' }));
    expect(select).toHaveBeenCalledExactlyOnceWith('source-instance');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '取消并让过' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ kind: 'cancelAndPass', windowId: 'window-one', intentId: 'restored-intent' });
  });

  it('keeps a passed teammate separate from another member still composing without reopening responses', () => {
    const teammate = { ...testView.players[0], id: 'p1', seat: 1, name: '乙' };
    const opponents = [2, 3].map(seat => ({ ...testView.players[0], id: `p${seat}`, seat, name: `对手${seat}`, team: 1 }));
    render(<ResponseWindow view={{ ...view, mode: 'teams', players: [...view.players, teammate, ...opponents], responseWindow: { ...view.responseWindow!, canBegin: false, members: [{ playerId: 'p0', status: 'passed' }, { playerId: 'p1', status: 'composing' }] } }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={vi.fn()} />);
    expect(screen.getByText('已让过，等待 乙 完成响应选择')).toBeInTheDocument();
    expect(screen.getByLabelText('响应决定状态')).toHaveTextContent('甲 · 已让过 / 乙 · 正在选择响应');
    expect(screen.queryByRole('timer')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '连锁' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '查看响应牌' })).not.toBeInTheDocument();
    expect(screen.queryByText(/全队.*让过|没有可用响应/)).not.toBeInTheDocument();
  });

  it.each([{ busy: true }, { uncertain: true }, { connection: 'offline' as const }, { connection: 'connecting' as const }])('does not send decision or cancellation commands while blocked by %o', blocker => {
    const submit = vi.fn();
    const props = { busy: false, uncertain: false, connection: 'online' as const, ...blocker };
    const { rerender } = render(<ResponseWindow view={view} {...props} onSelectCard={vi.fn()} onAction={submit} />);
    const begin = screen.getByRole('button', { name: '连锁' });
    const pass = screen.getByRole('button', { name: '不连锁，让过' });
    expect(begin).toBeDisabled(); expect(pass).toBeDisabled();
    fireEvent.click(begin); fireEvent.click(pass);
    rerender(<ResponseWindow view={{ ...view, responseWindow: { ...view.responseWindow!, myIntentId: 'restored-intent', members: [{ playerId: 'p0', status: 'composing' }] } }} {...props} onSelectCard={vi.fn()} onAction={submit} />);
    const cancel = screen.getByRole('button', { name: '取消并让过' });
    expect(cancel).toBeDisabled(); fireEvent.click(cancel);
    expect(submit).not.toHaveBeenCalled();
  });

  it('does not time a pending game choice or offer intent commands during it', () => {
    const submit = vi.fn();
    render(<ResponseWindow view={{ ...view, pendingChoice: testChoice }} busy={false} uncertain={false} connection="online" onSelectCard={vi.fn()} onAction={submit} />);
    expect(screen.getByText('请你完成选择：选择目标')).toBeInTheDocument();
    expect(screen.queryByRole('timer')).not.toBeInTheDocument();
    expect(within(screen.getByRole('region', { name: '当前待结算效果与响应' })).queryByRole('button')).not.toBeInTheDocument();
    advance(60_000); expect(submit).not.toHaveBeenCalled();
  });
});
