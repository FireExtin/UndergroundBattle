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
for (const [id, sha, fields] of [
  ['JC116', '5a314d56e3d6c660f381e45710dbf0964cf8560cfa170906b3d4e938abb90e16', { cost: 1, loyalty: [], magic: '', subtypes: ['装备', '武器'], ruleTraits: { retreat: true } }],
  ['JC020', '2edb40dfa18d2df22b5b397957d0a03642ade3cee1a10f965bb5e55b0c4f7223', { cost: 2, loyalty: ['绿色'], magic: '', subtypes: ['装备', '武器'] }],
  ['JC093', 'ac3398f82103f6d50eb97b6bb3467244834c8e3867fb92c295d3c420b522bdd2', { cost: 1, loyalty: ['黑色', '黑色'], magic: '死亡', subtypes: ['装备', '武器'] }],
  ['XQ07', '944a254795fc17e946f7150de9a47649d8c6efa3006770850447503b06a22df9', { cost: 1, loyalty: ['绿色'], magic: '神圣', subtypes: ['装备', '物品'] }],
]) it(`preserves original ${id} scan and complete printed interpretation`, () => {
  const d = catalog.cards.find(c => c.id === id);
  expect(d).toMatchObject({ ...fields, kind: 'attachment' });
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  if (id === 'JC020') expect(d.text).toContain('结附于目标本方角色');
  if (id === 'JC093') expect(d.text).toContain('该能力每回合至多只能发动两次');
  if (id === 'XQ07') expect(d.text).toContain('其印刷防御力成为1直到回合结束');
  const rendered = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0', exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`);
  rendered.unmount();
});
it('renders overridden printed base alongside final defense and immutable original for all four seats, then clears it', () => {
  const d = catalog.cards.find(c => c.id === 'LC01');
  const c = { ...d, cardId: d.id, instanceId: 'water-affected', owner: 'p2', controller: 'p0', kind: 'character', region: 0,
    exhausted: false, faceDown: false, currentPrintedDefense: 1, defense: 3 };
  const rendered = render(<CardTile card={c} definition={d} viewerId="p0" />);
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    rendered.rerender(<CardTile card={c} definition={d} viewerId={viewer} />);
    expect(screen.getByText('本回合印刷防御 1')).toBeInTheDocument();
    expect(screen.getByText('当前防御 3 · 原始印刷防御 4')).toBeInTheDocument();
  }
  rendered.rerender(<CardTile card={{ ...c, currentPrintedDefense: undefined, defense: 4 }} definition={d} viewerId="p0" />);
  expect(screen.queryByText('本回合印刷防御 1')).not.toBeInTheDocument();
  rendered.unmount();
});
it('conceals a stale printed-defense grant from unauthorized hidden projections and all hidden card footers', () => {
  const d = catalog.cards.find(c => c.id === 'LC01');
  const c = { ...d, cardId: d.id, instanceId: 'hidden-water', owner: 'p2', controller: 'p0', region: 0,
    exhausted: false, faceDown: true, currentPrintedDefense: 1, defense: 1, kind: 'hidden' };
  const rendered = render(<CardTile card={c} definition={d} viewerId="p0" />);
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    if (viewer !== 'p0') expect(visibleCard(c, viewer).currentPrintedDefense).toBeUndefined();
    rendered.rerender(<CardTile card={c} definition={d} viewerId={viewer} />);
    expect(screen.queryByText('本回合印刷防御 1')).not.toBeInTheDocument();
  }
  rendered.unmount();
});
