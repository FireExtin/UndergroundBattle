// Purpose: former four red criteria plus fast-poll lifecycle, identity and cancellation regressions.
import { writeFileSync } from 'node:fs';
import { act, cleanup, fireEvent, screen } from '@testing-library/react';
import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, savePending } from './api';
import { playerStorage } from './playerStorage';
import { testChoice } from './testFixtures';
import { advance, changeVisibility, json, mountSynthetic, pass, playing, session } from './AutoPassPolling.fixture';

const observations = [];
const record = (h, scenario, extra = {}) => observations.push({ scenario, ...extra, trace: h.trace });
const toggle = () => fireEvent.click(screen.getByRole('switch'));
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); vi.setSystemTime(0); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });
afterAll(() => {
  // Default tests have no file side effects. Evidence is emitted only by the explicit measurement run.
  if (process.env.HEGEMONY_AUTOPASS_TRACE_OUTPUT) writeFileSync(process.env.HEGEMONY_AUTOPASS_TRACE_OUTPUT,
    JSON.stringify({ method: 'actual useGame + controlled AutoPass, fake clock, synthetic fetch only, zero RTT except deferred cases', observations }, null, 2) + '\n');
});

describe('approved background AutoPass acceptance (former four red tests)', () => {
  it('discovers a turn within 3s even when the last accepted view is not our turn, then waits the existing 550ms', async () => {
    const h = await mountSynthetic({ enabled: true });
    await advance(1); h.server.view = { ...playing, version: 2, legalActions: [pass] };
    await advance(2_999);
    expect(h.polls().map(event => event.timeMs)).toEqual([3_000]);
    expect(h.current().view.version).toBe(2);
    await advance(549); expect(h.commands()).toHaveLength(0);
    await advance(1); expect(h.commands()).toHaveLength(1);
    expect(h.commands()[0]).toMatchObject({ timeMs: 3_550, expectedVersion: 2, action: { kind: 'pass' } });
    record(h, 'first discovery while stale view is not our turn', { serverTransitionMs: 1, discoveredMs: 3_000, commandMs: 3_550, eventToPostMs: 3_549 });
  });
  it('rearms a sleeping 12s timer when the user turns AutoPass on without replacing the polling lifecycle', async () => {
    const h = await mountSynthetic();
    expect(screen.getByRole('switch')).not.toBeChecked();
    await advance(1_000); toggle();
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('true');
    await advance(3_000);
    expect(h.polls().map(event => event.timeMs)).toEqual([4_000]);
    expect(h.initialReads()).toHaveLength(1); expect(h.commands()).toHaveLength(0);
    record(h, 'enable at 1000ms, rearm healthy idle timer to 4000ms');
  });
  it('keeps 3s synchronization while waiting for another seat chooser but sends no automatic command', async () => {
    const h = await mountSynthetic({ enabled: true, view: { ...playing, legalActions: [pass],
      waitingChoice: { playerId: 'p1', kind: 'target', title: '选择目标' } } });
    await advance(3_000);
    expect(h.polls().map(event => event.timeMs)).toEqual([3_000]); expect(h.commands()).toHaveLength(0);
    record(h, 'other-seat chooser continues fast sync, no automatic decision');
  });
  it('uses the in-memory opt-in when preference persistence fails instead of rereading storage to choose 12s', async () => {
    const h = await mountSynthetic(); const original = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (key, value) {
      if (key === 'hegemony.autoPass.v1') throw new Error('synthetic preference storage failure');
      return original.call(this, key, value);
    });
    toggle(); expect(screen.getByRole('switch')).toBeChecked();
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('false');
    await advance(3_000);
    expect(h.polls().map(event => event.timeMs)).toEqual([3_000]);
    record(h, 'storage write failure retains live opt-in');
  });
});

