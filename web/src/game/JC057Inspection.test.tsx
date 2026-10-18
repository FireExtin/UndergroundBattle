import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { Table } from './Table';
import { ChoicePanel } from './ChoicePanel';
import { testCard, testCatalog, testView } from './testFixtures';
import type { Card, Choice, LegalAction, View } from './types';

const secret = '仅当前检视者获准看到的印刷正文';
const front: Card = { ...testCard, instanceId: 'enemy-hidden-123456', cardId: 'JC052', name: '终焉的诗寇蒂',
  owner: 'p1', controller: 'p1', region: undefined, faceDown: false, text: secret };
const monitor: Card = { ...testCard, instanceId: 'monitor-654321', cardId: 'JC057', name: '监控员',
  owner: 'p0', controller: 'p0', region: 0, faceDown: false, exhausted: false };
const concealed: Card = { ...front, name: '暗藏者', cardId: undefined, kind: 'hidden', region: 0,
  faceDown: true, text: undefined, cost: undefined, icons: undefined, defense: undefined };
const view: View = { ...testView, status: 'playing', hand: [], assets: [], legalActions: [], pendingChoice: null,
  players: [...testView.players, { ...testView.players[0], id: 'p1', seat: 1, name: '乙', team: 1 }],
  regions: [0, 1, 2].map(index => ({ id: `region:${index}`, index, cardId: 'DQJC115', name: '死者之城',
    threshold: 3, points: 1, influence: [0, 0], characters: index === 0 ? [monitor, concealed] : [] })),
  privateInspections: [{ sourceInstanceId: monitor.instanceId, region: 0, cards: [front] }] };
const definitions = new Map(testCatalog.cards.map(card => [card.id, card]));
const inspect: Choice = { id: 'inspect-hand', kind: 'jc057-hand-inspect', playerId: 'p0',
  title: '监控员：检视目标玩家手牌', description: '仅你可见。确认后结束检视，不弃牌、不公开展示。',
  min: 0, max: 0, allowDecline: false, options: [], previewCards: [front] };
const choose: LegalAction = { id: 'choose-inspect', kind: 'choose', label: '完成选择', choiceId: inspect.id };

it('reads authorized print separately while keeping the table hidden, and never submits a game command', () => {
  const onAction = vi.fn(); render(<Table view={view} catalog={testCatalog} busy={false} onAction={onAction} />);
  expect(screen.queryByText(secret)).not.toBeInTheDocument();
  const hiddenTile = screen.getByRole('button', { name: /^查看暗藏者/ });
  const button = screen.getByRole('button', { name: /检视暗藏者 .*监控员/ });
  fireEvent.click(button);
  expect(within(screen.getByRole('dialog')).getByText(secret)).toBeInTheDocument();
  expect(within(screen.getByRole('dialog')).getByText('监控员检视 · 仅你可见 · 该牌仍为暗藏者')).toBeInTheDocument();
  expect(hiddenTile).not.toHaveTextContent(front.name);
  fireEvent.click(button); fireEvent.click(button);
  expect(onAction).not.toHaveBeenCalled();
});
it('closes the private reader when permission ends even if the same instance is in another visible zone', () => {
  const props = { catalog: testCatalog, busy: false, onAction: vi.fn() };
  const { rerender } = render(<Table {...props} view={view} />);
  fireEvent.click(screen.getByRole('button', { name: /检视暗藏者 .*监控员/ }));
  rerender(<Table {...props} view={{ ...view, version: 2, privateInspections: undefined, hand: [front] }} />);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /检视暗藏者 .*监控员/ })).not.toBeInTheDocument();
});
it('drops an open inspection when its source region, room or viewer changes', () => {
  const props = { catalog: testCatalog, busy: false, onAction: vi.fn() };
  const { rerender } = render(<Table {...props} view={view} />);
  fireEvent.click(screen.getByRole('button', { name: /检视暗藏者 .*监控员/ }));
  rerender(<Table {...props} view={{ ...view, privateInspections: [{ sourceInstanceId: monitor.instanceId, region: 1, cards: [front] }] }} />);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  rerender(<Table {...props} view={view} />);
  fireEvent.click(screen.getByRole('button', { name: /检视暗藏者 .*监控员/ }));
  rerender(<Table {...props} view={{ ...view, roomId: 'another-room' }} />);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  rerender(<Table {...props} view={view} />);
  fireEvent.click(screen.getByRole('button', { name: /检视暗藏者 .*监控员/ }));
  rerender(<Table {...props} view={{ ...view, you: 'p1', privateInspections: undefined }} />);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});
it('keeps the read permission available inside the current choice without submitting or changing that choice', () => {
  const onAction = vi.fn();
  const pending: Choice = { ...inspect, id: 'discard-now', kind: 'discard', title: '选择弃牌',
    previewCards: undefined, options: [{ id: 'held', label: '手牌' }], min: 1, max: 1 };
  render(<Table view={{ ...view, pendingChoice: pending, legalActions: [{ ...choose, choiceId: pending.id }] }}
    catalog={testCatalog} busy={false} onAction={onAction} />);
  const choiceDialog = screen.getByRole('dialog');
  fireEvent.click(within(choiceDialog).getByRole('button', { name: /检视暗藏者 .*监控员/ }));
  expect(screen.getByText(secret)).toBeInTheDocument();
  expect(onAction).not.toHaveBeenCalled();
});
it('confirms a JC057 hand inspection with no card selection, including an empty hand', () => {
  const onSubmit = vi.fn();
  const { rerender } = render(<ChoicePanel choice={inspect} action={choose} definitions={definitions} busy={false} onSubmit={onSubmit} viewerId="p0" />);
  expect(screen.getByText('你检视的手牌')).toBeInTheDocument();
  expect(screen.getByText(secret)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: /^确认/ }));
  expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ kind: 'choose', choiceId: inspect.id, selected: [] }));
  rerender(<ChoicePanel choice={{ ...inspect, id: 'empty-inspection', previewCards: [] }} action={choose}
    definitions={definitions} busy={false} onSubmit={onSubmit} viewerId="p0" />);
  expect(screen.getByText('该玩家当前没有手牌。')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /^确认/ })).toBeEnabled();
});
it('expires a hand-inspection reader with its choice instead of falling back to the hand or table', () => {
  const props = { catalog: testCatalog, busy: false, onAction: vi.fn() };
  const { rerender } = render(<Table {...props} view={{ ...view, privateInspections: undefined, pendingChoice: inspect, legalActions: [choose] }} />);
  fireEvent.click(screen.getByRole('button', { name: `放大阅读${front.name}` }));
  expect(screen.getAllByRole('dialog').some(dialog => within(dialog).queryByText('检视的手牌 · 仅你可见'))).toBe(true);
  rerender(<Table {...props} view={{ ...view, privateInspections: undefined, pendingChoice: null, hand: [front] }} />);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});
