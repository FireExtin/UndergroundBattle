import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CardContent, CardTile } from './CardTile';
import { testCard, testCatalog } from './testFixtures';

describe('concealed character presentation', () => {
  it('shows damage immunity on own concealed board cards while preserving printed data in inspection', () => {
    const card = { ...testCard, kind: 'hidden', faceDown: true, damage: 2 };
    const definition = { ...testCatalog.cards[0], defense: 1 };
    const { rerender } = render(<CardContent card={card} definition={definition} compact />);
    expect(screen.getByText('暗藏者 · 不受角色伤害')).toBeInTheDocument();
    expect(screen.queryByText('防御 1')).not.toBeInTheDocument();
    expect(screen.queryByText('伤害 2')).not.toBeInTheDocument();
    rerender(<CardContent card={card} definition={definition} />);
    expect(screen.getByText('防御 1')).toBeInTheDocument();
    expect(screen.getByText('真实印刷文字')).toBeInTheDocument();
  });
  it('never reveals a teammate’s or opponent’s concealed identity, even when a stale source includes print data', () => {
    const card = { ...testCard, owner: 'p1', controller: 'p0', kind: 'hidden', faceDown: true, cost: 2, effectiveCost: 1 };
    const { rerender } = render(<CardTile card={card} definition={testCatalog.cards[0]} viewerId="p0" compact />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    expect(screen.queryByText('无知路人')).not.toBeInTheDocument();
    expect(screen.queryByText('真实印刷文字')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('当前费用 1，印刷费用 2')).not.toBeInTheDocument();
    rerender(<CardTile card={card} definition={testCatalog.cards[0]} viewerId="p2" compact />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    rerender(<CardTile card={card} definition={testCatalog.cards[0]} viewerId="p1" compact />);
    expect(screen.getByRole('button', { name: '查看无知路人' })).toBeInTheDocument();
  });
  it('shows the server effective cost and its printed cost without calculating reductions', () => {
    const card = { ...testCard, cost: 3, effectiveCost: 1 };
    const { rerender } = render(<CardContent card={card} definition={testCatalog.cards[0]} compact />);
    expect(screen.getByLabelText('当前费用 1，印刷费用 3')).toHaveTextContent('1');
    rerender(<CardContent card={card} definition={testCatalog.cards[0]} />);
    expect(screen.getByText('当前费用 1 · 印刷费用 3')).toBeInTheDocument();
    rerender(<CardContent card={{ ...card, effectiveCost: 0 }} definition={testCatalog.cards[0]} />);
    expect(screen.getByLabelText('当前费用 0，印刷费用 3')).toHaveTextContent('0');
    expect(screen.getByText('当前费用 0 · 印刷费用 3')).toBeInTheDocument();
    rerender(<CardContent card={{ ...card, effectiveCost: undefined }} definition={testCatalog.cards[0]} />);
    expect(screen.getByLabelText('费用 3')).toHaveTextContent('3');
    expect(screen.queryByText('当前费用 1 · 印刷费用 3')).not.toBeInTheDocument();
  });
});
