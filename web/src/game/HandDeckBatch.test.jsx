import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
for (const [id, sha, fields] of [
  ['JC130', 'be7d3ec07aea099829f0b01ea506999810091c7b7a202a6bfb0f8ec32eb3055f', { name: '深入研究', cost: 2, loyalty: ['白色'], color: '中立', kind: 'spell', magic: '', subtypes: ['事务', '研究'] }],
  ['JC131', '78b1cff75c50f96763ae2cfeb346524e9046f2e3a445a5e00268113365bc976f', { name: '空投物资', cost: 3, loyalty: ['绿色', '绿色'], color: '中立', kind: 'spell', magic: '', subtypes: ['事务', '策略'] }],
  ['JC096', '28c1eb001f27dc220b8905ba97d7d396680adda8f7c1b8c9e0f2ec97f4907c7c', { name: '阿格里帕之犬', subtitle: '墨菲斯托的化身', cost: 4, loyalty: ['黑色', '黑色'], magic: '鲜血', kind: 'character', unique: true, defense: 2, permanentIcons: { investigation: 0, combat: 2, influence: 1 }, temporaryIcons: { investigation: 1, combat: 0, influence: 0 } }],
  ['XQ17', 'e5ad8fe27074632eab8d6207eb2e6f08efb69d20f343f2d50701d30f5a5bb5eb', { name: '祭品', cost: 1, loyalty: ['红色'], magic: '', kind: 'character', subtypes: ['人类'], defense: 1, temporaryIcons: { investigation: 0, combat: 0, influence: 1 } }],
]) it(`reads the unchanged original ${id} with reviewed printed fields`, () => {
  const d = catalog.cards.find(c => c.id === id);
  expect(d).toMatchObject(fields);
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  const mounted = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0', exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`);
  mounted.unmount();
});
it('exposes the admitted play and trigger summaries without adding an unprinted cost', () => {
  expect(catalog.cards.find(c => c.id === 'JC130').abilities).toMatchObject([{ key: 'research', timing: 'standard', costs: [], triggered: false }]);
  expect(catalog.cards.find(c => c.id === 'JC131').abilities).toMatchObject([{ key: 'airdrop', timing: 'standard', costs: [], triggered: false }]);
  expect(catalog.cards.find(c => c.id === 'JC096').abilities).toMatchObject([{ key: 'sacrifice-draw', timing: 'fast', costs: [{ Assets: 1 }, 'ExhaustSource', 'SacrificeSelectedControlledCharacter'], triggered: false }]);
  expect(catalog.cards.find(c => c.id === 'XQ17').abilities).toMatchObject([{ key: 'offering-death', timing: 'fast', triggered: true }]);
});
