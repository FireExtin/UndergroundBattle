// Purpose: green measurements of released behavior and command/selection boundaries.
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act, cleanup } from '@testing-library/react';
import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { testChoice } from '../../src/game/testFixtures';
import { advance, mountSynthetic, pass, playing } from './harness';

const measurements = [];
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); vi.setSystemTime(0); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });
afterAll(() => {
  writeFileSync(resolve(process.cwd(), '../docs/evidence/autopass-background-2026-10-03/observed-traces.json'), JSON.stringify({
    source: '0ec1d7643fff64b3f8c98cdac343b11fea49d10e',
    method: 'released React hook + AutoPass; jsdom fake clock; synthetic fetch only; zero RTT unless deferred',
    measurements,
  }, null, 2) + '\n');
});

describe('released background AutoPass observations', () => {
  it('discovers a new sole-pass turn at 12s and posts at 12.55s although AutoPass was enabled before discovery', async () => {
    const h = await mountSynthetic({ enabled: true });
    await advance(1);
    h.server.view = { ...playing, version: 2, legalActions: [pass] };
    await advance(11_998);
    expect(h.polls()).toHaveLength(0); expect(h.commands()).toHaveLength(0);
    await advance(1);
    expect(h.polls().map(event => event.timeMs)).toEqual([12_000]);
    expect(h.current().view.version).toBe(2);
    await advance(549); expect(h.commands()).toHaveLength(0);
    await advance(1);
    expect(h.commands()).toHaveLength(1); expect(h.commands()[0].timeMs).toBe(12_550);
    expect(h.commands()[0]).toMatchObject({ expectedVersion: 2, action: { kind: 'pass' } });
    measurements.push({ scenario: 'first discovery while last view has no legal action', serverTransitionMs: 1, discoveredMs: 12_000, commandMs: 12_550, eventToPostMs: 12_549, trace: h.trace });
  });

  it.each([
    ['background AutoPass on playing', { enabled: true }, 5, 12_000],
    ['background AutoPass off playing', {}, 5, 12_000],
    ['background room lobby AutoPass on', { enabled: true, view: { ...playing, status: 'lobby' } }, 5, 12_000],
    ['background finished AutoPass on', { enabled: true, view: { ...playing, status: 'finished' } }, 5, 12_000],
    ['foreground playing', { enabled: true, hidden: false }, 40, 1_500],
    ['public lobby without an active seat', { enabled: true, active: false }, 0, null],
  ])('measures %s over 60s, excluding the immediate initial read', async (scenario, options, count, delay) => {
    const h = await mountSynthetic(options);
    await advance(60_000);
    expect(h.polls()).toHaveLength(count); expect(h.commands()).toHaveLength(0);
    expect(h.polls().map(event => event.timeMs)).toEqual(Array.from({ length: count }, (_, i) => (i + 1) * delay));
    measurements.push({ scenario, windowMs: 60_000, periodicStateGets: count, initialStateGets: h.trace.filter(event => event.kind === 'initial-state').length, trace: h.trace });
  });

  it('resumes immediately on becoming visible and does not overlap an in-flight poll', async () => {
    let release;
    const h = await mountSynthetic({ enabled: true, poll: () => new Promise(resolve => { release = resolve; }) });
    await advance(100);
    h.visibility.mockReturnValue('visible');
    act(() => document.dispatchEvent(new Event('visibilitychange')));
    await advance(0); expect(h.polls()).toHaveLength(1);
    act(() => document.dispatchEvent(new Event('visibilitychange')));
    await advance(0); expect(h.polls()).toHaveLength(1);
    await act(async () => { release(new Response(null, { status: 204 })); });
    await advance(1_500); expect(h.polls()).toHaveLength(2);
    measurements.push({ scenario: 'visible immediate refresh, one in-flight poll', trace: h.trace });
  });

  it('keeps the command lock across background polls and sends one AutoPass command for an unchanged version', async () => {
    let release;
    const h = await mountSynthetic({ view: { ...playing, legalActions: [pass] }, enabled: true,
      command: () => new Promise(resolve => { release = resolve; }) });
    await advance(550); expect(h.commands()).toHaveLength(1); expect(h.current().busy).toBe(true);
    await act(async () => { await h.current().act({ kind: 'deploy', cardId: 'must-not-be-sent' }); });
    await advance(11_450);
    expect(h.polls()).toHaveLength(1); expect(h.commands()).toHaveLength(1);
    await act(async () => { release(h.json(h.server.view)); });
    expect(h.current().busy).toBe(false);
    await advance(24_000); expect(h.commands()).toHaveLength(1);
    measurements.push({ scenario: 'in-flight command blocks manual/new automatic command; unchanged version never duplicates', trace: h.trace });
  });

  it('reconciles lost acknowledgements using only the original command identity, version and action', async () => {
    let failing = true;
    const h = await mountSynthetic({ view: { ...playing, legalActions: [pass] }, enabled: true,
      command: (_body, server) => {
        if (failing) throw new Error('synthetic lost acknowledgement');
        return h.json({ ...server.view, version: 3, legalActions: [] });
      } });
    await advance(550);
    expect(h.commands()).toHaveLength(2); expect(h.current().uncertain).toBe(true);
    await advance(11_450);
    expect(h.commands()).toHaveLength(4); expect(h.current().uncertain).toBe(true);
    failing = false;
    await act(async () => { await h.current().act({ kind: 'deploy', cardId: 'must-not-be-sent' }); });
    expect(h.commands()).toHaveLength(5); expect(h.current().uncertain).toBe(false);
    expect(h.commands().map(({ timeMs, ...body }) => body)).toEqual(Array.from({ length: 5 }, () => {
      const { timeMs, ...body } = h.commands()[0]; return body;
    }));
    expect(localStorage.getItem('hegemony.pending.v1')).toBeNull();
    measurements.push({ scenario: 'lost acknowledgement retries, no new identity or action', trace: h.trace });
  });

  it.each([
    ['private chooser', { pendingChoice: testChoice }],
    ['public chooser', { waitingChoice: { playerId: 'p1', kind: 'target', title: '选择目标' } }],
    ['response intent window', { responseWindow: { id: 'synthetic-window', stackTopId: 'effect', holderTeam: 0, canBegin: false, members: [{ playerId: 'p0', status: 'undecided' }] } }],
  ])('cancels a queued AutoPass when a %s is accepted and keeps state synchronization active', async (scenario, blocker) => {
    const h = await mountSynthetic({ view: { ...playing, legalActions: [pass] }, enabled: true });
    await advance(300);
    h.server.view = { ...playing, version: 2, legalActions: [pass], ...blocker };
    h.visibility.mockReturnValue('visible');
    act(() => document.dispatchEvent(new Event('visibilitychange')));
    await advance(0);
    expect(h.current().view.version).toBe(2);
    await advance(2_000);
    expect(h.commands()).toHaveLength(0); expect(h.polls()).toHaveLength(2);
    measurements.push({ scenario: `${scenario} accepted at 300ms cancels old 550ms timer`, trace: h.trace });
  });
});
