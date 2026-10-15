import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChoicePanel } from './ChoicePanel';
import { visibleCard } from './CardTile';
import red from './jc052Native74Current.fixture.json';
import black from './jc087Native74.fixture.json';
import blue from './jc037Native74.fixture.json';
import gray from './jc061Native74.fixture.json';
import type { CardDefinition, View } from './types';

// These checks use immutable Native74 projections. They do not replay the engine or a live room.
const blueLeader = blue.scenarios.find(row => row.name === 'leader-replace')!.steps
  .flatMap(step => step.views).find(view => view.pendingChoice?.kind === 'target');
const cases = [
  { faction: 'red', view: red.cases.find(row => row.name === 'leader-sacrifice')!.beforeViews[0] },
  { faction: 'black', view: black.cases.find(row => row.name === 'move-entry')!.beforeViews[0] },
  { faction: 'blue', view: blueLeader },
  { faction: 'gray', view: gray.cases.find(row => row.name === 'entry-accept')!.beforeViews[0] },
];

describe('priority faction selection projections', () => {
  it.each(cases)('$faction reads the visible projection and directly replaces a single target without changing its ID', row => {
    const view = row.view as unknown as View;
    const choice = view.pendingChoice!;
    const action = view.legalActions.find(item => item.kind === 'choose')!;
    expect(choice.options.length).toBeGreaterThan(1);
    expect(choice.max).toBe(1);
    expect(action).toBeDefined();
    const submit = vi.fn(), read = vi.fn();
    const { container } = render(<ChoicePanel choice={choice} action={action}
      definitions={new Map(blue.catalog.cards.map(card => [card.id, card as CardDefinition]))}
      viewerId={view.you} playerLabels={Object.fromEntries(view.players.map(player => [player.id, player.name]))}
      busy={false} onSubmit={submit} onReadCard={read} />);
    const buttons = Array.from(container.querySelectorAll<HTMLButtonElement>('[data-choice-option]'));
    const first = buttons[0], last = buttons.at(-1)!;
    fireEvent.click(first);
    fireEvent.click(last);
    expect(first).toHaveAttribute('aria-pressed', 'false');
    expect(last).toHaveAttribute('aria-pressed', 'true');
    const firstCard = choice.options[0].card!;
    const reader = within(first.parentElement!).getByRole('button', { name: /^放大阅读/ });
    expect(reader).toHaveAccessibleDescription(new RegExp('目标 1'));
    fireEvent.click(reader);
    expect(read).toHaveBeenCalledExactlyOnceWith(visibleCard(firstCard, view.you));
    expect(last).toHaveAttribute('aria-pressed', 'true');
    expect(submit).not.toHaveBeenCalled();
    if (firstCard.faceDown && firstCard.controller !== view.you) {
      expect(reader).toHaveAccessibleName('放大阅读暗藏者');
      expect(read.mock.calls[0][0].cardId).toBeUndefined();
    }
    fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: choice.id, selected: [choice.options.at(-1)!.id] });
  });
});
