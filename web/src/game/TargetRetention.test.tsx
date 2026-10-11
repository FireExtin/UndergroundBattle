import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { actionForRoom } from './api';
import { Table } from './Table';
import { testCard, testCatalog, testChoice, testView } from './testFixtures';
import type { Action, LegalAction, View } from './types';

const source = { ...testCard, kind: 'spell', cardId: 'XQ49', name: '葬礼' };
const target = { ...testCard, instanceId: 'grave-target', name: '墓地目标', owner: 'p1', controller: 'p1' };
const responseAction: LegalAction = { id: 'funeral-target-v1', kind: 'play', cardId: source.instanceId, abilityId: 'funeral', targetId: target.instanceId, label: '葬礼：发动 → 墓地目标' };
const composing: View = {
  ...testView, status: 'playing', mode: 'teams', hand: [source], graveyard: [target],
  players: [testView.players[0], { ...testView.players[0], id: 'p1', name: '队友', seat: 1 },
    { ...testView.players[0], id: 'p2', name: '对手甲', seat: 2, team: 1 },
    { ...testView.players[0], id: 'p3', name: '对手乙', seat: 3, team: 1 }],
  stack: [{ id: 'stack-top', label: '对手待结算效果', controller: 'p2', resolutionState: 'awaitingResponses' }],
  serverNowMs: 1000,
  responseWindow: { id: 'response-window', stackTopId: 'stack-top', holderTeam: 0, canBegin: true,
    myIntentId: 'my-composing-intent', members: [{ playerId: 'p0', status: 'composing' },
      { playerId: 'p1', status: 'undecided', deadlineMs: 2000 }] },
  legalActions: [responseAction],
};
const props = { catalog: testCatalog, busy: false, connection: 'online' as const };
const choose = () => {
  fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
  fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' }));
  fireEvent.click(screen.getByRole('button', { name: '查看墓地目标' }));
};

