import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CardContent, CardTile } from './CardTile';
import { testCard, testCatalog } from './testFixtures';

describe('concealed character presentation', () => {
  it('shows damage immunity on own concealed board cards while preserving printed data in inspection', () => {
    const card = { ...testCard, kind: 'hidden', faceDown: true, damage: 2 };
    const definition = { ...testCatalog.cards[0], defense: 1 };
    const { rerender } = render(<CardContent card={card} definition={definition} compact />);
    expect(screen.getByText('暗藏者 · 基础势力 1 · 不受任何伤害')).toBeInTheDocument();
    expect(screen.getByLabelText('图标：调查0，战斗0，势力1')).toBeInTheDocument();
    expect(screen.queryByText('防御 1')).not.toBeInTheDocument();
    expect(screen.queryByText('伤害 2')).not.toBeInTheDocument();
    rerender(<CardContent card={card} definition={definition} />);
    expect(screen.getByText('防御 1')).toBeInTheDocument();
    expect(screen.getByText('真实印刷文字')).toBeInTheDocument();
  });
  it('never reveals a teammate’s or opponent’s concealed identity, even when a stale source includes print data', () => {
    const card = { ...testCard, owner: 'p1', controller: 'p0', kind: 'hidden', faceDown: true, cost: 2, effectiveCost: 1,
      icons: { investigation: 7, combat: 8, influence: 9 } };
    const { rerender } = render(<CardTile card={card} definition={testCatalog.cards[0]} viewerId="p0" compact />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    expect(screen.queryByText('无知路人')).not.toBeInTheDocument();
    expect(screen.queryByText('真实印刷文字')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('当前费用 1，印刷费用 2')).not.toBeInTheDocument();
    expect(screen.getByLabelText('图标：调查0，战斗0，势力1')).toBeInTheDocument();
    expect(screen.queryByLabelText('图标：调查7，战斗8，势力9')).not.toBeInTheDocument();
    rerender(<CardTile card={card} definition={testCatalog.cards[0]} viewerId="p2" compact />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    rerender(<CardTile card={card} definition={testCatalog.cards[0]} viewerId="p1" compact />);
    expect(screen.getByRole('button', { name: '查看无知路人' })).toBeInTheDocument();
  });
  it('explains an exhausted anonymous hidden card without suggesting that it participates in combat', () => {
    render(<CardContent card={{ ...testCard, cardId: undefined, kind: 'hidden', faceDown: true, exhausted: true }} compact />);
    expect(screen.getByText('暗藏者 · 基础势力 1 · 不受任何伤害。已横置，不参与对抗。')).toBeInTheDocument();
    expect(screen.getByLabelText('图标：调查0，战斗0，势力1')).toBeInTheDocument();
    expect(screen.queryByText('真实印刷文字')).not.toBeInTheDocument();
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

describe('attachment and current-stat reading', () => {
  it('shows authoritative effective icons and defense separately from print, including zero values', () => {
    const definition = { ...testCatalog.cards[0], defense: 1, permanentIcons: { investigation: 1, combat: 1, influence: 1 } };
    const card = { ...testCard, region: 0, icons: { investigation: 2, combat: 7, influence: 4 }, defense: 3 };
    const { rerender } = render(<CardContent card={card} definition={definition} attachmentCount={2} />);
    expect(screen.getByLabelText('当前有效图标：调查2，战斗7，势力4')).toBeInTheDocument();
    expect(screen.getByLabelText('印刷图标：调查1，战斗1，势力1')).toBeInTheDocument();
    expect(screen.getByText('当前防御 3 · 印刷防御 1')).toBeInTheDocument();
    expect(screen.getByLabelText('附属 2 张')).toBeInTheDocument();
    rerender(<CardContent card={{ ...card, exhausted: true, icons: { investigation: 0, combat: 0, influence: 0 }, defense: 0 }} definition={definition} />);
    expect(screen.getByLabelText('当前有效图标：调查0，战斗0，势力0')).toBeInTheDocument();
    expect(screen.getByText('当前防御 0 · 印刷防御 1')).toBeInTheDocument();
    expect(screen.getByLabelText('印刷图标：调查1，战斗1，势力1')).toBeInTheDocument();
  });

  it('does not describe effective values as print when the frozen catalog is unavailable', () => {
    render(<CardContent card={{ ...testCard, region: 0, icons: { investigation: 0, combat: 6, influence: 0 } }} />);
    expect(screen.getByLabelText('当前有效图标：调查0，战斗6，势力0')).toBeInTheDocument();
    expect(screen.queryByText('印刷图标')).not.toBeInTheDocument();
  });

  it('identifies a face-up attachment without presenting it as an independent fighting character', () => {
    const card = { ...testCard, kind: 'attachment', name: '示例附属', cost: 1, defense: undefined, icons: undefined };
    const definition = { ...testCatalog.cards[0], kind: 'attachment', subtypes: ['物品'], text: '宿主获得印刷能力。' };
    render(<CardContent card={card} definition={definition} />);
    expect(screen.getByText('附属 · 物品')).toBeInTheDocument();
    expect(screen.getByLabelText('费用 1')).toBeInTheDocument();
    expect(screen.queryByLabelText('图标：调查0，战斗0，势力1')).not.toBeInTheDocument();
    expect(screen.getByText('真实印刷文字')).toBeInTheDocument();
  });

  it('keeps world reading values separate from character effective stats', () => {
    render(<CardContent card={{ ...testCard, kind: 'region', region: 0 }} definition={{ ...testCatalog.cards[0], points: 3, threshold: 4 }} />);
    expect(screen.getByText('赢得 3 分 · 控制阈值 4')).toBeInTheDocument();
    expect(screen.queryByText('当前有效')).not.toBeInTheDocument();
    expect(screen.queryByText('印刷图标')).not.toBeInTheDocument();
  });
});
