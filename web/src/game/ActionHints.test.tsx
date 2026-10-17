import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCard, testCatalog, testChoice, testView } from './testFixtures';
import type { Catalog, View } from './types';

const catalog: Catalog = { ...testCatalog, cards: testCatalog.cards.map(card => ({ ...card, abilities: [
  { key: 'fast-key', label: '不依赖标签猜测', timing: 'fast', triggered: false },
  { key: 'action-fast-key', label: '行动阶段能力', timing: 'actionFast', triggered: false },
] })) };
const pass = { id: 'pass', kind: 'pass', label: '让过' };
const deploy = { id: 'deploy', kind: 'deploy', cardId: testCard.instanceId, region: 0, label: '派遣 → 地区' };
const fast = { id: 'fast', kind: 'activate', cardId: testCard.instanceId, abilityId: 'fast-key', label: '发动能力' };
const view: View = { ...testView, status: 'playing', regions: [{ id: 'region-one', index: 0, cardId: 'JC001', name: '地区', threshold: 3, points: 1, influence: [0, 0], characters: [] }], legalActions: [deploy, fast, pass] };
const tile = (container: HTMLElement, id = testCard.instanceId) => container.querySelector<HTMLElement>(`[data-card-instance="${id}"]`)!;

it('shows server candidate types on the hand and dock without guessing from printed abilities', () => {
  const submit = vi.fn();
  const app = render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={submit} />);
  expect(within(tile(app.container)).getByText('候选·标准/快速')).toBeInTheDocument();
  expect(tile(app.container)).toHaveAccessibleDescription(expect.stringContaining('仍需确认目标、费用与后续选择'));
  const dock = screen.getByRole('region', { name: '当前窗口与候选行动' });
  expect(dock).toHaveTextContent('当前候选：标准行动、快速行动');
  expect(dock).toHaveTextContent('可直接让过');
  expect(screen.getByRole('button', { name: '让过' })).toBeEnabled();
  expect(submit).not.toHaveBeenCalled();
  app.rerender(<Table view={{ ...view, version: 2, legalActions: [pass] }} catalog={catalog} busy={false} connection="online" onAction={submit} />);
  expect(tile(app.container)).toHaveAttribute('data-card-actionable', 'false');
  expect(screen.queryByText('候选·标准/快速')).not.toBeInTheDocument();
});

it('marks own hidden reveal and mounted/society sources but never a destination or foreign hidden identity', () => {
  const hidden = { ...testCard, instanceId: 'hidden-own', faceDown: true };
  const enemy = { ...testCard, instanceId: 'hidden-enemy', owner: 'p1', controller: 'p1', faceDown: true, name: '秘密原名' };
  const society = { ...testCard, instanceId: 'society', kind: 'society' };
  const attached = { ...testCard, instanceId: 'mounted', kind: 'attachment', hostId: hidden.instanceId };
  const current: View = { ...view, hand: [], players: [...view.players, { ...view.players[0], id: 'p1', seat: 1, name: '乙', team: 1 }],
    regions: [{ ...view.regions[0], characters: [hidden, enemy] }], attachments: [attached], societyZones: [{ id: 'society-zone', playerId: 'p0', card: society }],
    legalActions: [{ id: 'reveal', kind: 'reveal', cardId: hidden.instanceId, label: '现身' },
      { ...fast, id: 'mounted-fast', cardId: attached.instanceId, targetId: enemy.instanceId },
      { ...fast, id: 'society-fast', cardId: society.instanceId },
      { ...fast, id: 'malformed-secret', cardId: enemy.instanceId }] };
  const app = render(<Table view={current} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(app.container, hidden.instanceId)).toHaveTextContent('候选·快速');
  expect(tile(app.container, society.instanceId)).toHaveTextContent('候选·快速');
  expect(app.container.querySelector('[data-attachment-instance="mounted"]')).toHaveTextContent('候选·快速');
  expect(tile(app.container, enemy.instanceId)).toHaveAttribute('data-card-actionable', 'false');
  expect(tile(app.container, enemy.instanceId)).not.toHaveTextContent('候选');
  expect(screen.queryByText('秘密原名')).not.toBeInTheDocument();
});