describe('response target retention', () => {
  it('preserves the observed Funeral entry mode before any target click when the teammate deadline passes', () => {
    const submit = vi.fn();
    const funeral = { ...source, instanceId: 'i223', owner: 'p2', controller: 'p2' };
    const grave = { ...target, instanceId: 'i251', cardId: 'JC042', name: '年轻演员', owner: 'p2', controller: 'p2' };
    const quick = { ...responseAction, id: 'funeral-192', cardId: funeral.instanceId, targetId: grave.instanceId, label: '葬礼：发动 → 年轻演员' };
    const deployed: View = { ...composing, version: 191, turn: 2, phase: 'action', step: 'team:1', activeTeam: 1, priorityTeam: 1, you: 'p2',
      hand: [funeral], graveyard: [grave], assets: [{ ...testCard, instanceId: 'ready-asset', kind: 'asset', owner: 'p2', controller: 'p2' }],
      stack: [{ id: 'i266', cardId: 'i229', label: '外科医生派遣至纽约', controller: 'p3', resolutionState: 'awaitingResponses' }],
      serverNowMs: 1000, legalActions: [], responseWindow: { id: 'response:10', stackTopId: 'i266', holderTeam: 1, canBegin: true,
        members: [{ playerId: 'p2', status: 'undecided', deadlineMs: 5206 }, { playerId: 'p3', status: 'undecided', deadlineMs: 5206 }] } };
    const { container, rerender } = render(<Table {...props} view={deployed} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '连锁' }));
    expect(submit).toHaveBeenCalledTimes(1);
    const begin = submit.mock.calls[0][0];
    expect(begin).toMatchObject({ kind: 'beginResponse', windowId: 'response:10' });
    submit.mockClear();
    const choosing: View = { ...deployed, version: 192, legalActions: [quick],
      responseWindow: { ...deployed.responseWindow!, myIntentId: begin.intentId,
        members: [{ playerId: 'p2', status: 'composing' }, { playerId: 'p3', status: 'undecided', deadlineMs: 5206 }] } };
    rerender(<Table {...props} view={choosing} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' }));
    // This matches QA's observed case: the grave target is highlighted but has not been clicked.
    expect(screen.getByText('目标：尚未选择')).toBeInTheDocument();
    expect(container.querySelector('[data-pile-owner="p2"][data-pile-kind="graveyard"]')).toHaveAttribute('open');
    const latestAction = { ...quick, id: 'funeral-193' };
    const peerTimedOut: View = { ...choosing, version: 193, serverNowMs: 5206, legalActions: [latestAction],
      responseWindow: { ...choosing.responseWindow!, members: [{ playerId: 'p2', status: 'composing' }, { playerId: 'p3', status: 'passed' }] } };
    rerender(<Table {...props} view={peerTimedOut} onAction={submit} />);
    expect(screen.getByRole('region', { name: '点选合法目标' })).toBeInTheDocument();
    expect(screen.getByText('目标：尚未选择')).toBeInTheDocument();
    expect(container.querySelector('[data-pile-owner="p2"][data-pile-kind="graveyard"]')).toHaveAttribute('open');
    expect(screen.getByRole('button', { name: '查看年轻演员' })).toHaveAttribute('data-card-targeted', 'true');
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '查看年轻演员' }));
    fireEvent.click(screen.getByRole('button', { name: `确认 · ${quick.label}` }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(latestAction);
  });

  it('keeps a legal selected target when a teammate passes in the same composing frame', () => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={composing} onAction={submit} />);
    choose();
    expect(screen.getByRole('button', { name: `确认 · ${responseAction.label}` })).toBeEnabled();
    const latestAction = { ...responseAction, id: 'funeral-target-v2' };
    const peerPassed: View = { ...composing, version: 2, serverNowMs: 2000, legalActions: [latestAction],
      responseWindow: { ...composing.responseWindow!, members: [{ playerId: 'p0', status: 'composing' }, { playerId: 'p1', status: 'passed' }] } };
    rerender(<Table {...props} view={peerPassed} onAction={submit} />);
    expect(screen.getByText('目标：队友的墓地目标')).toBeInTheDocument();
    const confirm = screen.getByRole('button', { name: `确认 · ${responseAction.label}` });
    expect(confirm).toBeEnabled();
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledExactlyOnceWith(latestAction);
  });

  it('retains selection across fresh projections and submits only the latest response payload once', () => {
    const submit = vi.fn();
    let latest = composing;
    const onAction = (action: Action) => submit(latest.version, actionForRoom(latest, action));
    const { rerender } = render(<Table {...props} view={latest} onAction={onAction} />);
    choose();
    for (let version = 2; version <= 4; version++) {
      latest = { ...composing, version, serverNowMs: 1000 + version * 1000,
        legalActions: [{ ...responseAction, id: `fresh-${version}` }],
        responseWindow: { ...composing.responseWindow!, canBegin: false,
          members: [{ playerId: 'p0', status: 'composing' }, { playerId: 'p1', status: version === 2 ? 'composing' : 'passed' }] } };
      rerender(<Table {...props} view={latest} onAction={onAction} />);
      expect(screen.getByText('目标：队友的墓地目标')).toBeInTheDocument();
      expect(submit).not.toHaveBeenCalled();
    }
    const confirm = screen.getByRole('button', { name: `确认 · ${responseAction.label}` });
    fireEvent.click(confirm); fireEvent.click(confirm);
    expect(submit).toHaveBeenCalledTimes(1);
    expect(JSON.parse(JSON.stringify(submit.mock.calls[0]))).toEqual([4, {
      kind: 'submitResponse', windowId: 'response-window', intentId: 'my-composing-intent',
      action: { kind: 'play', cardId: source.instanceId, abilityId: 'funeral', targetId: target.instanceId },
    }]);
  });

  const endedFrames: [string, Partial<View>][] = [
    ['window closes', { responseWindow: null }],
    ['window changes', { responseWindow: { ...composing.responseWindow!, id: 'new-window' } }],
    ['intent changes', { responseWindow: { ...composing.responseWindow!, myIntentId: 'new-intent' } }],
    ['intent disappears', { responseWindow: { ...composing.responseWindow!, myIntentId: undefined } }],
    ['own member passes', { responseWindow: { ...composing.responseWindow!, members: [{ playerId: 'p0', status: 'passed' }] } }],
    ['own member must decide again', { responseWindow: { ...composing.responseWindow!, members: [{ playerId: 'p0', status: 'undecided', deadlineMs: 5000 }] } }],
    ['own member disappears', { responseWindow: { ...composing.responseWindow!, members: [{ playerId: 'p1', status: 'passed' }] } }],
    ['holder team changes', { responseWindow: { ...composing.responseWindow!, holderTeam: 1 } }],
    ['stack empties', { stack: [] }],
    ['stack/window disagree', { stack: [{ ...composing.stack[0], id: 'different-top' }] }],
    ['next stack effect', { stack: [{ ...composing.stack[0], id: 'new-top' }], responseWindow: { ...composing.responseWindow!, stackTopId: 'new-top' } }],
    ['stack resolves', { stack: [{ ...composing.stack[0], resolutionState: 'resolving' }] }],
    ['game ends', { status: 'finished' }],
    ['private choice opens', { pendingChoice: testChoice }],
    ['peer choice opens', { waitingChoice: { playerId: 'p1', title: '等待选择', kind: 'target' } }],
  ];
  it.each(endedFrames)('clears the draft without sending a stale action when %s, even at the same revision', (_name, change) => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={composing} onAction={submit} />);
    choose();
    const oldConfirm = screen.getByRole('button', { name: `确认 · ${responseAction.label}` });
    rerender(<Table {...props} view={{ ...composing, ...change }} onAction={submit} />);
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    fireEvent.click(oldConfirm);
    expect(submit).not.toHaveBeenCalled();
    // Restoring a superficially identical view must not resurrect a cleared local draft.
    rerender(<Table {...props} view={{ ...composing, version: 2 }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
  });

  it.each(['source disappears', 'action removed', 'ability changes', 'mode changes', 'cost changes'])('clears a draft when %s in the same composing frame', change => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={composing} onAction={submit} />);
    choose();
    const oldConfirm = screen.getByRole('button', { name: `确认 · ${responseAction.label}` });
    const next: View = { ...composing, version: 2,
      ...(change === 'source disappears' ? { hand: [] } : {}),
      legalActions: change === 'action removed' ? [] : [{ ...responseAction,
        ...(change === 'ability changes' ? { abilityId: 'different-ability' } : {}),
        ...(change === 'mode changes' ? { option: 'different-mode' } : {}),
        ...(change === 'cost changes' ? { costSelected: ['different-sacrifice'] } : {}),
      }],
    };
    rerender(<Table {...props} view={next} onAction={submit} />);
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    fireEvent.click(oldConfirm);
    expect(submit).not.toHaveBeenCalled();
  });

  it.each(['projection removes target', 'same printed card replaces target', 'legal list removes target'])('clears only an invalid destination when %s, allowing an explicit new choice', change => {
    const submit = vi.fn();
    const alternate = { ...target, instanceId: 'alternate-target', name: '另一目标' };
    const alternateAction = { ...responseAction, id: 'alternate-action', targetId: alternate.instanceId, label: '葬礼：发动 → 另一目标' };
    const initial = { ...composing, graveyard: [target, alternate], legalActions: [responseAction, alternateAction] };
    const { rerender } = render(<Table {...props} view={initial} onAction={submit} />);
    choose();
    const oldConfirm = screen.getByRole('button', { name: `确认 · ${responseAction.label}` });
    const next: View = { ...initial, version: 2,
      graveyard: change === 'projection removes target' ? [alternate]
        : change === 'same printed card replaces target' ? [{ ...target, instanceId: 'new-grave-instance' }, alternate] : initial.graveyard,
      legalActions: change === 'legal list removes target' ? [alternateAction] : initial.legalActions,
    };
    rerender(<Table {...props} view={next} onAction={submit} />);
    expect(screen.getByRole('region', { name: '点选合法目标' })).toBeInTheDocument();
    expect(screen.getByText('目标：尚未选择')).toBeInTheDocument();
    expect(screen.getByText('所选目标已失效，请重新选择目标。')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    fireEvent.click(oldConfirm);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '查看另一目标' }));
    fireEvent.click(screen.getByRole('button', { name: `确认 · ${alternateAction.label}` }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(alternateAction);
  });

  it('retains a legal region across peer updates but clears its selection if that region instance is replaced', () => {
    const submit = vi.fn();
    const region = { id: 'region-instance', index: 0, cardId: 'world', name: '纽约', threshold: 3, points: 3, influence: [0, 0], characters: [] };
    const regionAction = { ...responseAction, targetId: undefined, region: 0, label: '葬礼：发动 → 纽约' };
    const initial: View = { ...composing, regions: [region], legalActions: [regionAction] };
    const { rerender } = render(<Table {...props} view={initial} onAction={submit} />);
    fireEvent.click(screen.getByRole('button', { name: '查看葬礼' }));
    fireEvent.click(screen.getByRole('button', { name: '葬礼：发动' }));
    fireEvent.click(screen.getByRole('button', { name: '查看地区1 纽约' }));
    rerender(<Table {...props} view={{ ...initial, version: 2 }} onAction={submit} />);
    const oldConfirm = screen.getByRole('button', { name: '确认 · 葬礼：发动 → 地区 1 · 纽约' });
    expect(oldConfirm).toBeEnabled();
    rerender(<Table {...props} view={{ ...initial, version: 3, regions: [{ ...region, id: 'replacement-region' }] }} onAction={submit} />);
    expect(screen.queryByRole('button', { name: /^确认 ·/ })).not.toBeInTheDocument();
    expect(screen.getByText(/尚未选择地区/)).toBeInTheDocument();
    fireEvent.click(oldConfirm);
    expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '查看地区1 纽约' }));
    fireEvent.click(screen.getByRole('button', { name: '确认 · 葬礼：发动 → 地区 1 · 纽约' }));
    expect(submit).toHaveBeenCalledExactlyOnceWith(regionAction);
  });

  it('keeps the selection during busy or uncertain refreshes while blocking submission', () => {
    const submit = vi.fn();
    const { rerender } = render(<Table {...props} view={composing} onAction={submit} />);
    choose();
    for (const lock of [{ busy: true }, { uncertain: true }]) {
      rerender(<Table {...props} {...lock} view={{ ...composing, version: 2 }} onAction={submit} />);
      const confirm = screen.getByRole('button', { name: `确认 · ${responseAction.label}` });
      expect(confirm).toBeDisabled(); fireEvent.click(confirm);
      expect(screen.getByText('目标：队友的墓地目标')).toBeInTheDocument();
    }
    expect(submit).not.toHaveBeenCalled();
  });

  it('starts without a target draft after a full remount and never automatically submits a restored intent', () => {
    const submit = vi.fn();
    const mounted = render(<Table {...props} view={composing} onAction={submit} />);
    choose(); mounted.unmount();
    render(<Table {...props} view={{ ...composing, version: 2 }} onAction={submit} />);
    expect(screen.queryByRole('region', { name: '点选合法目标' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '取消并让过' })).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it('preserves a still-legal ordinary draft across a peer revision in the same action window', () => {
    const submit = vi.fn();
    const ordinary: View = { ...composing, stack: [], responseWindow: null };
    const { rerender } = render(<Table {...props} view={ordinary} onAction={submit} />);
    choose();
    rerender(<Table {...props} view={{ ...ordinary, version: 2 }} onAction={submit} />);
    expect(screen.getByRole('region', { name: '点选合法目标' })).toBeInTheDocument();
    expect(screen.getByText('目标：队友的墓地目标')).toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });
});
