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
const card = { ...blank, cardId: blank.id, instanceId: 'granted', owner: 'p2', controller: 'p0', region: 2, exhausted: false, faceDown: false, currentRenown: true, icons: { investigation: 2, combat: 0, influence: 0 } };
it('shows granted renown and authoritative investigation to all public seats, then removes the expired grant', () => {
  const rendered = render(<CardTile card={card} definition={blank} viewerId="p0" />);
  for (const viewer of ['p0', 'p1', 'p2', 'p3']) {
    rendered.rerender(<CardTile card={card} definition={blank} viewerId={viewer} />);
    expect(screen.getAllByText('声望')).toHaveLength(1);
    expect(screen.getByLabelText('当前有效图标：调查2，战斗0，势力0')).toBeInTheDocument();
  }
  rendered.rerender(<CardTile card={{ ...card, currentRenown: undefined, icons: { investigation: 0, combat: 0, influence: 0 } }} definition={blank} viewerId="p0" />);
  expect(screen.queryByText('声望')).not.toBeInTheDocument();
  rendered.unmount();
});
it('concealed cards clear a stale grant for all seats and redact print for the three unauthorized seats', () => {
  const hidden = { ...card, faceDown: true, kind: 'hidden' };
  const rendered = render(<CardTile card={hidden} definition={blank} viewerId="p0" />);
  expect(screen.queryByText('声望')).not.toBeInTheDocument();
  for (const viewer of ['p1', 'p2', 'p3']) {
    expect(visibleCard(hidden, viewer).currentRenown).toBeUndefined();
    rendered.rerender(<CardTile card={hidden} definition={blank} viewerId={viewer} />);
    expect(screen.getByRole('button', { name: '查看暗藏者' })).toBeInTheDocument();
    expect(screen.queryByText('声望')).not.toBeInTheDocument();
    expect(screen.queryByText(blank.name)).not.toBeInTheDocument();
  }
  rendered.unmount();
});
for (const [id, sha, fields] of [
  ['JC070', '3dd1ef905e2cfb4d806b258fdde0b1ec1fe2adad7868bb5cd7dc0bcf9cfe9e1b', { cost: 1, loyalty: ['白色', '白色'], defense: 1, temporaryIcons: { investigation: 1, combat: 0, influence: 0 }, keywords: ['公开', '声望'] }],
  ['JC076', '4390ebf4eabcf44c541303bfb764061c1622cb483ed0e034ee9651571cee27ac', { cost: 3, loyalty: ['白色'], defense: 1, temporaryIcons: { investigation: 2, combat: 0, influence: 0 }, keywords: ['声望'] }],
  ['JC074', 'e34e5bb081f3ee459d99038c68d60f4be4a44af05baa4289759b730045140914', { cost: 2, loyalty: ['白色'], subtypes: ['法术', '预言'], magic: '星辰' }],
]) it(`keeps all reviewed ${id} original bytes and printed fields readable`, () => {
  const d = catalog.cards.find(c => c.id === id);
  expect(d).toMatchObject(fields);
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  const rendered = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0', exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`);
  rendered.unmount();
});
