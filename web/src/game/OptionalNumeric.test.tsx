import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CardContent, CardTile } from './CardTile';
import { ChoicePanel } from './ChoicePanel';
import { ReadModal } from './ReadModal';
import { testCard, testCatalog, testChoice } from './testFixtures';

// JSON null is a real catalog/projection shape, even where older TS types are optional.
const event = JSON.parse(JSON.stringify({ ...testCard, cardId: 'JC006', name: '逆转异界之门', kind: 'spell',
  defense: null, damage: null, wounds: null, shield: null, cost: 2, effectiveCost: null }));
const definition = JSON.parse(JSON.stringify({ ...testCatalog.cards[0], id: 'JC006', name: event.name,
  kind: 'spell', cost: 2, defense: null, points: null, threshold: null }));

describe('optional numeric card presentation', () => {
  it.each(['hand', 'mulligan', 'reader'])('omits absent defense on %s without inventing zero', surface => {
    const content = surface === 'hand' ? <CardTile card={event} definition={definition} compact viewerId="p0" />
      : surface === 'reader' ? <ReadModal card={event} definition={definition} viewerId="p0" onClose={vi.fn()} />
      : <ChoicePanel choice={{ ...testChoice, kind: 'mulligan', options: [{ id: event.instanceId, label: event.name, card: event }] }} definitions={new Map([[definition.id, definition]])} busy={false} onSubmit={vi.fn()} viewerId="p0" />;
    const { container } = render(content);
    expect(screen.queryByText(/^防御 |^当前防御 /)).not.toBeInTheDocument();
    expect(container.textContent).not.toMatch(/null|undefined/);
    expect(screen.getByLabelText('费用 2')).toHaveTextContent('2');
    expect(container.querySelector('.hg-damage-marker,.hg-wound-marker,.hg-shield-marker')).toBeNull();
  });

  it.each([
    { current: undefined, printed: undefined, expected: null },
    { current: undefined, printed: null, expected: null },
    { current: null, printed: null, expected: null },
    { current: null, printed: 2, expected: '防御 2' },
    { current: 2, printed: null, expected: '防御 2' },
    { current: 0, printed: null, expected: '防御 0' },
    { current: undefined, printed: 0, expected: '防御 0' },
    { current: 0, printed: 0, expected: '当前防御 0 · 印刷防御 0' },
  ])('preserves numeric defense with current=$current and printed=$printed', ({ current, printed, expected }) => {
    const card = JSON.parse(JSON.stringify({ ...testCard, region: 0, defense: current }));
    const printedCard = JSON.parse(JSON.stringify({ ...testCatalog.cards[0], defense: printed }));
    const { container } = render(<CardContent card={card} definition={printedCard} />);
    if (expected) expect(screen.getByText(expected)).toBeInTheDocument();
    else expect(screen.queryByText(/^防御 |^当前防御 /)).not.toBeInTheDocument();
    expect(container.textContent).not.toMatch(/null|undefined/);
  });

  it('keeps unknown world values distinct from valid zero values', () => {
    const world = { ...event, kind: 'region' };
    const { rerender } = render(<CardContent card={world} definition={definition} />);
    expect(screen.getByText('赢得 — 分 · 控制阈值 —')).toBeInTheDocument();
    rerender(<CardContent card={world} definition={{ ...definition, points: 0, threshold: 0 }} />);
    expect(screen.getByText('赢得 0 分 · 控制阈值 0')).toBeInTheDocument();
  });

  it('does not print null optional society values or turn them into zero', () => {
    const society = { ...event, kind: 'society', name: '秘社' };
    const { container } = render(<CardContent card={society} definition={{ ...definition, startingHand: null, printedCost: null }} />);
    expect(container.textContent).not.toMatch(/null|undefined|防御|起手/);
    expect(container.querySelector('.hg-cost')).toBeNull();
  });
});
