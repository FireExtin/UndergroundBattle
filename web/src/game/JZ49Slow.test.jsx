// Native rule fixtures rendered through the production UI; not a natural match.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { kernel } from './testKernel';
import { ChoicePanel } from './ChoicePanel';
import { Table } from './Table';
import { createDeckDraft, validateDeckDraft } from './deckLibrary';

const catalog = JSON.parse(kernel.catalog());
const definitions = new Map(catalog.cards.map(c => [c.id, c]));
const fixtures = JSON.parse(readFileSync(resolve('src/game/jz49Test.fixture.json'), 'utf8')).fixtures;
const view = (kind, seat = 0) => fixtures.find(f => f.kind === kind).views[seat];
afterEach(cleanup);

it('checks the complete JZ49 printed definition', () => {
  expect(definitions.get('JZ49')).toMatchObject({ name: '蹒跚行尸', cost: 1,
    color: '黑', loyalty: ['黑色', '黑色'], magicIcon: 'Death', keywords: ['迟缓'],
    ruleTraits: { slow: true }, deckCopyLimit: 3,
    abilities: [{ key: 'mill-two-entry', triggered: true, timing: 'fast' }] });
});

it('checks recorded declaration, stack, composition and completion results', () => {
  expect(view('response-composing', 2).responseWindow.myIntentId).toBe('jz49-destroy-response');
  expect(view('final').stack).toHaveLength(0);
  expect(view('final').graveyard.some(c => c.cardId === 'JZ49')).toBe(true);
});

it.each([0, 1, 2, 3])('selects the actual optional player target p%s and submits one native choose action', seat => {
  const v = view('entry-declaration');
  const action = v.legalActions.find(a => a.kind === 'choose');
  const submit = vi.fn();
  expect(v.pendingChoice.options.map(o => o.id)).toEqual(['p0', 'p1', 'p2', 'p3']);
  expect(view('entry-declaration', 2).pendingChoice).toBeNull();
  render(<ChoicePanel choice={v.pendingChoice} action={action} definitions={definitions}
    busy={false} onSubmit={submit} viewerId="p0" />);
  fireEvent.click(screen.getByRole('button', { name: `+P${seat}`, exact: true }));
  fireEvent.click(screen.getByRole('button', { name: '确认选择' }));
  expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: v.pendingChoice.id, selected: [`p${seat}`] });
});

it('can decline the optional trigger without changing the mandatory exhausted state', () => {
  const v = view('entry-declaration');
  const action = v.legalActions.find(a => a.kind === 'choose');
  const submit = vi.fn();
  render(<ChoicePanel choice={v.pendingChoice} action={action} definitions={definitions}
    busy={false} onSubmit={submit} viewerId="p0" />);
  fireEvent.click(screen.getByRole('button', { name: '跳过此选择' }));
  expect(submit).toHaveBeenCalledExactlyOnceWith({ ...action, choiceId: v.pendingChoice.id, selected: [] });
});

it('shows entry exhaustion and keeps enemy hidden identity private', () => {
  const v = view('mill-stack');
  const submit = vi.fn();
  const m = render(<Table view={v} catalog={catalog} busy={false} onAction={submit} />);
  const c = v.regions.flatMap(r => r.characters).find(c => c.cardId === 'JZ49');
  expect(m.container.querySelector(`[data-card-instance="${c.instanceId}"]`)).toHaveAttribute('data-card-exhausted', 'true');
  expect(screen.getByRole('button', { name: '查看蹒跚行尸，已横置', exact: true })).toBeInTheDocument();
  m.rerender(<Table view={view('hidden-deployment', 2)} catalog={catalog} busy={false} onAction={submit} />);
  expect(screen.queryByText('蹒跚行尸')).not.toBeInTheDocument();
  expect(view('hidden-deployment', 2).regions.flatMap(r => r.characters).every(c => c.cardId !== 'JZ49')).toBe(true);
});

it('builds an actual fifty-card draft and rejects four JZ49 copies', () => {
  const draft = { ...createDeckDraft(catalog), name: '迟缓置墓测试', cards: [{ cardId: 'JZ49', count: 3 }, { cardId: 'JC125', count: 47 }] };
  expect(validateDeckDraft(draft, catalog).valid).toBe(true);
  expect(JSON.parse(kernel.newGameWithDeck('jz49-ui-deck', 'LOCAL', 'teams', 'P0', JSON.stringify(draft), '9')).view.status).toBe('lobby');
  const invalid = { ...draft, cards: [{ cardId: 'JZ49', count: 4 }, { cardId: 'JC125', count: 46 }] };
  expect(validateDeckDraft(invalid, catalog).valid).toBe(false);
  expect(() => kernel.newGameWithDeck('jz49-ui-invalid', 'LOCAL', 'teams', 'P0', JSON.stringify(invalid), '9')).toThrow();
});
