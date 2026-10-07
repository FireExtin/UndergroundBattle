// These are persisted native layouts, not a natural browser game.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { kernel } from './testKernel';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, validateDeckDraft } from './deckLibrary';

const catalog = JSON.parse(kernel.catalog());
const definitions = new Map(catalog.cards.map(c => [c.id, c]));
const fixtures = JSON.parse(readFileSync(resolve('src/game/jz48Test.fixture.json'), 'utf8')).fixtures;
const view = (kind, seat = 0) => fixtures.find(f => f.kind === kind).views[seat];
afterEach(cleanup);

it('admits JZ48 alone and preserves the exact published JZ49 catalog', () => {
  expect(definitions.get('JZ48')).toMatchObject({ name: '街头劫匪', cost: 1, loyalty: ['黑色'],
    magic: '', magicIcon: 'None', subtypes: ['人类', '罪犯'], defense: 1, unique: false,
    permanentIcons: { investigation: 0, combat: 1, influence: 0 },
    temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, abilities: [], deckCopyLimit: 3 });
});

it('restores every native seat projection including paid stack, hide declaration and cascade', () => {
  for (const row of fixtures) for (let seat = 0; seat < 4; seat++) {
  }
  expect(view('response-stack').stack).toHaveLength(2);
  expect(view('response-cascade-final').stack).toHaveLength(0);
  expect(view('response-cascade-final').graveyard.filter(c => c.cardId === 'JZ48')).toHaveLength(2);
  expect(view('score-ten-final').status).toBe('finished');
});

it('separates current permanent influence and defense from immutable printed values', () => {
  const v = view('supported-pair');
  const c = v.regions.flatMap(r => r.characters).find(c => c.cardId === 'JZ48');
  expect(c.icons.influence).toBe(1); expect(c.defense).toBe(2);
  render(<ReadModal card={c} definition={definitions.get('JZ48')} viewerId="p0" onClose={vi.fn()} />);
  expect(screen.getByLabelText('当前有效图标：调查0，战斗1，势力1')).toBeInTheDocument();
  expect(screen.getByLabelText('印刷图标：调查0，战斗1，势力0')).toBeInTheDocument();
  expect(screen.getByText('当前防御 2 · 印刷防御 1')).toBeInTheDocument();
});

it('shows exhaustion without erasing the supported current defense', () => {
  const v = view('exhausted-supported');
  const c = v.regions.flatMap(r => r.characters).find(c => c.cardId === 'JZ48' && c.exhausted);
  expect(c.defense).toBe(2);
  const m = render(<Table view={v} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(m.container.querySelector(`[data-card-instance="${c.instanceId}"]`)).toHaveAttribute('data-card-exhausted', 'true');
});

it('keeps hidden JZ48 print and continuous bonus out of opponent readings', () => {
  const own = view('hidden-source');
  const hidden = own.regions.flatMap(r => r.characters).find(c => c.faceDown);
  const enemy = view('hidden-source', 2).regions.flatMap(r => r.characters).find(c => c.instanceId === hidden.instanceId);
  expect(enemy.cardId).toBeUndefined(); expect(enemy.defense).toBeUndefined();
  render(<ReadModal card={hidden} definition={definitions.get('JZ48')} viewerId="p2" onClose={vi.fn()} />);
  expect(screen.queryByText('街头劫匪')).not.toBeInTheDocument();
  expect(screen.queryByLabelText('当前有效图标：调查0，战斗1，势力1')).not.toBeInTheDocument();
});

it('uses current controller for borrowed copies and gives each side its own values', () => {
  const v = view('borrowed-both-sides');
  const copies = v.regions.flatMap(r => r.characters).filter(c => c.cardId === 'JZ48');
  expect(copies).toHaveLength(2);
  expect(copies.find(c => c.controller === 'p0').owner).toBe('p1');
  for (const c of copies) { expect(c.defense).toBe(2); expect(c.icons.influence).toBe(1); }
  expect(new Set(copies.map(c => c.instanceId)).size).toBe(2);
});

it('offers the new gameplay definition in the real deck builder', () => {
  render(<DeckLibrary catalog={catalog} />);
  fireEvent.click(screen.getByRole('button', { name: '新建空白牌组' }));
  fireEvent.change(screen.getByRole('searchbox', { name: '检索卡牌' }), { target: { value: 'JZ48' } });
  expect(screen.getByRole('button', { name: '添加 街头劫匪（JZ48）' })).toBeEnabled();
});

it('validates fifty-card draft and rejects four same-name JZ48 copies', () => {
  const draft = { ...createDeckDraft(catalog), name: '罪犯持续测试', cards: [{ cardId: 'JZ48', count: 3 }, { cardId: 'JC125', count: 47 }] };
  expect(validateDeckDraft(draft, catalog).valid).toBe(true);
  expect(JSON.parse(kernel.newGameWithDeck('jz48-ui-deck', 'LOCAL', 'teams', 'P0', JSON.stringify(draft), '9')).view.status).toBe('lobby');
  const invalid = { ...draft, cards: [{ cardId: 'JZ48', count: 4 }, { cardId: 'JC125', count: 46 }] };
  expect(validateDeckDraft(invalid, catalog).valid).toBe(false);
  expect(() => kernel.newGameWithDeck('jz48-ui-invalid', 'LOCAL', 'teams', 'P0', JSON.stringify(invalid), '9')).toThrow();
});
