// Presentation boundary fixtures; rule computation stays in the actual WASM kernel.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { CardContent, visibleCard } from './CardTile';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { testView } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
const definition = catalog.cards.find(c => c.id === 'XQ43');
const spiritDefinition = catalog.cards.find(c => c.id === 'JZ58');
const aura = { instanceId: 'actual-region-aura', cardId: 'XQ43', name: '具现化', kind: 'attachment', hostId: 'region-instance-A', owner: 'p1', controller: 'p0', region: 0, faceDown: false, exhausted: false, cost: 2, text: definition.text };
const spirit = { instanceId: 'actual-spirit', cardId: 'JZ58', name: '噩梦残像', kind: 'character', owner: 'p1', controller: 'p1', region: 0, faceDown: false, exhausted: false, icons: { investigation: 0, combat: 0, influence: 1 }, convertedTemporaryIcons: { investigation: 0, combat: 0, influence: 1 } };
const view = { ...testView, status: 'playing', hand: [], legalActions: [], players: [...testView.players, { ...testView.players[0], id: 'p1', name: '乙', seat: 1, team: 1 }], attachments: [aura], regions: [{ id: aura.hostId, cardId: 'DQJC107', index: 0, name: '地区甲', threshold: 3, points: 3, influence: [0, 0], characters: [spirit] }] };

it('reads the complete real XQ43 program and byte-identical original scan', () => {
  expect(definition).toMatchObject({ name: '具现化', kind: 'attachment', cost: 2, loyalty: ['紫色'], color: '紫', magic: '星辰', subtypes: ['结界'], unique: true, defense: null, permanentIcons: { investigation: 0, combat: 0, influence: 0 }, temporaryIcons: { investigation: 0, combat: 0, influence: 0 }, keywords: [] });
  expect(definition.abilities).toEqual([{ key: 'attach-region', label: '结附地区', timing: 'standard', costs: [], triggered: false }]);
  const scans = JSON.parse(readFileSync(resolve('public/card-scans.json'), 'utf8'));
  const hash = '6e950624afff514a9ffed208fe51dc4137e5114b3f67a24bb0e694be0c935c93';
  expect(scans.XQ43).toEqual({ url: '/cards/XQ43.jpg', source: 'resource/ymsj-fun.github.io/cards/XQ43 具现化.jpg', sha256: hash });
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/XQ43.jpg'))).digest('hex')).toBe(hash);
  const m = render(<ReadModal card={aura} definition={definition} viewerId="p0" onClose={vi.fn()} />);
  expect(screen.getByRole('dialog')).toHaveTextContent('所有临时调查、临时战斗、临时势力');
  fireEvent.click(screen.getByRole('button', { name: '原始牌面' }));
  expect(screen.getByRole('img', { name: '具现化原始牌面' })).toHaveAttribute('src', '/cards/XQ43.jpg'); m.unmount();
});

