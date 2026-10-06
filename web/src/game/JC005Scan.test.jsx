// Historical catalog/scan assertions use frozen32; current36 is covered independently by JZ55UniqueDestroy.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/legacy-v0.2.32/hegemony_wasm.js';
import { ReadModal } from './ReadModal';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/legacy-v0.2.32/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const definition = catalog.cards.find(c => c.id === 'JC005');
const sha256 = '216c26cff74fe738dbc486e725828ddc8077705161b1fee566e9d806684364a1';

it('retains Site24 JC005 while adding the isolated JC008 candidate and retains the exact printed spell metadata and original bytes', () => {
  expect(catalog.engineVersion).toBe('rust-v0.2.32-jc030-blue-bat-candidate');
  expect(catalog.cardPoolVersion).toBe('limited-v2.29-jc030-blue-bat-candidate');
  expect(catalog.cards).toHaveLength(94);
  expect(catalog.cards.some(c => c.id === 'JC008')).toBe(true);
  expect(catalog.societies.map(c => c.id)).toEqual(['MSJC09', 'MSJC01', 'MSJC07', 'MSJC06', 'MSJC08', 'MSJC11', 'MSJC02']);
  expect(definition).toMatchObject({ name: '裂解术', kind: 'spell', type: '法术/空间',
    cost: 2, loyalty: ['黄色'], magic: '心灵', defense: null, unique: false,
    permanentIcons: { investigation: 0, combat: 0, influence: 0 },
    temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, keywords: [] });
  expect(definition.text).toContain('结附于角色的附属');
  expect(definition.text).toContain('资产区具有心灵领域图标');
  expect(definition.text).not.toMatch(/装备|额外费用/);
  const scans = JSON.parse(readFileSync(resolve('src/game/cardScansV032.fixture.json'), 'utf8'));
  expect(Object.keys(scans)).toHaveLength(101);
  expect(scans.JC005).toEqual({ url: '/cards/JC005.jpg', source: 'resource/ymsj-fun.github.io/cards/JC005 裂解术.jpg', sha256 });
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC005.jpg'))).digest('hex')).toBe(sha256);
});

it('reads the actual own-hand WASM spell projection with no invented defense and opens its original', () => {
  const draft = { id: 'jc005-reader', name: 'JC005正式原图回归', description: '', societyId: null,
    cards: [{ cardId: 'JC005', count: 3 }, { cardId: 'JC125', count: 47 }],
    rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion,
    engineVersion: catalog.engineVersion, updatedAt: '' };
  let room = JSON.parse(kernel.newGameWithDeck('jc005-reader', 'LOCAL', 'duel', 'P0', JSON.stringify(draft), '1'));
  room = JSON.parse(kernel.joinGameWithDeck(room.state, 'P1', JSON.stringify(draft)));
  for (const [seat, kind] of [[0, 'ready'], [1, 'ready'], [0, 'start']]) {
    room = JSON.parse(kernel.applyRoom(room.state, seat, JSON.stringify({ commandId: `reader-${seat}-${kind}`,
      expectedVersion: room.version, action: { kind: 'game', action: { kind } } }), '1000'));
  }
  const card = JSON.parse(kernel.view(room.state, 0)).hand.find(c => c.cardId === 'JC005');
  expect(card).toMatchObject({ owner: 'p0', controller: 'p0', kind: 'spell', cost: 2 });
  expect(Object.hasOwn(card, 'defense')).toBe(false);
  const rendered = render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()} />);
  const dialog = screen.getByRole('dialog', { name: '放大阅读裂解术' });
  expect(dialog).toHaveTextContent('法术/空间');
  expect(dialog).toHaveTextContent('资产区具有心灵领域图标');
  expect(dialog).not.toHaveTextContent('防御');
  expect(dialog).not.toHaveTextContent('null');
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: '裂解术原始牌面' })).toHaveAttribute('src', '/cards/JC005.jpg');
  rendered.unmount();
});
