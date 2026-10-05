import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, validateDeckDraft, saveDeckLibrary, readDeckLibrary, DECK_LIBRARY_STORAGE_KEY } from './deckLibrary';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
function draft(other) {
  const black = ['JC086', 'JC092', 'JC091', 'JC084'];
  const cards = catalog.cards.filter(c => c.color === '红' || c.color === other && (other === '蓝' || black.includes(c.id))).map(c => ({ cardId: c.id, count: 3 }));
  cards.push({ cardId: 'JC125', count: 50 - cards.reduce((n, c) => n + c.count, 0) });
  return { ...createDeckDraft(catalog), name: `红${other}混搭（合法中立补足）`, societyId: null, cards };
}
const counts = d => Object.fromEntries(['红', '黑', '蓝', '中立'].map(color => [color, d.cards.reduce((n, e) => n + (catalog.cards.find(c => c.id === e.cardId).color === color ? e.count : 0), 0)]));
beforeEach(() => localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY));
afterEach(() => localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY));

it('reports actual existing red/black and black/blue presets', () => {
  expect(counts(catalog.decks.find(d => d.id === 'responders'))).toEqual({ 红: 5, 黑: 4, 蓝: 0, 中立: 41 });
  expect(counts(catalog.decks.find(d => d.id === 'reclaimers'))).toEqual({ 红: 0, 黑: 15, 蓝: 3, 中立: 32 });
});
it.each(['黑', '蓝'])('validates, saves, reloads and joins the real red/%s 50-card deck', other => {
  const d = draft(other);
  expect(counts(d)).toEqual(other === '黑' ? { 红: 12, 黑: 12, 蓝: 0, 中立: 26 } : { 红: 12, 黑: 0, 蓝: 27, 中立: 11 });
  expect(validateDeckDraft(d, catalog)).toEqual({ valid: true, total: 50, issues: [] });
  const created = JSON.parse(kernel.newGameWithDeck('local-mixed-deck-ui', 'QA', 'teams', 'P0', JSON.stringify(d), '40'));
  expect(created.view.yourDeck.cards).toEqual([...d.cards].sort((a, b) => a.cardId.localeCompare(b.cardId)));
  saveDeckLibrary([d]);
  const select = vi.fn();
  const m = render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
  expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeEnabled();
  fireEvent.click(screen.getByRole('button', { name: '保存并选择此牌组' }));
  expect(select).toHaveBeenCalledWith(expect.objectContaining({ name: d.name, societyId: null, cards: d.cards }));
  m.unmount();
  expect(readDeckLibrary().drafts[0].cards).toEqual(d.cards);
});
it.each(['黑', '蓝'])('rejects four copies of the shared red character in red/%s', other => {
  const d = draft(other);
  d.cards = d.cards.map(e => ({ ...e, count: e.cardId === 'XQ17' ? 4 : e.cardId === 'JC125' ? e.count - 1 : e.count }));
  expect(validateDeckDraft(d, catalog).issues.some(i => i.code === 'copies')).toBe(true);
  expect(() => kernel.newGameWithDeck('local-overcopy', 'QA', 'teams', 'P0', JSON.stringify(d), '40')).toThrow(/祭品.*4张.*最多3张/);
  saveDeckLibrary([d]);
  const m = render(<DeckLibrary catalog={catalog} onSelectDraft={vi.fn()} />);
  expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
  m.unmount();
});
