import { render, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Table } from './Table';
import { testCatalog, testView } from './testFixtures';
import type { View } from './types';

// Display-only synthetic projections, never the parent's private room state.
// An observer has no matching player, hand or legal action; no spectator API
// or session is created by these component tests.
const perspectives = [
  { name: 'winning host', you: 'p0', victory: '你的秘社取得了霸权' },
  { name: 'other player', you: 'p1', victory: 'A赢得了霸权' },
  { name: 'read-only observer', you: 'observer', victory: 'A赢得了霸权' },
];
const finishedView = (you: string): View => ({
  ...testView, status: 'finished', version: 20, you, turn: 6,
  winnerTeam: 0, winScore: 9, priorityTeam: 0,
  hand: [], legalActions: you === 'p0' ? [{ id: 'restart', kind: 'restart', label: '再开一局' }] : [],
  players: [
    { ...testView.players[0], id: 'p0', name: 'A', team: 0, score: 9 },
    { ...testView.players[0], id: 'p1', seat: 1, name: 'B', team: 1, score: 4 },
  ],
});
const playingView = (you: string): View => ({
  ...finishedView(you), status: 'playing', version: 19, winnerTeam: undefined,
  players: finishedView(you).players.map(p => ({ ...p, score: p.team === 0 ? 8 : 4 })),
  legalActions: you === 'p0' ? [{ id: 'pass', kind: 'pass', label: '让过' }] : [],
});
const props = { catalog: testCatalog, busy: false, connection: 'online' as const };
const summary = (container: HTMLElement) => container.querySelector('.hg-phase-line > strong');
function expectFinished(container: HTMLElement, victory: string) {
  expect(summary(container)).toHaveTextContent('对局已结束');
  expect(summary(container)).not.toHaveTextContent(/轮到|等待.*行动/);
  expect(within(container.querySelector<HTMLElement>('.hg-victory')!).getByRole('heading', { level: 1 })).toHaveTextContent(victory);
  expect([...container.querySelectorAll('.hg-team-tally b')].map(n => n.textContent).sort()).toEqual(['4', '9']);
}

describe('finished table summary', () => {
  it.each(perspectives)('$name sees an ended summary despite the retained priority', ({ you, victory }) => {
    const submit = vi.fn();
    const { container } = render(<Table {...props} view={finishedView(you)} onAction={submit} />);
    expectFinished(container, victory);
    if (you === 'p0') expect(within(container).getByRole('button', { name: '再开一局' })).toBeEnabled();
    else expect(within(container).queryByRole('button', { name: '再开一局' })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });

  it.each(perspectives)('$name updates at game end and restores the same summary on a fresh mount', ({ you, victory }) => {
    const submit = vi.fn();
    const first = render(<Table {...props} view={playingView(you)} onAction={submit} />);
    expect(summary(first.container)).not.toHaveTextContent('对局已结束');
    first.rerender(<Table {...props} view={finishedView(you)} onAction={submit} />);
    expectFinished(first.container, victory);
    first.unmount();
    // A new component receiving the saved JSON projection models refresh display.
    // It does not claim browser seat persistence or repeat the parent's playtest.
    const restored = JSON.parse(JSON.stringify(finishedView(you))) as View;
    const fresh = render(<Table {...props} view={restored} onAction={submit} />);
    expectFinished(fresh.container, victory);
    expect(submit).not.toHaveBeenCalled();
  });

  it.each([
    { you: 'p0', expected: '轮到你参与行动' },
    { you: 'p1', expected: '等待 A 行动' },
  ])('keeps the playing summary for $you', ({ you, expected }) => {
    const { container } = render(<Table {...props} view={playingView(you)} onAction={vi.fn()} />);
    expect(summary(container)).toHaveTextContent(expected);
    expect(container.querySelector('.hg-victory')).toBeNull();
  });
});
