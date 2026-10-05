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
const card = { ...blank, cardId: blank.id, instanceId: 'protected', owner: 'p2', controller: 'p0', region: 2,
  exhausted: false, faceDown: false, currentDamagePrevention: true, defense: 2,
  icons: { investigation: 0, combat: 1, influence: 0 } };
it('renders authoritative turn damage prevention, attached defense and combat to all four seats then clears expired grants', () => {
  const rendered = render(<CardTile card={card} definition={blank} viewerId="p0" />);
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    rendered.rerender(<CardTile card={card} definition={blank} viewerId={viewer} />);
    expect(screen.getByText('本回合防止伤害')).toBeInTheDocument();
    expect(screen.getByText('当前防御 2 · 印刷防御 1')).toBeInTheDocument();
    expect(screen.getByLabelText('当前有效图标：调查0，战斗1，势力0')).toBeInTheDocument();
  }
  rendered.rerender(<CardTile card={{ ...card, currentDamagePrevention: undefined,
    defense: 1, icons: { investigation: 0, combat: 0, influence: 0 } }} definition={blank} viewerId="p0" />);
  expect(screen.queryByText('本回合防止伤害')).not.toBeInTheDocument();
  expect(screen.queryByText('当前防御 2 · 印刷防御 1')).not.toBeInTheDocument();
  rendered.unmount();
});
it('clears stale prevention from hidden unauthorized projections and renders no hidden prevention footer', () => {
  const hidden = { ...card, kind: 'hidden', faceDown: true };
  const rendered = render(<CardTile card={hidden} definition={blank} viewerId="p0" />);
  expect(screen.queryByText('本回合防止伤害')).not.toBeInTheDocument();
  for (const viewer of ['p1', 'p2', 'p3']) {
    expect(visibleCard(hidden, viewer).currentDamagePrevention).toBeUndefined();
    rendered.rerender(<CardTile card={hidden} definition={blank} viewerId={viewer} />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    expect(screen.queryByText(blank.name)).not.toBeInTheDocument();
    expect(screen.queryByText('本回合防止伤害')).not.toBeInTheDocument();
  }
  rendered.unmount();
});
for (const [id, sha, fields] of [
  ['JC078', 'd238d2c7412349579c00fa4a5a662e9214bdad13df25b8917ae1034251f7e2b9', { cost: 1, loyalty: ['白色'], magic: '神圣', subtypes: ['法术', '能量'], abilities: [{ key: 'prevent-turn-damage', timing: 'fast' }] }],
  ['XQ14', '0baea888495adba62a7688be1b95c95b5ffa60d689c104b7ff3a26dd2340ffb2', { cost: 1, loyalty: ['蓝色'], magic: '鲜血', kind: 'attachment', subtypes: ['状态'] }],
  ['XQ47', '9b370fb92ad2e1c3ac6f33d3402ab2c8cbdea1898aae4c4a386f17631d654a03', { cost: 1, loyalty: [], magic: '', kind: 'attachment', subtypes: ['装备', '防具'] }],
  ['JC112', '870b526451ed20441601cfed413a99e2e64a266d60daa698bb2066840383bd71', { cost: 1, loyalty: [], magic: '', defense: 1, permanentIcons: { investigation: 0, combat: 0, influence: 1 }, ruleTraits: { public: true, cannot_be_equipped: true } }],
]) it(`preserves ${id} original image bytes and printed field interpretation`, () => {
  const d = catalog.cards.find(c => c.id === id);
  expect(d).toMatchObject(fields);
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  const rendered = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0',
    exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`);
  rendered.unmount();
});
