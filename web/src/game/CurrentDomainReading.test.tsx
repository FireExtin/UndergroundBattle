import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CardContent, visibleCard } from './CardTile';
import { ChoicePanel } from './ChoicePanel';
import { ReadModal } from './ReadModal';
import native from './currentDomainsSelection.fixture.json';
import { testChoice } from './testFixtures';
import type { Card, CardDefinition } from './types';

// Exact Native76 card excerpts; these checks render UI, not engine replays or live games.
const definitions = new Map(native.definitions.map(card => [card.id, card as unknown as CardDefinition]));
const cards = Object.fromEntries(Object.entries(native.cases).map(([key, value]) => [key, value.card])) as Record<keyof typeof native.cases, Card>;
const viewerId = native.cases.two.viewerId;
const content = (card: Card, compact = false) => <CardContent card={card} definition={definitions.get(card.cardId!)} viewerId={viewerId} compact={compact} />;

describe('current domain reading', () => {
  it('reads each server count directly, including exhaustion, decreases and removal', () => {
    const reader = (card: Card) => <ReadModal card={card} definition={definitions.get(card.cardId!)} viewerId={viewerId} onClose={vi.fn()} />;
    const { rerender } = render(reader(cards.two));
    expect(screen.getByRole('group', { name: '领域对照' })).toHaveTextContent('当前领域心灵 ×2印刷领域：无');
    rerender(reader(cards.twoExhausted));
    expect(screen.getByRole('group', { name: '领域对照' })).toHaveTextContent('心灵 ×2');
    expect(screen.getByText('已横置')).toBeInTheDocument();
    rerender(reader(cards.one));
    expect(screen.getByRole('group', { name: '领域对照' })).toHaveTextContent('心灵 ×1');
    expect(screen.queryByText('心灵 ×2')).not.toBeInTheDocument();
    rerender(reader(cards.granted));
    expect(screen.getByRole('group', { name: '领域对照' })).toHaveTextContent('心灵 ×1');
    rerender(reader(cards.removed));
    expect(screen.queryByRole('group', { name: '领域对照' })).not.toBeInTheDocument();
    expect(screen.queryByText(/心灵 ×/)).not.toBeInTheDocument();
  });

  it('retains printed domains when the optional current field is absent', () => {
    const { container } = render(content(cards.printed));
    expect(cards.printed.currentDomains).toBeUndefined();
    expect(container.querySelector('.hg-card-affiliation')).toHaveTextContent('心灵');
    expect(screen.queryByRole('group', { name: '领域对照' })).not.toBeInTheDocument();
  });

  it.each([
    { label: 'compact', card: cards.two, compact: true },
    { label: 'hand', card: { ...cards.two, region: undefined } },
    { label: 'asset', card: { ...cards.two, kind: 'asset' } },
    { label: 'sealed', card: { ...cards.two, kind: 'sealed' } },
    { label: 'own concealed', card: { ...cards.two, faceDown: true, controller: viewerId } },
    { label: 'other concealed', card: { ...cards.two, faceDown: true, controller: 'other-player' } },
  ])('does not present stale current domains in $label cards', ({ card, compact }) => {
    render(content(card, compact));
    expect(screen.queryByRole('group', { name: '领域对照' })).not.toBeInTheDocument();
    expect(screen.queryByText(/心灵 ×/)).not.toBeInTheDocument();
    if (card.faceDown) expect(visibleCard(card, viewerId).currentDomains).toBeUndefined();
  });

  it('refreshes a selected candidate without changing its identity or using reading as selection', () => {
    const read = vi.fn(), submit = vi.fn();
    const action = { id: 'domain-choice', kind: 'choose', label: '确认选择', choiceId: testChoice.id };
    const panel = (card: Card) => <ChoicePanel choice={{ ...testChoice, options: [{ id: 'candidate-original', label: card.name, card }] }} action={action} definitions={definitions} busy={false} onSubmit={submit} onReadCard={read} viewerId={viewerId} />;
    const { rerender, container } = render(panel(cards.two));
    const option = container.querySelector<HTMLButtonElement>('[data-choice-option="candidate-original"]')!;
    expect(within(option).getByRole('group', { name: '领域对照' })).toHaveTextContent('心灵 ×2');
    fireEvent.click(option);
    rerender(panel(cards.one));
    expect(option).toHaveAttribute('aria-pressed', 'true');
    expect(within(option).getByRole('group', { name: '领域对照' })).toHaveTextContent('心灵 ×1');
    fireEvent.click(screen.getByRole('button', { name: `放大阅读${cards.one.name}` }));
    expect(read).toHaveBeenCalledExactlyOnceWith(cards.one);
    expect(option).toHaveAttribute('aria-pressed', 'true');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, selected: ['candidate-original'] });
  });
});
