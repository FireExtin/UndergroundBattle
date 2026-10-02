import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
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
