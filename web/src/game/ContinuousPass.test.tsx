import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AutoPass } from './AutoPass';
import { Table } from './Table';
import { testChoice, testView } from './testFixtures';
import type { View } from './types';

const pass = { id: 'pass', kind: 'pass', label: '让过' };
const optional = { id: 'asset', kind: 'asset', cardId: 'instance-a', label: '建立资产' };
const base: View = { ...testView, status: 'playing', legalActions: [pass, optional] };
const start = () => fireEvent.click(screen.getByRole('button', { name: '一键连续让过' }));
const cancel = () => fireEvent.click(screen.getByRole('button', { name: '取消连续让过' }));
const advance = (ms = 550) => act(() => vi.advanceTimersByTime(ms));
const status = () => screen.getByLabelText('连续让过状态');
const props = () => ({ enabled: false, onEnabledChange: vi.fn(), busy: false, uncertain: false, connection: 'online' as const, onAction: vi.fn(), onActivityChange: vi.fn() });
beforeEach(() => vi.useFakeTimers());
afterEach(() => { cleanup(); vi.useRealTimers(); });

describe('pass until the selected phase, only in this turn', () => {
  it('passes optional opportunities but stops before the selected action phase', () => {
    const p = props(); const view = { ...base, phase: 'start', step: 'prepare' };
    const app = render(<AutoPass {...p} view={view} />);
    fireEvent.change(screen.getByLabelText('连续让过的目标阶段'), { target: { value: 'action' } });
    start(); expect(p.onEnabledChange).toHaveBeenCalledWith(false);
    advance(); expect(p.onAction).toHaveBeenCalledExactlyOnceWith(pass);
    app.rerender(<AutoPass {...p} view={{ ...view, version: 2, step: 'draw' }} />);
    advance(); expect(p.onAction).toHaveBeenCalledTimes(2);
    app.rerender(<AutoPass {...p} view={{ ...base, version: 3 }} />);
    advance(10_000); expect(p.onAction).toHaveBeenCalledTimes(2);
    expect(status()).toHaveTextContent('已到达行动阶段');
    expect(screen.getByRole('button', { name: '一键连续让过' })).toBeDisabled();
  });

  it('stops before confrontation and does not rearm at later phases of the same turn', () => {
    const p = props(); const app = render(<AutoPass {...p} view={base} />);
    fireEvent.change(screen.getByLabelText('连续让过的目标阶段'), { target: { value: 'confrontation' } });
    start(); advance(); expect(p.onAction).toHaveBeenCalledTimes(1);
    app.rerender(<AutoPass {...p} view={{ ...base, version: 2, phase: 'confrontation', step: 'mobility' }} />);
    advance(5000); expect(p.onAction).toHaveBeenCalledTimes(1);
    expect(status()).toHaveTextContent('已到达对抗阶段');
    app.rerender(<AutoPass {...p} view={{ ...base, version: 3, phase: 'end', step: 'end' }} />);
    advance(5000); expect(p.onAction).toHaveBeenCalledTimes(1);
  });

  it('includes end windows and cleanup, then sends nothing in the new turn', () => {
    const p = props(); const app = render(<AutoPass {...p} view={base} />);
    start(); advance();
    app.rerender(<AutoPass {...p} view={{ ...base, version: 2, phase: 'end', step: 'end' }} />);
    advance(); expect(p.onAction).toHaveBeenCalledTimes(2);
    expect(status()).toHaveTextContent('连续让过已开启');
    app.rerender(<AutoPass {...p} view={{ ...base, version: 3, turn: base.turn + 1, phase: 'start', step: 'prepare' }} />);
    advance(10_000); expect(p.onAction).toHaveBeenCalledTimes(2);
    expect(status()).toHaveTextContent('本回合已结束');
  });

  it('cancels repeated clicks and pending timers, and submits once per authoritative version', () => {
    const p = props(); const app = render(<AutoPass {...p} view={base} />);
    start(); cancel(); advance(1000); expect(p.onAction).not.toHaveBeenCalled();
    start(); advance(300); cancel(); advance(1000); expect(p.onAction).not.toHaveBeenCalled();
    start(); advance(); expect(p.onAction).toHaveBeenCalledTimes(1);
    app.rerender(<AutoPass {...p} view={{ ...base }} />); advance(2000);
    expect(p.onAction).toHaveBeenCalledTimes(1);
    app.rerender(<AutoPass {...p} busy view={{ ...base, version: 2 }} />);
    expect(screen.getByRole('button', { name: '取消连续让过' })).toBeEnabled();
    cancel(); app.rerender(<AutoPass {...p} view={{ ...base, version: 2 }} />);
    advance(2000); expect(p.onAction).toHaveBeenCalledTimes(1);
  });

  it.each([
    { label: 'own mandatory payment/selection', view: { ...base, pendingChoice: testChoice } },
    { label: 'another player’s selection', view: { ...base, waitingChoice: { playerId: 'p1', kind: 'payment', title: '必须支付' } } },
    { label: 'paused table', view: { ...base, pause: { pausedAtMs: 1000, pausedBy: 0 } } },
    { label: 'finished game', view: { ...base, status: 'finished' as const } },
    { label: 'different room', view: { ...base, roomId: 'other-room' } },
    { label: 'different actor', view: { ...base, you: 'p1' } },
    { label: 'offline', connection: 'offline' as const },
    { label: 'unconfirmed receipt', uncertain: true },
    { label: 'manual selection/reader', interacting: true },
    { label: 'manual action', manualEpoch: 1 },
  ])('stops on $label and stays stopped after the blocker clears', ({ label: _label, ...blocker }) => {
    const p = props(); const app = render(<AutoPass {...p} view={base} />);
    start(); advance(300);
    app.rerender(<AutoPass {...p} view={base} {...blocker} />); advance(1000);
    expect(p.onAction).not.toHaveBeenCalled();
    app.rerender(<AutoPass {...p} view={{ ...base, version: 2 }} />); advance(2000);
    expect(p.onAction).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: '取消连续让过' })).not.toBeInTheDocument();
  });

  it('does not stop during a normal in-flight acknowledgement, but stops if it remains uncertain afterwards', () => {
    const p = props(); const app = render(<AutoPass {...p} view={base} />);
    start(); advance();
    app.rerender(<AutoPass {...p} busy view={base} />); advance(2000);
    expect(screen.getByRole('button', { name: '取消连续让过' })).toBeInTheDocument();
    app.rerender(<AutoPass {...p} view={{ ...base, version: 2 }} />); advance();
    expect(p.onAction).toHaveBeenCalledTimes(2);
    app.rerender(<AutoPass {...p} uncertain view={{ ...base, version: 2 }} />);
    expect(status()).toHaveTextContent('尚未确认');
    app.rerender(<AutoPass {...p} view={{ ...base, version: 3 }} />); advance(2000);
    expect(p.onAction).toHaveBeenCalledTimes(2);
  });

  it('starts off after refresh and unregisters its polling activity when unmounted', () => {
    const p = props(); const app = render(<AutoPass {...p} view={base} />);
    start(); advance(300); expect(p.onActivityChange).toHaveBeenLastCalledWith(true);
    app.unmount(); expect(p.onActivityChange).toHaveBeenLastCalledWith(false);
    render(<AutoPass {...p} view={base} />); advance(2000);
    expect(p.onAction).not.toHaveBeenCalled(); expect(status()).toHaveTextContent('已关闭');
  });
});

