import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CardContent, visibleCard } from './CardTile';
import type { Card, CardDefinition } from './types';
const definition: CardDefinition = { id: 'XQ19', name: '熵力化身', kind: 'character', cost: 0, costAppearance: 'dash', color: '红', magic: '星辰', text: '熵力化身不能正面打出或付费现身。' };
const card: Card = { instanceId: 'entropy-a', cardId: 'XQ19', name: '熵力化身', kind: 'character', owner: 'p0', controller: 'p0', faceDown: false, exhausted: false, costAppearance: 'dash' };
describe('printed dash stays distinct from numeric comparison zero', () => {
  it('shows a dash for an authorized known card, never free printed cost0', () => {
    render(<CardContent card={card} definition={definition} viewerId="p0" />);
    expect(screen.getByLabelText('印刷费用 —').textContent).toBe('—');
    expect(screen.queryByLabelText('费用 0')).toBeNull();
    expect(screen.getByTitle('印刷费用为横杠；比较费用时按0计算')).toBeTruthy();
  });
  it('retains dash when a view is rendered without its catalog definition', () => {
    render(<CardContent card={{ ...card, effectiveCost: 0 }} viewerId="p0" />);
    expect(screen.getByLabelText('印刷费用 —').textContent).toBe('—');
    expect(screen.queryByText(/当前费用 0/)).toBeNull();
  });
  it('concealed opponent projection removes even an injected dash hint', () => {
    const hidden = visibleCard({ ...card, faceDown: true }, 'p1');
    expect(hidden.cardId).toBeUndefined(); expect(hidden.costAppearance).toBeUndefined();
    render(<CardContent card={{ ...card, faceDown: true }} definition={definition} viewerId="p1" />);
    expect(screen.queryByLabelText('印刷费用 —')).toBeNull();
  });
  it('keeps ordinary printed0 distinct from a dash', () => {
    render(<CardContent card={{ ...card, name: '无知路人', cardId: 'JC125', costAppearance: undefined, cost: 0 }} definition={{ ...definition, id: 'JC125', name: '无知路人', costAppearance: undefined }} viewerId="p0" />);
    expect(screen.getByLabelText('费用 0').textContent).toBe('0');
    expect(screen.queryByLabelText('印刷费用 —')).toBeNull();
  });
});
