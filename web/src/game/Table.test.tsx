import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCatalog, testView } from './testFixtures';

describe('playable table', () => {
  it('shows readable cards and only the selected card’s server-authored actions', () => {
    const submit = vi.fn(); const action = { id: 'deploy-a', kind: 'deploy', label: '派遣到地区一', cardId: 'instance-a', region: 0 };
    render(<Table view={{ ...testView, status: 'playing', legalActions: [action, { id: 'unrelated', kind: 'asset', label: '不相关行动', cardId: 'another-card' }] }} catalog={testCatalog} busy={false} onAction={submit} />);
    expect(screen.queryByRole('button', { name: '派遣到地区一' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看无知路人' }));
    expect(screen.getAllByText('真实印刷文字').length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole('button', { name: '派遣到地区一' }));
    expect(submit).toHaveBeenCalledWith(action);
    expect(screen.queryByRole('button', { name: '不相关行动' })).not.toBeInTheDocument();
  });
  it('distinguishes waiting chooser from the priority holder', () => {
    render(<Table view={{ ...testView, status: 'playing', legalActions: [], waitingChoice: { playerId: 'p0', kind: 'investigation', title: '排列调查牌' } }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    expect(screen.getByText('等待 甲：排列调查牌')).toBeInTheDocument();
    expect(screen.getByText('优先权')).toBeInTheDocument();
  });
});
