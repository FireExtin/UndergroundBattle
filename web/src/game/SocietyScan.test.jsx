import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const rawCatalog = JSON.parse(kernel.catalog());
const catalog = rawCatalog;
const society = catalog.societies[0];
const sha256 = '41e478a3f38ead83477498fd88131831a5fd843be364c1c0daa3ba10153c2302';

it('keeps the sole real society unchanged while the local JC008 candidate adds one ordinary card', () => {
  expect(catalog.engineVersion).toBe('rust-v0.2.26-msjc07-resource-policy-candidate');
  expect(catalog.cards).toHaveLength(87);
  expect(catalog.societies.map(card => card.id)).toEqual(['MSJC09', 'MSJC01', 'MSJC07']);
  expect(rawCatalog.societies[0].unique).toBe(true);
  const scans = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8'));
  expect(Object.keys(scans)).toHaveLength(90);
  expect(scans.MSJC09).toEqual({ url: '/cards/MSJC09.jpg', source: 'resource/ymsj-fun.github.io/cards/MSJC09 秘社.jpg', sha256 });
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/MSJC09.jpg'))).digest('hex')).toBe(sha256);
  expect(Object.keys(scans).some(id => id.startsWith('FIXTURE'))).toBe(false);
});

it('reads the actual four-seat public society projection and opens its original without character costs or stats', () => {
  const deck = societyId => JSON.stringify({
    id: 'society-scan-local', name: '原图阅读回归', description: '', societyId,
    cards: catalog.decks.find(deck => deck.id === 'watchers').cards,
    rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion,
    engineVersion: catalog.engineVersion, updatedAt: '',
  });
  let room = JSON.parse(kernel.newGameWithDeck('society-scan-local', 'local', 'teams', '甲', deck('MSJC09'), '18446744073709551615'));
  for (const name of ['乙', '丙', '丁']) room = JSON.parse(kernel.joinGameWithDeck(room.state, name, deck(null)));
  const action = (seat, kind) => {
    room = JSON.parse(kernel.applyRoom(room.state, seat, JSON.stringify({
      commandId: 'scan-' + kind + '-' + seat, expectedVersion: room.version,
      action: { kind: 'game', action: { kind } },
    }), '1000'));
  };
  for (let seat = 0; seat < 4; seat++) action(seat, 'ready');
  action(0, 'start');
  for (let seat = 0; seat < 4; seat++) {
    const view = JSON.parse(kernel.view(room.state, seat));
    const card = view.societyZones[0].card;
    expect(card.cardId).toBe('MSJC09');
    expect(card.faceDown).toBe(false);
    const reader = render(<ReadModal card={card} definition={society} viewerId={'p' + seat} onClose={vi.fn()} />);
    const dialog = screen.getByRole('dialog', { name: '放大阅读秘社' });
    expect(dialog).toHaveTextContent('未知的聚会');
    expect(dialog).toHaveTextContent('起手 6 张');
    expect(dialog.querySelector('.hg-cost, .hg-icons')).toBeNull();
    expect(dialog).not.toHaveTextContent('防御');
    expect(dialog).not.toHaveTextContent('白底图标');
    fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
    expect(screen.getByRole('img', { name: '秘社原始牌面' })).toHaveAttribute('src', '/cards/MSJC09.jpg');
    expect(screen.getByRole('link', { name: '打开秘社原始牌面全图' })).toHaveAttribute('href', '/cards/MSJC09.jpg');
    reader.unmount();
  }
});

it('maps the complete JC004 definition to the exact reviewed original and reader metadata', () => {
  const definition = catalog.cards.find(card => card.id === 'JC004');
  const scans = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8'));
  expect(definition).toMatchObject({ name: '力场法师', cost: 4, loyalty: ['黄色', '黄色'], magic: '心灵',
    permanentIcons: { investigation: 1, combat: 0, influence: 1 }, temporaryIcons: { investigation: 0, combat: 1, influence: 0 }, defense: 2 });
  expect(scans.JC004.sha256).toBe('d9209031b83d49c2eacdbedd2967384852acc51329093f362073c462dabf92c4');
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC004.jpg'))).digest('hex')).toBe(scans.JC004.sha256);
  const draft = { id: 'jc004-reader', name: 'JC004原图回归', description: '', societyId: null,
    cards: [{ cardId: 'JC004', count: 3 }, { cardId: 'JC125', count: 47 }], rulesVersion: catalog.rulesVersion,
    cardPoolVersion: catalog.cardPoolVersion, engineVersion: catalog.engineVersion, updatedAt: '' };
  let room = JSON.parse(kernel.newGameWithDeck('jc004-reader', 'LOCAL', 'duel', 'P0', JSON.stringify(draft), '1'));
  room = JSON.parse(kernel.joinGameWithDeck(room.state, 'P1', JSON.stringify(draft)));
  for (const [seat, kind] of [[0, 'ready'], [1, 'ready'], [0, 'start']]) room = JSON.parse(kernel.applyRoom(room.state, seat,
    JSON.stringify({ commandId: `reader-${seat}-${kind}`, expectedVersion: room.version, action: { kind: 'game', action: { kind } } }), '1000'));
  const card = JSON.parse(kernel.view(room.state, 0)).hand.find(card => card.cardId === 'JC004');
  expect(card).toMatchObject({ owner: 'p0', controller: 'p0', faceDown: false, cost: 4, defense: 2 });
  const reader = render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()} />);
  const dialog = screen.getByRole('dialog', { name: '放大阅读力场法师' });
  expect(dialog).toHaveTextContent('本地区的目标角色');
  expect(dialog).not.toHaveTextContent('或暗藏者');
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: '力场法师原始牌面' })).toHaveAttribute('src', '/cards/JC004.jpg');
  reader.unmount();
});