describe('fast polling policy and lifecycle', () => {
  it.each([
    ['background playing AutoPass on (including repeated 204)', { enabled: true }, 20, 3_000],
    ['background playing AutoPass off', {}, 5, 12_000],
    ['background room lobby AutoPass on', { enabled: true, view: { ...playing, status: 'lobby' } }, 5, 12_000],
    ['background finished AutoPass on', { enabled: true, view: { ...playing, status: 'finished' } }, 5, 12_000],
    ['foreground playing AutoPass on', { enabled: true, hidden: false }, 40, 1_500],
    ['public lobby without an active seat', { enabled: true, active: false }, 0, null],
  ])('measures %s for 60s', async (scenario, options, count, delay) => {
    const h = await mountSynthetic(options); await advance(60_000);
    expect(h.polls().map(event => event.timeMs)).toEqual(Array.from({ length: count }, (_, i) => (i + 1) * delay));
    expect(h.initialReads()).toHaveLength(options.active === false ? 0 : 1); expect(h.commands()).toHaveLength(0);
    record(h, scenario, { periodicGetsIn60s: count });
  });
  it('stops a queued 550ms pass and returns to 12s when switched off', async () => {
    const h = await mountSynthetic({ enabled: true, view: { ...playing, legalActions: [pass] } });
    await advance(300); toggle();
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('false');
    await advance(11_999); expect(h.polls()).toHaveLength(0); expect(h.commands()).toHaveLength(0);
    await advance(1); expect(h.polls().map(e => e.timeMs)).toEqual([12_300]);
    expect(h.commands()).toHaveLength(0); record(h, 'off at 300ms cancels pass, next poll 12300ms');
  });
  it('uses 12s after a poll accepts finished, including subsequent 204 responses', async () => {
    const h = await mountSynthetic({ enabled: true });
    h.server.view = { ...playing, version: 2, status: 'finished', legalActions: [pass] };
    await advance(3_000); expect(h.current().view.status).toBe('finished');
    await advance(11_999); expect(h.polls()).toHaveLength(1);
    await advance(1); await advance(12_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([3_000, 15_000, 27_000]);
    expect(h.commands()).toHaveLength(0); record(h, 'finished accepted at 3s downgrades to 12s');
  });
  it('rearms an idle healthy timer when a command acknowledgement finishes the game', async () => {
    const h = await mountSynthetic({ enabled: true, command: (_body, server) => json({ ...server.view, version: 2, status: 'finished' }) });
    await advance(1_000); await act(async () => { await h.current().act({ kind: 'pass' }); });
    await advance(11_999); expect(h.polls()).toHaveLength(0);
    await advance(1); expect(h.polls().map(e => e.timeMs)).toEqual([13_000]);
    expect(h.current().view.status).toBe('finished'); record(h, 'finished command ACK at 1s rearms to 13s');
  });
  it('does not let an older playing view restore fast polling after finished', async () => {
    const h = await mountSynthetic({ enabled: true, view: { ...playing, version: 10, status: 'finished' },
      read: ({ afterVersion }) => afterVersion === null ? undefined : json({ ...playing, version: 9 }) });
    await advance(12_000); expect(h.current().view).toMatchObject({ version: 10, status: 'finished' });
    await advance(12_000);
    expect(h.polls().map(e => [e.timeMs, e.afterVersion])).toEqual([[12_000, 10], [24_000, 10]]);
    record(h, 'older playing poll never changes finished policy');
  });
  it('keeps the foreground 1.5s deadline across preference changes', async () => {
    const h = await mountSynthetic({ hidden: false });
    await advance(1_000); toggle(); await advance(500);
    await advance(500); toggle(); await advance(1_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([1_500, 3_000]);
    expect(h.initialReads()).toHaveLength(1); record(h, 'foreground toggles do not shift 1.5s cadence');
  });
  it('keeps the last foreground timer when hidden and resumes immediately when visible', async () => {
    const h = await mountSynthetic({ enabled: true, hidden: false });
    await advance(1_000); changeVisibility(h, false); await advance(500); await advance(3_000);
    await advance(500); changeVisibility(h, true); await advance(0); await advance(1_500);
    expect(h.polls().map(e => e.timeMs)).toEqual([1_500, 4_500, 5_000, 6_500]);
    record(h, 'visibility lifecycle unchanged, hidden next interval 3s');
  });
  it('does not rebuild the polling effect for each accepted version', async () => {
    const h = await mountSynthetic({ enabled: true, read: ({ afterVersion }, server) => {
      if (afterVersion === null) return undefined;
      server.view = { ...server.view, version: server.view.version + 1 }; return json(server.view);
    } });
    await advance(60_000);
    expect(h.polls()).toHaveLength(20); expect(h.initialReads()).toHaveLength(1);
    expect(h.polls().map(e => e.afterVersion)).toEqual(Array.from({ length: 20 }, (_, i) => i + 1));
    expect(h.current().view.version).toBe(21); record(h, '20 new versions in 60s retain one polling lifecycle');
  });
  it('discovers playing from a stale lobby at 12s, then uses 3s', async () => {
    const h = await mountSynthetic({ enabled: true, view: { ...playing, status: 'lobby' } });
    h.server.view = { ...playing, version: 2 };
    await advance(12_000); await advance(3_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([12_000, 15_000]); record(h, 'lobby-to-start limitation retained');
  });
  it.each([true, false])('does not overlap an in-flight GET; completion uses the latest opt-in %s', async enabledAtCompletion => {
    let release;
    const h = await mountSynthetic({ read: ({ afterVersion }) => afterVersion === null ? undefined : new Promise(resolve => { release = resolve; }) });
    await advance(12_000); await advance(1_000); toggle();
    if (!enabledAtCompletion) { await advance(500); toggle(); await advance(500); } else await advance(1_000);
    changeVisibility(h, true); await advance(0); changeVisibility(h, false);
    expect(h.polls()).toHaveLength(1); expect(h.initialReads()).toHaveLength(1);
    const inFlight = h.requests.find(r => r.url.includes('?afterVersion='));
    expect(inFlight.init.signal.aborted).toBe(false);
    await act(async () => { release(new Response(null, { status: 204 })); });
    const delay = enabledAtCompletion ? 3_000 : 12_000;
    await advance(delay - 1); expect(h.polls()).toHaveLength(1);
    await advance(1); expect(h.polls().map(e => e.timeMs)).toEqual([12_000, 14_000 + delay]);
    record(h, `in-flight GET completed at 14s uses latest opt-in ${enabledAtCompletion}`);
  });
  it('preserves 2/4/8/15s offline backoff across preference changes, then resumes 3s after recovery', async () => {
    let failing = true;
    const h = await mountSynthetic({ enabled: true, read: ({ afterVersion }) => afterVersion !== null && failing ? json({ message: 'synthetic offline' }, 503) : undefined });
    await advance(3_000); expect(h.current().connection).toBe('offline');
    await advance(1_000); toggle(); await advance(1_000);
    await advance(1_000); toggle(); await advance(3_000); await advance(8_000); await advance(15_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([3_000, 5_000, 9_000, 17_000, 32_000]);
    failing = false; await advance(15_000); await advance(3_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([3_000, 5_000, 9_000, 17_000, 32_000, 47_000, 50_000]);
    expect(h.current().connection).toBe('online'); record(h, 'offline backoff preserved, recovery returns to 3s');
  });
});

describe('fast polling identity, commands and chooser boundaries', () => {
  it('keeps player namespaces isolated, restores ordinary preference and reloads without new identity', async () => {
    const other = { ...session, seat: 1, token: 'synthetic-independent-token' };
    const h = await mountSynthetic({ enabled: true, read: ({ init }) => {
      if (new Headers(init.headers).get('Authorization') === `Bearer ${other.token}`) return json({ ...playing, you: 'p1' });
    } });
    await act(async () => { h.current().startIndependentSession(); });
    expect(h.current().session).toBeNull(); expect(h.current().autoPassEnabled).toBe(false);
    const scope = playerStorage().scope;
    await act(async () => { h.current().resume(other); });
    expect(screen.getByRole('switch')).not.toBeChecked(); toggle();
    expect(sessionStorage.getItem(`hegemony.player.${scope}.hegemony.autoPass.v1`)).toBe('true');
    toggle(); expect(playerStorage().getItem('hegemony.autoPass.v1')).toBe('false');
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('true');
    await h.remount(); expect(h.current().session.seat).toBe(1); expect(h.current().autoPassEnabled).toBe(false);
    await advance(12_000); expect(h.polls().at(-1).timeMs).toBe(12_000);
    await act(async () => { h.current().useOrdinarySession(); });
    expect(h.current().session.seat).toBe(0); expect(screen.getByRole('switch')).toBeChecked();
    await advance(3_000);
    expect(h.polls().at(-1)).toMatchObject({ timeMs: 15_000, authorization: `Bearer ${session.token}` });
    expect(h.commands()).toHaveLength(0); record(h, 'namespace switch, scoped preference and refresh identity');
  });
  it('ignores other-tab storage changes until a reload instead of using storage as a second live preference', async () => {
    const h = await mountSynthetic({ enabled: true });
    localStorage.setItem('hegemony.autoPass.v1', 'false');
    act(() => window.dispatchEvent(new StorageEvent('storage', { key: 'hegemony.autoPass.v1', newValue: 'false', storageArea: localStorage })));
    await advance(3_000); expect(h.current().autoPassEnabled).toBe(true); expect(screen.getByRole('switch')).toBeChecked();
    await h.remount(); expect(h.current().autoPassEnabled).toBe(false);
    await advance(12_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([3_000, 15_000]); record(h, 'single live memory source, persisted off read on reload');
  });
  it('cleans fast timers on leave and ignores a late state fetch even when fetch ignores abort', async () => {
    let release;
    const h = await mountSynthetic({ enabled: true, read: ({ afterVersion }) => afterVersion === null ? undefined : new Promise(resolve => { release = resolve; }) });
    await advance(3_000); await act(async () => { h.current().leave(); });
    expect(h.requests.find(r => r.url.includes('?afterVersion=')).init.signal.aborted).toBe(true);
    await act(async () => { release(json({ ...playing, version: 99, legalActions: [pass] })); });
    await advance(60_000);
    expect(h.current().session).toBeNull(); expect(h.current().view).toBeNull();
    expect(h.polls()).toHaveLength(1); expect(h.commands()).toHaveLength(0); record(h, 'leave clears fast loop and late state');
  });
  it.each([200, 401])('rejects an old same-room seat response (%s) after switching to another seat/lobby', async status => {
    let release; const other = { ...session, seat: 1, token: 'synthetic-other-seat-token' };
    const lobby = { ...playing, you: 'p1', version: 10, status: 'lobby' };
    const h = await mountSynthetic({ enabled: true, read: ({ init, afterVersion }) => {
      if (new Headers(init.headers).get('Authorization') === `Bearer ${other.token}`) return afterVersion === null ? json(lobby) : new Response(null, { status: 204 });
      if (afterVersion !== null) return new Promise(resolve => { release = resolve; });
    } });
    await advance(3_000); await act(async () => { h.current().leave(); h.current().resume(other); });
    await act(async () => { release(status === 200 ? json({ ...playing, version: 99, legalActions: [pass] }) : json({ error: 'invalid_seat_token' }, 401)); });
    expect(h.current().view).toMatchObject({ version: 10, you: 'p1', status: 'lobby' });
    await advance(11_999); expect(h.polls()).toHaveLength(1);
    await advance(1);
    expect(h.polls().at(-1)).toMatchObject({ timeMs: 15_000, afterVersion: 10, authorization: `Bearer ${other.token}` });
    expect(h.commands()).toHaveLength(0); record(h, `late old seat ${status} cannot contaminate new lobby or token`);
  });
  it('rejects an aborted old response even after returning to the same seat token (ABA)', async () => {
    let release; let first = true;
    const h = await mountSynthetic({ enabled: true, read: ({ afterVersion }) => {
      if (afterVersion !== null && first) { first = false; return new Promise(resolve => { release = resolve; }); }
    } });
    await advance(3_000); h.server.view = { ...playing, version: 3 };
    await act(async () => { h.current().leave(); h.current().resume(session); });
    await act(async () => { release(json({ ...playing, version: 99, legalActions: [pass] })); });
    await advance(3_000);
    expect(h.current().view.version).toBe(3); expect(h.polls().map(e => [e.timeMs, e.afterVersion])).toEqual([[3_000, 1], [6_000, 3]]);
    expect(h.commands()).toHaveLength(0); record(h, 'same-token ABA still rejects old aborted effect');
  });
  it('clears the last view across player-mode switches and rejects a late initial GET after ordinary-scope ABA', async () => {
    let release; let first = true;
    const h = await mountSynthetic({ enabled: true, ready: false, read: ({ afterVersion }) => {
      if (afterVersion === null && first) { first = false; return new Promise(resolve => { release = resolve; }); }
    } });
    expect(h.current().view).toBeNull();
    await advance(1_000); await act(async () => { h.current().startIndependentSession(); });
    expect(h.current().autoPassEnabled).toBe(false); expect(h.current().session).toBeNull();
    await advance(1_000); h.server.view = { ...playing, version: 3, status: 'lobby' };
    await act(async () => { h.current().useOrdinarySession(); });
    await act(async () => { release(json({ ...playing, version: 99, legalActions: [pass] })); });
    expect(h.current().autoPassEnabled).toBe(true);
    expect(h.current().view).toMatchObject({ version: 3, status: 'lobby' });
    await advance(11_999); expect(h.polls()).toHaveLength(0);
    await advance(1);
    expect(h.polls().map(e => [e.timeMs, e.afterVersion])).toEqual([[14_000, 3]]);
    expect(h.commands()).toHaveLength(0); record(h, 'player-mode initial GET ABA preserves newly accepted lobby and 12s policy');
  });
  it('holds the POST lock across 3s reads and ignores an older playing ACK after a newer finished poll', async () => {
    let release;
    const h = await mountSynthetic({ enabled: true, view: { ...playing, legalActions: [pass] }, command: () => new Promise(resolve => { release = resolve; }) });
    await advance(550); expect(h.current().busy).toBe(true);
    h.server.view = { ...playing, version: 3, status: 'finished' };
    await advance(2_450);
    await act(async () => { await h.current().act({ kind: 'deploy', cardId: 'must-not-be-sent' }); });
    expect(h.commands()).toHaveLength(1);
    await act(async () => { release(json({ ...playing, version: 2, legalActions: [pass] })); });
    expect(h.current().view).toMatchObject({ version: 3, status: 'finished' }); expect(h.current().busy).toBe(false);
    await advance(12_000);
    expect(h.polls().map(e => e.timeMs)).toEqual([3_000, 15_000]); expect(h.commands()).toHaveLength(1);
    record(h, 'POST lock and late ACK cannot revive earlier playing state');
  });
  it('never duplicates an unchanged pass version across fast 204 polls, but permits a new authoritative version', async () => {
    const h = await mountSynthetic({ enabled: true, view: { ...playing, legalActions: [pass] }, command: (_body, server) => json(server.view) });
    await advance(550); await advance(11_450); expect(h.commands()).toHaveLength(1);
    h.server.view = { ...playing, version: 2, legalActions: [pass] };
    await advance(3_000); await advance(550);
    expect(h.commands()).toHaveLength(2);
    expect(h.commands().map(e => [e.timeMs, e.expectedVersion])).toEqual([[550, 1], [15_550, 2]]);
    expect(h.commands()[0].commandId).not.toBe(h.commands()[1].commandId);
    await advance(6_000); expect(h.commands()).toHaveLength(2); record(h, 'one automatic command per authoritative pass version');
  });
  it.each([true, false])('retries lost ACKs with original actor/id/version/paced action; manual recovery %s', async manualRecovery => {
    let failing = true;
    const h = await mountSynthetic({ enabled: true, view: { ...playing, serverNowMs: 0, legalActions: [pass] }, command: (_body, server) => {
      if (failing) throw new Error('synthetic lost ACK'); return json(server.view);
    } });
    await advance(550); expect(h.commands()).toHaveLength(2); expect(h.current().uncertain).toBe(true);
    await advance(2_450); expect(h.commands()).toHaveLength(4);
    failing = false; h.server.view = { ...playing, version: 3, serverNowMs: 6_000 };
    if (manualRecovery) await act(async () => { await h.current().act({ kind: 'deploy', cardId: 'must-not-be-sent' }); });
    else await advance(3_000);
    expect(h.commands()).toHaveLength(5); expect(h.current().uncertain).toBe(false);
    const bodies = h.commands().map(({ timeMs, ...body }) => body);
    expect(bodies).toEqual(Array.from({ length: 5 }, () => bodies[0]));
    expect(bodies[0]).toMatchObject({ authorization: `Bearer ${session.token}`, expectedVersion: 1, action: { kind: 'game', action: { kind: 'pass' } } });
    expect(readPending()).toBeNull();
    await advance(3_000); expect(h.commands()).toHaveLength(5); record(h, `3s lost ACK reconciliation preserves original paced command, manual recovery ${manualRecovery}`);
  });
  it('restores a persisted original command on reload without regressing the newer view or opt-in', async () => {
    savePending({ roomId: session.roomId, seat: 0, commandId: 'synthetic-original-before-reload', expectedVersion: 1, action: { kind: 'pass' } });
    const h = await mountSynthetic({ enabled: true, view: { ...playing, version: 5 }, command: () => json({ ...playing, version: 2 }) });
    expect(h.commands()).toHaveLength(1);
    expect(h.commands()[0]).toMatchObject({ commandId: 'synthetic-original-before-reload', expectedVersion: 1 });
    expect(h.current().view.version).toBe(5); expect(h.current().uncertain).toBe(false);
    await advance(3_000); expect(h.polls().at(-1).afterVersion).toBe(5);
    record(h, 'persisted original receipt reconciles before first fast sleep');
  });
  it.each([
    ['private choice', { pendingChoice: testChoice }],
    ['public choice', { waitingChoice: { playerId: 'p1', kind: 'target', title: '选择目标' } }],
    ...['undecided', 'composing', 'passed'].map(status => [`response ${status}`, { responseWindow: { id: 'synthetic-window', stackTopId: 'effect', holderTeam: 0, canBegin: false, members: [{ playerId: 'p0', status }] } }]),
  ])('a natural fast poll cancels the 550ms timer when %s arrives', async (scenario, blocker) => {
    const h = await mountSynthetic({ enabled: true,
      view: { ...playing, legalActions: [pass, { id: 'synthetic-asset', kind: 'asset', label: '手动建立资产' }] },
      command: (_body, server) => { server.view = { ...playing, version: 2, legalActions: [pass] }; return json(server.view); } });
    await advance(2_800); await act(async () => { await h.current().act(pass); });
    h.server.view = { ...playing, version: 3, legalActions: [pass], ...blocker };
    await advance(200); expect(h.current().view.version).toBe(3);
    await advance(6_000);
    expect(h.commands()).toHaveLength(1); expect(h.commands()[0].timeMs).toBe(2_800);
    expect(h.polls().map(e => e.timeMs)).toEqual([3_000, 6_000, 9_000]);
    record(h, `${scenario} accepted at 3s cancels pass queued at 2.8s, keeps 3s sync`);
  });
});
