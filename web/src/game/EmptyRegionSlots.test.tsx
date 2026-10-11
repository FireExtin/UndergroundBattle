import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { actionForRoom } from './api';
import { Table } from './Table';
import { testCard, testCatalog, testView } from './testFixtures';
import type { Action, Card, LegalAction, Region, View } from './types';

// Public engine projections keep the original slots after the world deck runs out.
const zeroIcons = { investigation: 0, combat: 0, influence: 0 };
const regionAt = (index: number): Region => ({
  id: `world-instance-${index}`, index, cardId: 'DQJC112', name: `地区牌${index + 1}`,
  threshold: 4, points: 3, influence: [1, 2], characters: [],
  iconsByTeam: [{ investigation: 1, combat: 2, influence: 3 }, { investigation: 3, combat: 2, influence: 1 }],
});
const emptyAt = (index: number): Region => ({
  id: `empty-region:${index}`, index, cardId: '', name: '空位',
  threshold: 0, points: 0, influence: [0, 0], characters: [],
  iconsByTeam: [zeroIcons, zeroIcons], skipConfrontation: true,
});
const source = { ...testCard, instanceId: 'empty-slot-source', name: '行动来源' };
const props = { catalog: testCatalog, busy: false, connection: 'online' as const };
const board = (mode: View['mode'], emptyIndices: number[] = []): View => ({
  ...testView, status: 'playing', mode, serverNowMs: 1000, worldDeckCount: 0, hand: [source], legalActions: [],
  regions: Array.from({ length: mode === 'duel' ? 3 : 5 }, (_, index) => emptyIndices.includes(index) ? emptyAt(index) : regionAt(index)),
  players: Array.from({ length: mode === 'duel' ? 2 : 4 }, (_, seat) => ({
    ...testView.players[0], id: `p${seat}`, seat, name: seat === 0 ? '甲' : `玩家${seat + 1}`,
    team: mode === 'duel' ? seat : Math.floor(seat / 2),
  })),
});
const asResponse = (view: View): View => ({
  ...view, serverNowMs: 1000,
  stack: [{ id: 'same-stack-top', label: '对手待结算效果', controller: 'p1', resolutionState: 'awaitingResponses' }],
  responseWindow: { id: 'same-response-window', stackTopId: 'same-stack-top', holderTeam: 0,
    canBegin: true, myIntentId: 'same-composing-intent', members: [{ playerId: 'p0', status: 'composing' }] },
});
const labelAt = (index: number) => `地区 ${index + 1} · 地区牌${index + 1}`;
const chooseRegion = (index: number) => {
  fireEvent.click(screen.getByRole('button', { name: '查看行动来源' }));
  fireEvent.click(screen.getByRole('button', { name: '发动地区能力' }));
  fireEvent.click(screen.getByRole('button', { name: `查看地区${index + 1} 地区牌${index + 1}` }));
};
const regionAction = (index: number): LegalAction => ({
  id: `region-action-${index}`, kind: 'activate', cardId: source.instanceId, abilityId: 'region-effect',
  region: index, label: `发动地区能力 → 地区牌${index + 1}`,
});