it('mounts on the exact region instance, reads ownership separately, and exposes region inspector objects', () => {
  const m = render(<Table view={view} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(m.container.querySelector('[data-attachment-host="region-instance-A"]')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: '查看地区附属具现化' }));
  expect(screen.getByText(/乙 拥有 · 甲 操控 · 附着于 地区地区甲/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
  expect(screen.getByRole('dialog')).toHaveTextContent('附着于 地区地区甲');
  fireEvent.click(screen.getByRole('button', { name: '关闭放大阅读' }));
  fireEvent.click(screen.getByRole('button', { name: '查看地区1 地区甲' }));
  expect(within(screen.getByRole('region', { name: '此地区的附属' })).getByRole('button', { name: '查看地区附属动作具现化' })).toBeInTheDocument();
  m.rerender(<Table view={{ ...view, version: 2, regions: [{ ...view.regions[0], id: 'replacement-region-instance' }] }} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(screen.queryByRole('button', { name: '查看地区附属具现化' })).not.toBeInTheDocument(); m.unmount();
});

it('forwards the exact server-authored region play and attachment destruction target', () => {
  const submit = vi.fn(); const hand = { ...aura, owner: 'p0', controller: 'p0', instanceId: 'held-XQ43', region: undefined };
  const play = { id: 'play-region', kind: 'play', cardId: hand.instanceId, region: 0, label: '结附地区' };
  let m = render(<Table view={{ ...view, hand: [hand], attachments: [], legalActions: [play] }} catalog={catalog} busy={false} onAction={submit} />);
  fireEvent.click(screen.getByRole('button', { name: '查看具现化' }));
  const actionButton = screen.queryByRole('button', { name: '结附地区' }); if (actionButton) fireEvent.click(actionButton);
  fireEvent.click(screen.getByRole('button', { name: '查看地区1 地区甲' }));
  fireEvent.click(screen.getByRole('button', { name: /确认 · 结附地区/ })); expect(submit).toHaveBeenCalledExactlyOnceWith(play); m.unmount();
  submit.mockClear(); const source = { ...hand, instanceId: 'held-JC005', cardId: 'JC005', name: '裂解术' };
  const destroy = { id: 'destroy-region-attachment', kind: 'play', cardId: source.instanceId, targetId: aura.instanceId, label: '裂解地区附属' };
  m = render(<Table view={{ ...view, hand: [source], legalActions: [destroy] }} catalog={catalog} busy={false} onAction={submit} />);
  fireEvent.click(screen.getByRole('button', { name: '查看裂解术' }));
  const targetButton = screen.queryByRole('button', { name: '裂解地区附属' }); if (targetButton) fireEvent.click(targetButton);
  expect(m.container.querySelector('[data-attachment-instance="actual-region-aura"]')).toHaveAttribute('data-card-targeted', 'true');
  fireEvent.click(screen.getByRole('button', { name: '查看地区附属具现化' }));
  fireEvent.click(screen.getByRole('button', { name: /确认 · 裂解地区附属/ })); expect(submit).toHaveBeenCalledExactlyOnceWith(destroy); m.unmount();
});

it('displays server conversion alongside immutable printed icons and clears it after departure', () => {
  const original = JSON.stringify(spiritDefinition);
  const m = render(<CardContent card={spirit} definition={spiritDefinition} viewerId="p0" />);
  expect(screen.getByLabelText('当前有效图标：调查0，战斗0，势力1')).toBeInTheDocument();
  expect(screen.getByLabelText('本地区临时转永久图标：调查0，战斗0，势力1')).toBeInTheDocument();
  expect(screen.getByLabelText('印刷图标：调查0，战斗0，势力0')).toBeInTheDocument();
  expect(JSON.stringify(spiritDefinition)).toBe(original);
  m.rerender(<CardContent card={{ ...spirit, convertedTemporaryIcons: undefined, icons: { investigation: 0, combat: 0, influence: 0 } }} definition={spiritDefinition} viewerId="p0" />);
  expect(screen.queryByText('具现化：临时转永久')).not.toBeInTheDocument();
  expect(screen.getByLabelText('当前有效图标：调查0，战斗0，势力0')).toBeInTheDocument(); m.unmount();
});

it('does not leak conversion or original identity through concealed or older projections', () => {
  const concealed = { ...spirit, faceDown: true };
  const sanitized = visibleCard(concealed, 'p0'); expect(sanitized.convertedTemporaryIcons).toBeUndefined(); expect(sanitized.cardId).toBeUndefined();
  expect(visibleCard(concealed, 'p1').convertedTemporaryIcons).toBeUndefined();
  let m = render(<CardContent card={concealed} definition={spiritDefinition} viewerId="p0" />);
  expect(screen.queryByText('具现化：临时转永久')).not.toBeInTheDocument(); expect(screen.queryByText('噩梦残像')).not.toBeInTheDocument(); m.unmount();
  m = render(<Table view={{ ...view, attachments: undefined, regions: [{ ...view.regions[0], characters: [{ ...spirit, convertedTemporaryIcons: undefined }] }] }} catalog={catalog} busy={false} onAction={vi.fn()} />);
  expect(screen.queryByRole('button', { name: '查看地区附属具现化' })).not.toBeInTheDocument(); m.unmount();
});
