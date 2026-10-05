import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { CardTile } from './CardTile';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
for (const [id, sha, fields] of [
  ['JZ74', '3915ef2565eff330862507076c052942322e59f47039f31f7c0ebe658181f085', { name: '意外事故', cost: 1, loyalty: [], subtypes: ['突发状况'], abilities: [{ key: 'repress-draw', timing: 'fast', costs: [], triggered: false }] }],
  ['JC126', '33ed0032c1e6983f25be998c66891f137f2f0975031493883f6cc1a9bb5e40e0', { name: '产业扩张', cost: 3, loyalty: ['蓝色'], subtypes: ['策略'], abilities: [{ key: 'expand-assets', timing: 'standard', costs: [], triggered: false }] }],
]) it(`reads ${id} original and compiled whole-card fields`, () => {
  const d = catalog.cards.find(c => c.id === id);
  expect(d).toMatchObject({ ...fields, kind: 'spell', color: '中立', magic: '', defense: null, unique: false, keywords: [], permanentIcons: { investigation: 0, combat: 0, influence: 0 }, temporaryIcons: { investigation: 0, combat: 0, influence: 0 } });
  expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
  const m = render(<ReadModal card={{ ...d, cardId: id, instanceId: id, owner: 'p0', controller: 'p0', exhausted: false, faceDown: false }} definition={d} viewerId="p0" onClose={vi.fn()} />);
  if (id === 'JC126') expect(screen.getByText(/忠诚.*蓝色/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: `${d.name}原始牌面` })).toHaveAttribute('src', `/cards/${id}.jpg`);
  m.unmount();
});

it.each(['p0', 'p1', 'p2', 'p3'])('reads an expanded public asset without restoring printed character rules for %s', viewer => {
  // This is the server asset projection, whose identity is no longer a character.
  const d = catalog.cards.find(c => c.id === 'JZ58');
  const asset = { instanceId: 'expanded-asset', kind: 'asset', name: '资产', owner: 'p2', controller: 'p2', faceDown: false, exhausted: false, color: '紫', magic: '星辰' };
  const m = render(<CardTile card={asset} definition={d} viewerId={viewer} />);
  expect(screen.getByRole('button', { name: '查看资产' })).toBeInTheDocument();
  expect(screen.getByText('紫 · 星辰')).toBeInTheDocument();
  expect(screen.getByText('未横置时可提供 1 费用，并提供所示派系与魔法忠诚。')).toBeInTheDocument();
  expect(screen.queryByText('噩梦残像')).not.toBeInTheDocument();
  expect(screen.queryByText(/灵体：/)).not.toBeInTheDocument();
  expect(screen.queryByLabelText(/费用/)).not.toBeInTheDocument();
  m.rerender(<CardTile card={{ ...asset, exhausted: true }} definition={d} viewerId={viewer} />);
  expect(screen.getByRole('button', { name: '查看资产，已横置' })).toBeInTheDocument();
  expect(screen.getByText('紫 · 星辰')).toBeInTheDocument();
  m.unmount();
});