it('follows phase and resource changes solely through replacement legal actions and survives remount', () => {
  const app = render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  const end: View = { ...view, version: 2, phase: 'end', step: 'end', assets: [], legalActions: [{ ...fast, abilityId: 'action-fast-key' }, pass] };
  app.rerender(<Table view={end} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(app.container)).toHaveTextContent('候选·行动阶段快速');
  expect(screen.getByRole('region', { name: '当前窗口与候选行动' })).toHaveTextContent('窗口：结束阶段');
  const waiting: View = { ...end, version: 3, priorityTeam: 1, legalActions: [] };
  app.rerender(<Table view={waiting} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(app.container)).toHaveAttribute('data-card-actionable', 'false');
  app.unmount();
  const refreshed = render(<Table view={waiting} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(refreshed.container)).not.toHaveTextContent('候选');
});

it.each(['busy', 'uncertain', 'offline', 'connecting', 'pause', 'pending', 'waiting'] as const)('clears stale hints while %s without changing server actions', state => {
  const current = { ...view, ...(state === 'pause' ? { pause: { pausedAtMs: 1000, pausedBy: 0 } } : {}),
    ...(state === 'pending' ? { pendingChoice: testChoice } : {}),
    ...(state === 'waiting' ? { waitingChoice: { playerId: 'p1', kind: 'target', title: '目标选择' } } : {}) };
  const app = render(<Table view={current} catalog={catalog} busy={state === 'busy'} uncertain={state === 'uncertain'} connection={state === 'offline' || state === 'connecting' ? state : 'online'} onAction={vi.fn()} />);
  expect(tile(app.container)).toHaveAttribute('data-card-actionable', 'false');
  expect(tile(app.container)).not.toHaveTextContent('候选·');
  expect(current.legalActions).toEqual(view.legalActions);
});

it('waits for an authoritative composing intent and removes hints again after passing the response', () => {
  const current: View = { ...view, stack: [{ id: 'effect', label: '待结算', controller: 'p1' }], legalActions: [fast, pass],
    responseWindow: { id: 'window', stackTopId: 'effect', holderTeam: 0, canBegin: true, members: [{ playerId: 'p0', status: 'undecided' }] } };
  const app = render(<Table view={current} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(app.container)).toHaveAttribute('data-card-actionable', 'false');
  expect(screen.getByRole('region', { name: '当前窗口与候选行动' })).toHaveTextContent('先选择“连锁”');
  const composing: View = { ...current, version: 2, responseWindow: { ...current.responseWindow!, myIntentId: 'intent', members: [{ playerId: 'p0', status: 'composing' }] } };
  app.rerender(<Table view={composing} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(app.container)).toHaveTextContent('候选·快速');
  app.rerender(<Table view={{ ...composing, version: 3, responseWindow: { ...composing.responseWindow!, myIntentId: undefined, canBegin: false, members: [{ playerId: 'p0', status: 'passed' }] } }} catalog={catalog} busy={false} connection="online" onAction={vi.fn()} />);
  expect(tile(app.container)).toHaveAttribute('data-card-actionable', 'false');
});

it('separates candidate sources from target selection and preserves one confirmed submission on repeated clicks', () => {
  const submit = vi.fn();
  const app = render(<Table view={{ ...view, legalActions: [deploy, pass] }} catalog={catalog} busy={false} connection="online" onAction={submit} />);
  fireEvent.click(tile(app.container));
  fireEvent.click(screen.getByRole('button', { name: '派遣' }));
  expect(tile(app.container)).toHaveAttribute('data-card-actionable', 'false');
  expect(screen.getByRole('region', { name: '当前窗口与候选行动' })).toHaveTextContent('正在选目标');
  fireEvent.click(screen.getByRole('button', { name: '查看地区1 地区' }));
  const confirm = within(screen.getByLabelText('确认目标行动')).getByRole('button');
  fireEvent.click(confirm); fireEvent.click(confirm);
  expect(submit).toHaveBeenCalledExactlyOnceWith(deploy);
});
