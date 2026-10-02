import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCatalog, testView } from './testFixtures';
import { testCard } from './testFixtures';

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
  it('keeps the viewing team below the world in every seat, identifies pieces and opens an owner-only readable card', () => {
    const players = Array.from({ length: 4 }, (_, seat) => ({ ...testView.players[0], id: `p${seat}`, seat, name: `玩家${seat + 1}`, team: Math.floor(seat / 2) }));
    const characters = players.map(player => ({ ...testCard, instanceId: `secret-${player.seat}`, name: `身份${player.seat + 1}`, text: `私密文字${player.seat + 1}`, owner: player.id, controller: player.id, faceDown: true, kind: 'hidden' }));
    const view = { ...testView, status: 'playing' as const, mode: 'teams' as const, players, hand: [], legalActions: [], regions: [{ id: 'region-a', index: 0, cardId: 'world', name: '香港', threshold: 3, points: 3, influence: [0, 0], characters }] };
    const { container, rerender } = render(<Table view={view} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
    for (const player of players) {
      rerender(<Table view={{ ...view, you: player.id }} catalog={testCatalog} busy={false} onAction={vi.fn()} />);
      const sides = container.querySelectorAll('.hg-region-side');
      expect(sides[0]).toHaveAttribute('data-side', 'opponent');
      expect(sides[1]).toHaveAttribute('data-side', 'mine');
      expect(sides[1]).toHaveAttribute('data-team', String(player.team));
      expect(screen.getByRole('button', { name: `查看身份${player.seat + 1}` })).toBeInTheDocument();
      for (const other of players.filter(other => other.id !== player.id)) expect(screen.queryByRole('button', { name: `查看身份${other.seat + 1}` })).not.toBeInTheDocument();
    }
    fireEvent.click(screen.getByRole('button', { name: '查看身份4' }));
    fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
    expect(screen.getByRole('dialog', { name: '放大阅读身份4' })).toBeInTheDocument();
    expect(screen.getAllByText('私密文字4').length).toBeGreaterThan(0);
    expect(screen.queryByText('私密文字1')).not.toBeInTheDocument();
    expect(container.querySelector('.hg-dock-action')).toHaveAttribute('data-next-step', 'wait');
    expect(screen.getByRole('button', { name: '等待玩家 · 可浏览' })).toBeDisabled();
  });
});
