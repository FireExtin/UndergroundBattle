import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import * as current from '../../../rust-game-wasm/legacy-v0.2.8/hegemony_wasm.js';
import * as frozen from '../../../rust-game-wasm/legacy-v0.2.7/hegemony_wasm.js';
import { DeckLibrary } from './DeckLibraryPanel';
import { DECK_LIBRARY_STORAGE_KEY, readDeckLibrary } from './deckLibrary';

// Real compiled catalogs: no hand-written supported/copy-limit metadata.
current.initSync({ module: readFileSync(resolve('../rust-game-wasm/legacy-v0.2.8/hegemony_wasm_bg.wasm')) });
frozen.initSync({ module: readFileSync(resolve('../rust-game-wasm/legacy-v0.2.7/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(current.catalog());
const oldCatalog = JSON.parse(frozen.catalog());
const added = ['BQ083', 'JC001', 'JC006', 'JC007', 'JC047', 'JC075', 'JC088', 'JC104', 'XQ16'];
const entered = [...added, 'JC016', 'BQ022'];
afterEach(() => localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY));

it('edits all nine v028 cards through normal controls, saves/reloads, and enters four seats with the selected draft', () => {
  localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY);
  expect(catalog.engineVersion).toBe('rust-v0.2.8');
  expect(catalog.cards).toHaveLength(48);
  expect(catalog.cards.filter(card => !oldCatalog.cards.some(old => old.id === card.id)).map(card => card.id).sort()).toEqual(added);
  const select = vi.fn();
  const editor = render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
  fireEvent.change(screen.getByRole('textbox', { name: '牌组名称' }), { target: { value: 'v028九卡自组' } });
  for (const id of [...entered, 'JC125']) {
    const card = catalog.cards.find(value => value.id === id);
    fireEvent.click(screen.getByRole('button', { name: `添加 ${card.name}（${id}）` }));
    fireEvent.change(screen.getByRole('spinbutton', { name: `${card.name}（${id}）张数` }), { target: { value: id === 'JC125' ? '17' : '3' } });
  }
  const selectButton = () => screen.getByRole('button', { name: '保存并选择此牌组' });
  expect(selectButton()).toBeEnabled();
  const lawyer = catalog.cards.find(card => card.id === 'XQ16');
  const lawyerCount = screen.getByRole('spinbutton', { name: `${lawyer.name}（XQ16）张数` });
  fireEvent.change(lawyerCount, { target: { value: '4' } });
  expect(selectButton()).toBeDisabled();
  fireEvent.change(lawyerCount, { target: { value: '3' } });
  const neutralCount = screen.getByRole('spinbutton', { name: '无知路人（JC125）张数' });
  fireEvent.change(neutralCount, { target: { value: '16' } });
  expect(selectButton()).toBeDisabled();
  fireEvent.change(neutralCount, { target: { value: '17' } });
  fireEvent.click(screen.getByRole('button', { name: '保存牌组' }));
  const saved = readDeckLibrary().drafts[0];
  expect(saved.cards).toHaveLength(12);
  for (const id of entered) expect(saved.cards).toContainEqual({ cardId: id, count: 3 });
  editor.unmount();
  render(<DeckLibrary catalog={catalog} onSelectDraft={select} />);
  expect(screen.getByRole('textbox', { name: '牌组名称' })).toHaveValue('v028九卡自组');
  fireEvent.click(selectButton());
  const draft = select.mock.calls[0][0];
  expect({ ...draft, updatedAt: saved.updatedAt }).toEqual(saved);
  expect(Date.parse(draft.updatedAt)).toBeGreaterThanOrEqual(Date.parse(saved.updatedAt));
  // Local kernel acceptance, distinct from public natural UI play.
  let result = JSON.parse(current.newGameWithDeck('local-nine-card-entry', 'invite', 'teams', 'P0', JSON.stringify(draft), '42'));
  for (let seat = 1; seat < 4; seat++) result = JSON.parse(current.joinGameWithDeck(result.state, `P${seat}`, JSON.stringify(draft)));
  for (let seat = 0; seat < 4; seat++) {
    const before = result.version;
    result = JSON.parse(current.applyRoom(result.state, seat, JSON.stringify({ commandId: `ready-${seat}`, expectedVersion: result.version, action: { kind: 'game', action: { kind: 'ready' } } }), '1000'));
    expect(result.version).toBe(before + 1);
    expect(result.outcome).toBe('accepted');
  }
  result = JSON.parse(current.applyRoom(result.state, 0, JSON.stringify({ commandId: 'start', expectedVersion: result.version, action: { kind: 'game', action: { kind: 'start' } } }), '1000'));
  expect(result.outcome).toBe('accepted');
  for (let guard = 0; guard < 12; guard++) {
    const views = [0, 1, 2, 3].map(seat => JSON.parse(current.view(result.state, seat)));
    const seat = views.findIndex(view => view.pendingChoice);
    if (seat < 0) break;
    result = JSON.parse(current.applyRoom(result.state, seat, JSON.stringify({ commandId: `setup-${guard}`, expectedVersion: result.version, action: { kind: 'game', action: { kind: 'choose', choiceId: views[seat].pendingChoice.id, selected: [] } } }), '1000'));
    expect(result.outcome).toBe('accepted');
  }
  for (let seat = 0; seat < 4; seat++) expect(JSON.parse(current.view(result.state, seat)).pendingChoice).toBeNull();
  const envelope = JSON.parse(result.state);
  expect(envelope.game.players).toHaveLength(4);
  for (const player of envelope.game.players) {
    expect(player.hand.length + player.deck.length).toBe(50);
    for (const id of entered) expect([...player.hand, ...player.deck].filter(card => card.definition === id)).toHaveLength(3);
  }
});

it('keeps the v027 39-card builder frozen and rejects a saved v028 nine-card draft without rewriting provenance', () => {
  const draft = { id: 'nine-saved', name: '保留v028来源', description: '', societyId: null,
    cards: [{ cardId: 'JC125', count: 23 }, ...added.map(cardId => ({ cardId, count: 3 }))],
    rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion, engineVersion: catalog.engineVersion, updatedAt: '2026-10-03T00:00:00Z' };
  localStorage.setItem(DECK_LIBRARY_STORAGE_KEY, JSON.stringify({ version: 1, drafts: [draft] }));
  expect(oldCatalog.engineVersion).toBe('rust-v0.2.7');
  expect(oldCatalog.cards).toHaveLength(39);
  const select = vi.fn();
  render(<DeckLibrary catalog={oldCatalog} onSelectDraft={select} />);
  for (const id of added) expect(screen.queryByRole('button', { name: new RegExp(`^添加 .*（${id}）$`) })).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: '保存并选择此牌组' })).toBeDisabled();
  expect(readDeckLibrary().drafts[0]).toEqual(draft);
  expect(select).not.toHaveBeenCalled();
});
