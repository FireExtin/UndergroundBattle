import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { actionForRoom } from './api';
import { Table } from './Table';
import { testCard, testCatalog, testChoice, testView } from './testFixtures';
import type { Action, LegalAction, View } from './types';

// Deterministic authorized projections exercise local interaction, not engine rules.
const source = { ...testCard, instanceId: 'source', name: '行动来源' };
const target = { ...testCard, instanceId: 'target', name: '目标角色', owner: 'p1', controller: 'p1' };
const alternate = { ...target, instanceId: 'alternate', name: '另一角色' };
const region = { id: 'region-instance', index: 0, cardId: 'world', name: '纽约', threshold: 3, points: 3, influence: [0, 0], characters: [target, alternate] };
const action: LegalAction = { id: 'action-v1', kind: 'activate', cardId: source.instanceId, abilityId: 'targeted', targetId: target.instanceId, label: '发动能力 → 目标角色' };
const alternative: LegalAction = { ...action, id: 'alternate-action', targetId: alternate.instanceId, label: '发动能力 → 另一角色' };
const ordinary: View = { ...testView, status: 'playing', mode: 'teams', serverNowMs: 1000, hand: [source], regions: [region],
  players: [testView.players[0], { ...testView.players[0], id: 'p1', seat: 1, name: '队友' },
    { ...testView.players[0], id: 'p2', seat: 2, name: '对手甲', team: 1 },
    { ...testView.players[0], id: 'p3', seat: 3, name: '对手乙', team: 1 }],
  legalActions: [action, alternative] };
const props = { catalog: testCatalog, busy: false, connection: 'online' as const };
const choose = () => {
  fireEvent.click(screen.getByRole('button', { name: '查看行动来源' }));
  fireEvent.click(screen.getByRole('button', { name: '发动能力' }));
  fireEvent.click(screen.getByRole('button', { name: '查看目标角色' }));
};

