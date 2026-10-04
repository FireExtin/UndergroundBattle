import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { CardTile, visibleCard } from './CardTile';
import { ReadModal } from './ReadModal';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const blank = catalog.cards.find(c => c.id === 'JC125');
const card = { ...blank, cardId: blank.id, instanceId: 'equipped', owner: 'p2', controller: 'p0', region: 2,
  exhausted: false, faceDown: false, currentBarrier: true, defense: 2, icons: { investigation: 1, combat: 0, influence: 0 } };
it('shows authoritative host barrier, temporary investigation and defense to four seats, then removes the lost grant', () => {
  const rendered = render(<CardTile card={card} definition={blank} viewerId="p0" />);
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    rendered.rerender(<CardTile card={card} definition={blank} viewerId={viewer} />);
    expect(screen.getByText('屏障')).toBeInTheDocument();
    expect(screen.getByLabelText('当前有效图标：调查1，战斗0，势力0')).toBeInTheDocument();
    expect(screen.getByText('当前防御 2 · 印刷防御 1')).toBeInTheDocument();
  }
  rendered.rerender(<CardTile card={{ ...card, currentBarrier: undefined, defense: 1,
    icons: { investigation: 0, combat: 0, influence: 0 } }} definition={blank} viewerId="p0" />);
  expect(screen.queryByText('屏障')).not.toBeInTheDocument();
  rendered.unmount();
});
it('concealment hides stale host barrier from all seats and print from every unauthorized seat', () => {
  const hidden = { ...card, faceDown: true, kind: 'hidden' };
  const rendered = render(<CardTile card={hidden} definition={blank} viewerId="p0" />);
  expect(screen.queryByText('屏障')).not.toBeInTheDocument();
  for (const viewer of ['p1', 'p2', 'p3']) {
    expect(visibleCard(hidden, viewer).currentBarrier).toBeUndefined();
    rendered.rerender(<CardTile card={hidden} definition={blank} viewerId={viewer} />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    expect(screen.queryByText('屏障')).not.toBeInTheDocument();
    expect(screen.queryByText(blank.name)).not.toBeInTheDocument();
  }
  rendered.unmount();
});
for (const [id, sha, fields] of [
  ['JC071', '85fde88b2a35d595f166d87c0ab27b244198c3919c1f6a381e54e07598f73b67', { cost: 2, loyalty: ['白色'], defense: 1, magic: '星辰', permanentIcons: { investigation: 0, combat: 0, influence: 1 } }],
  ['JC073', 'cebf528de52de72d7400d4c769c2642063342d75a01670e120b5bc71aba23d42', { cost: 3, loyalty: ['白色'], kind: 'attachment', magic: '星辰', subtypes: ['装备', '护身符'] }],
  ['JC102', '842fb9f07e9b8afbc0cecd6c224972c5643bad9b710daa518faceb59fc9a3b5d', { cost: 2, loyalty: [], magic: '鲜血', subtypes: ['法术', '阴'], abilities: [{ key: 'damage-character', timing: 'actionFast' }] }],
  ['JC132', '219bb4062c3e9053b9fc9d309ad919ebe4b6a98c3af314c6987ca948b3eeb4f1', { cost: 4, color: '中立', loyalty: ['黄色'], magic: '心灵', subtypes: ['法术', '空间'] }],
]) it(`preserves ${id} original bytes, corrected loyalty and readable printed text`, () => {
  const d = catalog.cards.find(c => c.id === id);
  expect(d).toMatchObject(fields);
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  const rendered = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0',
    exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`);
  rendered.unmount();
});