describe('continuous response decisions', () => {
  const response: View = { ...base, serverNowMs: 1000, stack: [{ id: 'effect', controller: 'p1', label: '对方效果' }],
    responseWindow: { id: 'window', stackTopId: 'effect', holderTeam: 0, canBegin: true, members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 5000 }] } };
  it('declines only the current actor’s undecided response, even when paid optional responses exist', () => {
    const p = props(); const app = render(<AutoPass {...p} view={response} />);
    start(); advance();
    expect(p.onAction).toHaveBeenCalledExactlyOnceWith({ kind: 'passResponse', windowId: 'window' });
    app.rerender(<AutoPass {...p} view={{ ...response, version: 2, responseWindow: { ...response.responseWindow!, members: [{ playerId: 'p1', status: 'undecided', deadlineMs: 5000 }] } }} />);
    advance(2000); expect(p.onAction).toHaveBeenCalledTimes(1);
  });
  it('never submits a response decision after its projected deadline', () => {
    const p = props(); render(<AutoPass {...p} view={{ ...response, responseWindow: { ...response.responseWindow!, members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 1200 }] } }} />);
    start(); advance(1000); expect(p.onAction).not.toHaveBeenCalled();
  });
  it('stops when composing and never converts a paid response into cancelAndPass', () => {
    const p = props(); const app = render(<AutoPass {...p} view={response} />);
    start(); advance(300);
    app.rerender(<AutoPass {...p} view={{ ...response, version: 2, responseWindow: { ...response.responseWindow!, myIntentId: 'intent', members: [{ playerId: 'p0', status: 'composing' }] } }} />);
    advance(1000); expect(p.onAction).not.toHaveBeenCalled();
    expect(status()).toHaveTextContent('正在编写响应');
  });
  it('cancels when a real manual response starts through the table', () => {
    const submit = vi.fn(); render(<Table view={response} catalog={null} busy={false} connection="online" onAction={submit} />);
    start(); advance(300); fireEvent.click(screen.getByRole('button', { name: '连锁' }));
    expect(submit).toHaveBeenCalledWith(expect.objectContaining({ kind: 'beginResponse', windowId: 'window' }));
    advance(2000); expect(submit).toHaveBeenCalledTimes(1);
    expect(status()).toHaveTextContent('手动操作');
  });
});