describe('ordinary action selection across peer updates', () => {
  it('keeps the entry mode before a target click and uses the latest authoritative action exactly once', () => {
    let latest = ordinary;
    const sent = vi.fn();
    const selectedActions: Action[] = [];
    const onAction = (selected: Action) => { selectedActions.push(selected); sent(latest.version, actionForRoom(latest, selected)); };
    const { rerender } = render(<Table {...props} view={latest} onAction={onAction} />);
    fireEvent.click(screen.getByRole('button', { name: '查看行动来源' }));
    fireEvent.click(screen.getByRole('button', { name: '发动能力' }));
    latest = { ...ordinary, version: 2, legalActions: [{ ...action, id: 'action-v2' }, alternative],
      assets: [{ ...testCard, instanceId: 'teammate-asset', owner: 'p1', controller: 'p1' }] };
    rerender(<Table {...props} view={latest} onAction={onAction} />);
    expect(screen.getByRole('region', { name: '点选合法目标' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '查看目标角色' }));
    const confirm = screen.getByRole('button', { name: '确认 · 发动能力 → 目标角色' });
    act(() => { fireEvent.click(confirm); fireEvent.click(confirm); });
    expect(sent).toHaveBeenCalledTimes(1);
    expect(selectedActions[0]).toBe(latest.legalActions[0]);
    expect(JSON.parse(JSON.stringify(sent.mock.calls[0]))).toEqual([2, { kind: 'game', action: {
      kind: 'activate', cardId: source.instanceId, abilityId: 'targeted', targetId: target.instanceId,
    } }]);
  });

  it('retains a legal target through several teammate updates and busy or uncertain refreshes without submitting', () => {
    const sent = vi.fn();
    const { rerender } = render(<Table {...props} view={ordinary} onAction={sent} />);
    choose();
    for (const [index, lock] of [{ busy: true }, { uncertain: true }, {}].entries()) {
      rerender(<Table {...props} {...lock} view={{ ...ordinary, version: index + 2,
        players: ordinary.players.map(player => player.id === 'p1' ? { ...player, handCount: index + 2 } : player),
        legalActions: [{ ...action, id: `latest-${index}` }, alternative] }} onAction={sent} />);
      const confirm = screen.getByRole('button', { name: '确认 · 发动能力 → 目标角色' });
      if (index < 2) expect(confirm).toBeDisabled();
      else expect(confirm).toBeEnabled();
      expect(screen.getByText('目标：队友的目标角色')).toBeInTheDocument();
      if (index < 2) fireEvent.click(confirm);
    }
    expect(sent).not.toHaveBeenCalled();
  });

  const changedWindows: [string, Partial<View>][] = [
    ['turn', { turn: 2 }], ['phase', { phase: 'confrontation' }], ['step', { step: 'region:0:combat' }],
    ['active team', { activeTeam: 1 }], ['priority team', { priorityTeam: 1 }],
    ['stack opens', { stack: [{ id: 'new-effect', label: '新效果', controller: 'p1' }] }],
    ['response window opens', { responseWindow: { id: 'window', stackTopId: 'top', holderTeam: 0, canBegin: true, members: [] } }],
    ['own choice opens', { pendingChoice: testChoice }],
    ['peer choice opens', { waitingChoice: { playerId: 'p1', title: '队友选择', kind: 'target' } }],
    ['game ends', { status: 'finished' }], ['room changes', { roomId: 'another-room' }], ['seat changes', { you: 'p1' }],
  ];
  it.each(changedWindows)('requires a new explicit selection when %s changes, even if legal payloads look identical', (_name, change) => {
    const sent = vi.fn();
    const { rerender } = render(<Table {...props} view={ordinary} onAction={sent} />);
    choose();
    rerender(<Table {...props} view={{ ...ordinary, ...change, version: 2 }} onAction={sent} />);
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    expect(sent).not.toHaveBeenCalled();
    rerender(<Table {...props} view={{ ...ordinary, version: 3 }} onAction={sent} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
  });

  it.each(['source disappears', 'action disappears', 'ability changes', 'cost changes', 'mode changes'])('invalidates the draft when %s in the same window', change => {
    const sent = vi.fn();
    const { rerender } = render(<Table {...props} view={ordinary} onAction={sent} />);
    choose();
    const next = { ...action, ...(change === 'ability changes' ? { abilityId: 'other' } : {}),
      ...(change === 'cost changes' ? { costSelected: ['new-cost'] } : {}),
      ...(change === 'mode changes' ? { option: 'other-mode' } : {}) };
    rerender(<Table {...props} view={{ ...ordinary, version: 2,
      hand: change === 'source disappears' ? [] : ordinary.hand,
      legalActions: change === 'action disappears' ? [] : [next] }} onAction={sent} />);
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    expect(sent).not.toHaveBeenCalled();
  });

  it.each(['hidden', 'replacement instance', 'no longer legal'])('clears a target that is %s and permits only an explicit alternative', change => {
    const sent = vi.fn();
    const { rerender } = render(<Table {...props} view={ordinary} onAction={sent} />);
    choose();
    rerender(<Table {...props} view={{ ...ordinary, version: 2,
      regions: [{ ...region, characters: change === 'hidden' ? [alternate]
        : change === 'replacement instance' ? [{ ...target, instanceId: 'replacement' }, alternate] : region.characters }],
      legalActions: change === 'no longer legal' ? [alternative] : ordinary.legalActions }} onAction={sent} />);
    expect(screen.getByText('所选目标已失效，请重新选择目标。')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(sent).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '查看另一角色' }));
    fireEvent.click(screen.getByRole('button', { name: '确认 · 发动能力 → 另一角色' }));
    expect(sent).toHaveBeenCalledExactlyOnceWith(alternative);
  });

  it('retains a region until its real instance changes, rather than treating an equal index as the same destination', () => {
    const sent = vi.fn();
    const destination = { ...action, targetId: undefined, region: 0, label: '发动能力 → 纽约' };
    const initial = { ...ordinary, legalActions: [destination] };
    const { rerender } = render(<Table {...props} view={initial} onAction={sent} />);
    fireEvent.click(screen.getByRole('button', { name: '查看行动来源' }));
    fireEvent.click(screen.getByRole('button', { name: '发动能力' }));
    fireEvent.click(screen.getByRole('button', { name: '查看地区1 纽约' }));
    rerender(<Table {...props} view={{ ...initial, version: 2 }} onAction={sent} />);
    expect(screen.getByRole('button', { name: '确认 · 发动能力 → 地区 1 · 纽约' })).toBeEnabled();
    rerender(<Table {...props} view={{ ...initial, version: 3, regions: [{ ...region, id: 'replacement-region' }] }} onAction={sent} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.getByText('所选目标已失效，请重新选择目标。')).toBeInTheDocument();
    expect(sent).not.toHaveBeenCalled();
  });
});
