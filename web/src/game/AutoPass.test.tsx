import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AutoPass } from './AutoPass';
import { testChoice, testView } from './testFixtures';
import type { View } from './types';

const pass = { id: 'pass-1', kind: 'pass', label: '让过' };
const emptyWindow: View = { ...testView, status: 'playing', legalActions: [pass] };
beforeEach(() => { localStorage.clear(); vi.useFakeTimers(); });
afterEach(() => { cleanup(); vi.useRealTimers(); localStorage.clear(); });
const advance = (ms = 550) => act(() => vi.advanceTimersByTime(ms));

describe('opt-in automatic pass', () => {
  it('defaults off, remembers opting in, submits once per version and cancels when stopped', () => {
    const submit = vi.fn();
    const { rerender } = render(<AutoPass view={emptyWindow} busy={false} uncertain={false} connection="online" onAction={submit} />);
    expect(screen.getByRole('switch', { name: '无可用行动时自动让过' })).not.toBeChecked();
    advance(1000); expect(submit).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('switch'));
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('true');
    advance(549); expect(submit).not.toHaveBeenCalled();
    advance(1); expect(submit).toHaveBeenCalledExactlyOnceWith(pass);
    rerender(<AutoPass view={{ ...emptyWindow }} busy={false} uncertain={false} connection="online" onAction={submit} />);
    advance(1100); expect(submit).toHaveBeenCalledTimes(1);
    rerender(<AutoPass view={{ ...emptyWindow, version: 2 }} busy={false} uncertain={false} connection="online" onAction={submit} />);
    advance(300); fireEvent.click(screen.getByRole('switch'));
    advance(1000); expect(submit).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem('hegemony.autoPass.v1')).toBe('false');
  });
  it('cancels pending automation for real actions, any chooser, unsafe connection or unconfirmed command', () => {
    localStorage.setItem('hegemony.autoPass.v1', 'true'); const submit = vi.fn();
    const { rerender } = render(<AutoPass view={emptyWindow} busy={false} uncertain={false} connection="online" onAction={submit} />);
    const blockers = [
      { view: { ...emptyWindow, legalActions: [pass, { id: 'asset', kind: 'asset', label: '建立资产', cardId: 'card-a' }] } },
      { view: { ...emptyWindow, pendingChoice: testChoice } },
      { view: { ...emptyWindow, waitingChoice: { playerId: 'p1', kind: 'investigation', title: '排序调查牌' } } },
      { busy: true }, { uncertain: true }, { connection: 'offline' as const },
      { view: { ...emptyWindow, status: 'finished' as const } },
    ];
    for (const blocker of blockers) {
      rerender(<AutoPass view={emptyWindow} busy={false} uncertain={false} connection="online" onAction={submit} />);
      advance(300);
      rerender(<AutoPass view={emptyWindow} busy={false} uncertain={false} connection="online" onAction={submit} {...blocker} />);
      advance(1000);
      expect(submit).not.toHaveBeenCalled();
    }
    rerender(<AutoPass view={emptyWindow} busy={false} uncertain={false} connection="online" onAction={submit} />);
    advance(); expect(submit).toHaveBeenCalledExactlyOnceWith(pass);
  });
});
