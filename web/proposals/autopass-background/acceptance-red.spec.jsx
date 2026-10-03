// Purpose: deliberately red requirements for the reviewed proposal, never default/release tests.
// These assertions target existing production components; no simulated policy implements them.
import { cleanup, fireEvent, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { advance, mountSynthetic, pass, playing } from './harness';

beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); vi.setSystemTime(0); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('PROPOSAL ONLY: background 3s when AutoPass enabled and playing', () => {
  it('discovers a turn within 3s even when the last accepted view is not our turn, then waits the existing 550ms', async () => {
    const h = await mountSynthetic({ enabled: true });
    await advance(1);
    h.server.view = { ...playing, version: 2, legalActions: [pass] };
    await advance(2_999);
    expect(h.polls().map(event => event.timeMs)).toEqual([3_000]);
    expect(h.current().view.version).toBe(2);
    await advance(549); expect(h.commands()).toHaveLength(0);
    await advance(1); expect(h.commands()).toHaveLength(1);
    expect(h.commands()[0]).toMatchObject({ timeMs: 3_550, expectedVersion: 2, action: { kind: 'pass' } });
  });

  it('rearms a sleeping 12s timer when the user turns AutoPass on without replacing the polling lifecycle', async () => {
    const h = await mountSynthetic();
    await advance(1_000);
    fireEvent.click(screen.getByRole('switch'));
    await advance(3_000);
    expect(h.polls().map(event => event.timeMs)).toEqual([4_000]);
    expect(h.trace.filter(event => event.kind === 'initial-state')).toHaveLength(1);
    expect(h.commands()).toHaveLength(0);
  });

  it('keeps 3s synchronization while waiting for another seat chooser but sends no automatic command', async () => {
    const h = await mountSynthetic({ enabled: true, view: { ...playing,
      legalActions: [pass], waitingChoice: { playerId: 'p1', kind: 'target', title: '选择目标' } } });
    await advance(3_000);
    expect(h.polls().map(event => event.timeMs)).toEqual([3_000]);
    expect(h.commands()).toHaveLength(0);
  });

  it('uses the in-memory opt-in when preference persistence fails instead of rereading storage to choose 12s', async () => {
    const h = await mountSynthetic();
    const originalSetItem = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (key, value) {
      if (key === 'hegemony.autoPass.v1') throw new Error('synthetic preference storage failure');
      return originalSetItem.call(this, key, value);
    });
    fireEvent.click(screen.getByRole('switch'));
    expect(screen.getByRole('switch')).toBeChecked();
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('false');
    await advance(3_000);
    expect(h.polls().map(event => event.timeMs)).toEqual([3_000]);
  });
});