describe('fixed empty world slots', () => {
  it.each([
    ['duel', [1]], ['duel', [0, 2]], ['duel', [0, 1, 2]],
    ['teams', [2]], ['teams', [0, 2, 4]], ['teams', [0, 1, 2, 3, 4]],
  ] as [View['mode'], number[]][])('keeps every %s slot in its original order with empty indices %j', (mode, emptyIndices) => {
    const view = board(mode, emptyIndices);
    const submit = vi.fn();
    const { container } = render(<Table {...props} view={view} onAction={submit} />);
    const world = container.querySelector('.hg-board')!;
    const slots = within(world as HTMLElement).getAllByRole('article');
    expect(slots).toHaveLength(mode === 'duel' ? 3 : 5);
    expect(slots.map(slot => slot.id)).toEqual(view.regions.map(region => `hg-region-${region.index}`));
    expect(slots.map(slot => slot.getAttribute('aria-label'))).toEqual(view.regions.map(region => `地区 ${region.index + 1} · ${region.name}`));
    for (const index of emptyIndices) {
      const slot = screen.getByRole('article', { name: `地区 ${index + 1} · 空位` });
      expect(within(slot).getByText(`地区 ${index + 1} · 空位`)).toBeInTheDocument();
      expect(within(slot).queryByRole('button')).not.toBeInTheDocument();
      expect(within(slot).queryByRole('img')).not.toBeInTheDocument();
      expect(slot.querySelector('img, .hg-region-center, .hg-region-side, .hg-region-threshold, .hg-region-value')).toBeNull();
      expect(slot).not.toHaveAttribute('data-region-targeted', 'true');
      expect(slot).not.toHaveAttribute('data-region-available', 'true');
      expect(slot).not.toHaveTextContent(/控制阈值|势力标志|争夺区|合法落点|略过对抗|分/);
      fireEvent.click(slot);
    }
    expect(screen.queryByRole('complementary', { name: '选牌行动' })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it.each([
    [0, false], [7, true], [0, true],
  ] as [number, boolean][])('keeps nonempty cards on the board with worldDeckCount=%i and skipConfrontation=%s', (worldDeckCount, skipConfrontation) => {
    const view = { ...board('duel'), worldDeckCount, regions: board('duel').regions.map(region => ({ ...region, skipConfrontation })) };
    render(<Table {...props} view={view} onAction={vi.fn()} />);
    const slot = screen.getByRole('article', { name: labelAt(1) });
    expect(within(slot).getByRole('button', { name: '查看地区2 地区牌2' })).toBeInTheDocument();
    expect(within(slot).getByText('控制阈值')).toHaveTextContent('控制阈值 4');
    expect(within(slot).getAllByText('势力标志')).toHaveLength(2);
    expect(within(slot).getAllByRole('img', { name: /^当前对抗图标/ })).toHaveLength(2);
    fireEvent.click(within(slot).getByRole('button', { name: '查看地区2 地区牌2' }));
    expect(screen.getByRole('heading', { name: labelAt(1) })).toBeInTheDocument();
    expect(screen.getByText('控制阈值 4 · 赢得后 3 分')).toBeInTheDocument();
  });

  it('highlights only real destinations and gives an empty slot no dispatch interaction', () => {
    const submit = vi.fn();
    const view = board('duel', [1]);
    const actions = [0, 2].map(index => ({ ...regionAction(index), kind: 'deploy', abilityId: undefined,
      label: `派遣行动来源 → 地区牌${index + 1}` }));
    render(<Table {...props} view={{ ...view, legalActions: actions }} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看行动来源' }));
    fireEvent.click(screen.getByRole('button', { name: '派遣行动来源' }));
    const empty = screen.getByRole('article', { name: '地区 2 · 空位' });
    expect(empty).not.toHaveAttribute('data-region-available', 'true');
    expect(empty).not.toHaveAttribute('data-region-targeted', 'true');
    expect(within(empty).queryByText('合法落点')).not.toBeInTheDocument();
    for (const index of [0, 2]) {
      const real = screen.getByRole('article', { name: labelAt(index) });
      expect(real).toHaveAttribute('data-region-available', 'true');
      expect(within(real).getByText('合法落点')).toBeInTheDocument();
    }
    fireEvent.click(empty);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '查看地区3 地区牌3' }));
    fireEvent.click(screen.getByRole('button', { name: '确认 · 派遣行动来源 → 地区 3 · 地区牌3' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(actions[1]);
  });

  it.each(['ordinary', 'response'] as const)('clears a selected %s region when that slot becomes empty and never submits its old confirmation', frame => {
    const actions = [regionAction(1), regionAction(2)];
    const initial = { ...board('duel'), legalActions: actions };
    const view = frame === 'response' ? asResponse(initial) : initial;
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    chooseRegion(1);
    const oldConfirm = screen.getByRole('button', { name: `确认 · 发动地区能力 → ${labelAt(1)}` });
    expect(oldConfirm).toBeEnabled();
    const emptied = { ...view, version: 2, regions: view.regions.map(region => region.index === 1 ? emptyAt(1) : region), legalActions: [actions[1]] };
    rerender(<Table {...props} view={emptied} onAction={submit} />);
    expect(screen.getByRole('article', { name: '地区 2 · 空位' })).toBeInTheDocument();
    expect(screen.getByText('所选目标已失效，请重新选择目标。')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    fireEvent.click(oldConfirm);
    expect(submit).not.toHaveBeenCalled();
    // A later real card at that index cannot resurrect an old destination choice.
    rerender(<Table {...props} view={{ ...view, version: 3, regions: view.regions.map(region => region.index === 1 ? { ...region, id: 'replacement-world-instance' } : region) }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看地区3 地区牌3' }));
    fireEvent.click(screen.getByRole('button', { name: `确认 · 发动地区能力 → ${labelAt(2)}` }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(actions[1]);
  });

  it.each(['ordinary', 'response'] as const)('preserves a selected real %s destination when another slot becomes empty', frame => {
    const actions = [regionAction(1), regionAction(2)];
    const initial = { ...board('duel'), legalActions: actions };
    let latest = frame === 'response' ? asResponse(initial) : initial;
    const sent = vi.fn();
    const selected = vi.fn((action: Action) => sent(latest.version, actionForRoom(latest, action)));
    const { rerender } = render(<Table {...props} view={latest} onAction={selected} />);
    chooseRegion(1);
    const freshAction = { ...actions[0], id: 'fresh-region-action' };
    latest = { ...latest, version: 2, regions: latest.regions.map(region => region.index === 0 ? emptyAt(0) : region), legalActions: [freshAction, actions[1]] };
    rerender(<Table {...props} view={latest} onAction={selected} />);
    expect(screen.getByRole('article', { name: '地区 1 · 空位' })).toBeInTheDocument();
    const confirm = screen.getByRole('button', { name: `确认 · 发动地区能力 → ${labelAt(1)}` });
    expect(confirm).toBeEnabled();
    expect(screen.queryByText('所选目标已失效，请重新选择目标。')).not.toBeInTheDocument();
    expect(sent).not.toHaveBeenCalled();
    fireEvent.click(confirm);
    expect(selected).toHaveBeenCalledExactlyOnceWith(freshAction);
    expect(sent).toHaveBeenCalledExactlyOnceWith(2, frame === 'response' ? {
      kind: 'submitResponse', windowId: 'same-response-window', intentId: 'same-composing-intent',
      action: { kind: 'activate', cardId: source.instanceId, abilityId: 'region-effect', region: 1 },
    } : { kind: 'game', action: { kind: 'activate', cardId: source.instanceId, abilityId: 'region-effect', region: 1 } });
  });

  it.each(['ordinary', 'response'] as const)('keeps another legal %s character target selected after an unrelated region empties', frame => {
    const target = { ...testCard, instanceId: 'retained-target', name: '保留目标', region: 1 };
    const action: LegalAction = { id: 'target-character', kind: 'activate', cardId: source.instanceId, abilityId: 'character-effect', targetId: target.instanceId, label: '发动角色能力 → 保留目标' };
    const initial = { ...board('duel'), regions: board('duel').regions.map(region => region.index === 1 ? { ...region, characters: [target] } : region), legalActions: [action] };
    const view = frame === 'response' ? asResponse(initial) : initial;
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看行动来源' }));
    fireEvent.click(screen.getByRole('button', { name: '发动角色能力' }));
    fireEvent.click(screen.getByRole('button', { name: '查看保留目标' }));
    const fresh = { ...action, id: 'fresh-character-action' };
    rerender(<Table {...props} view={{ ...view, version: 2, regions: view.regions.map(region => region.index === 0 ? emptyAt(0) : region), legalActions: [fresh] }} onAction={submit} />);
    expect(screen.getByText('目标：甲的保留目标')).toBeInTheDocument();
    const confirm = screen.getByRole('button', { name: '确认 · 发动角色能力 → 保留目标' });
    expect(confirm).toBeEnabled();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith(fresh);
  });

  it('keeps a won region readable in its owner’s score pile without restoring its battlefield slot', () => {
    const won: Card = { ...testCard, instanceId: regionAt(1).id, cardId: 'DQJC112', name: '已赢得的纽约', kind: 'region', text: '已赢走地区的印刷文字' };
    const view = { ...board('duel', [1]), scoreCards: [won], players: board('duel').players.map(player => player.id === won.owner ? { ...player, score: 3 } : player) };
    const { container } = render(<Table {...props} view={view} onAction={vi.fn()} />);
    fireEvent.click(screen.getByLabelText('查看甲的计分，1张'));
    const pile = container.querySelector('[data-pile-owner="p0"][data-pile-kind="scoreCards"]') as HTMLElement;
    fireEvent.click(within(pile).getByRole('button', { name: '查看已赢得的纽约' }));
    fireEvent.click(screen.getByRole('button', { name: '放大文字与图标 ↗' }));
    const reading = screen.getByRole('dialog', { name: '放大阅读已赢得的纽约' });
    expect(within(reading).getByText('已赢走地区的印刷文字')).toBeInTheDocument();
    fireEvent.click(within(reading).getByRole('button', { name: '关闭放大阅读' }));
    const world = container.querySelector('.hg-board') as HTMLElement;
    expect(within(world).getAllByRole('article')).toHaveLength(3);
    expect(within(world).getByRole('article', { name: '地区 2 · 空位' }).querySelector('button, img')).toBeNull();
    expect(within(world).queryByText('已赢得的纽约')).not.toBeInTheDocument();
  });

  it('closes a browsed region’s details when the slot empties and requires an explicit click on a later replacement', () => {
    const view = board('duel');
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={view} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看地区2 地区牌2' }));
    expect(screen.getByRole('heading', { name: labelAt(1) })).toBeInTheDocument();
    expect(screen.getByText('控制阈值 4 · 赢得后 3 分')).toBeInTheDocument();
    expect(screen.getByText('选择手牌可查看向此地区派遣的行动。')).toBeInTheDocument();
    rerender(<Table {...props} view={{ ...view, version: 2,
      regions: view.regions.map(region => region.index === 1 ? emptyAt(1) : region) }} onAction={submit} />);
    expect(screen.getByRole('article', { name: '地区 2 · 空位' })).toBeInTheDocument();
    expect(screen.queryByRole('complementary', { name: '选牌行动' })).not.toBeInTheDocument();
    expect(screen.queryByText('选择手牌可查看向此地区派遣的行动。')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '放大地区文字 ↗' })).not.toBeInTheDocument();
    const replacement = { ...regionAt(1), id: 'later-world-instance', name: '新地区牌', threshold: 5, points: 2 };
    rerender(<Table {...props} view={{ ...view, version: 3,
      regions: view.regions.map(region => region.index === 1 ? replacement : region) }} onAction={submit} />);
    expect(screen.queryByRole('complementary', { name: '选牌行动' })).not.toBeInTheDocument();
    expect(screen.queryByText('选择手牌可查看向此地区派遣的行动。')).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '查看地区2 新地区牌' }));
    expect(screen.getByRole('heading', { name: '地区 2 · 新地区牌' })).toBeInTheDocument();
    expect(screen.getByText('控制阈值 5 · 赢得后 2 分')).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it('disables a controlled stale action targeting an empty slot identifier without a region field', () => {
    // A deliberately stale fixture protects the UI guard; the engine excludes this illegal action.
    const stale: LegalAction = { id: 'controlled-stale-slot-target', kind: 'privilege',
      targetId: 'empty-region:1', label: '受控过时空槽目标' };
    const submit = vi.fn();
    render(<Table {...props} view={{ ...board('duel', [1]), legalActions: [stale] }} onAction={submit} />);
    const button = screen.getByRole('button', { name: stale.label });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('title', '目标当前未显示，请等待牌桌更新。');
    fireEvent.click(button);
    expect(submit).not.toHaveBeenCalled();
  });

  it('shows the old real region as missing on the stack after its fixed slot becomes empty', () => {
    const original = regionAt(1);
    const view: View = { ...board('duel', [1]), stack: [{ id: 'old-region-effect', label: '原地区上的待结算效果', controller: 'p1',
      resolutionState: 'awaitingResponses', targetId: original.id, targets: [original.id],
      targetSummaries: [{ instanceId: original.id, label: original.name, kind: 'region', region: original.index,
        valid: false, status: 'missing', invalidReason: '原目标已离场' }] }] };
    const { container } = render(<Table {...props} view={view} onAction={vi.fn()} />);
    const response = screen.getByRole('region', { name: '当前待结算效果与响应' });
    expect(within(response).getByText('原目标已离场')).toBeInTheDocument();
    const target = response.querySelector(`[data-target-instance="${original.id}"]`)!;
    expect(target).toHaveAttribute('data-target-valid', 'false');
    expect(target).toHaveAttribute('data-target-status', 'missing');
    expect(response.querySelector('[data-target-instance="empty-region:1"]')).toBeNull();
    expect(container.querySelector('#hg-region-1 button')).toBeNull();
  });
});
