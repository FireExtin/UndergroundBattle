import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readActiveSession, saveSession } from './api';
import { selectPlayerMode } from './playerStorage';
import type { SavedSession } from './types';
import { useSeatConflict } from './useSeatConflict';

const channelName = 'hegemony.seatPresence.v1';
const seat: SavedSession = { roomId: 'room-a', seat: 0, token: 'private-seat-token', inviteCode: 'INVITE' };
type HookProps = { session: SavedSession | null };
const hook = ({ session }: HookProps) => useSeatConflict(session);
const active = (session: SavedSession = seat): HookProps => ({ session });
const savedIn = (namespace: string | null, session: SavedSession = seat): HookProps => {
  selectPlayerMode(namespace ? 'independent' : 'ordinary', namespace || undefined);
  saveSession(session);
  return { session: readActiveSession() };
};
const flush = () => act(async () => {});

// Each instance has its own listeners, as separate browsing contexts do. The
// sender never receives its own message, and close does not retract queued sends.
class TestBroadcastChannel {
  static instances: TestBroadcastChannel[] = [];
  readonly messages: unknown[] = [];
  readonly listeners = new Set<(event: MessageEvent) => void>();
  closed = false;
  suspended = false;
  constructor(readonly name: string) { TestBroadcastChannel.instances.push(this); }
  addEventListener(_type: string, listener: (event: MessageEvent) => void) { this.listeners.add(listener); }
  postMessage(data: unknown) {
    if (this.closed) throw new Error('closed channel');
    this.messages.push(structuredClone(data));
    if (this.suspended) return;
    for (const peer of TestBroadcastChannel.instances) {
      if (peer === this || peer.name !== this.name) continue;
      queueMicrotask(() => { if (!peer.closed && !peer.suspended) peer.receive(data); });
    }
  }
  receive(data: unknown) { for (const listener of this.listeners) listener(new MessageEvent('message', { data })); }
  close() { this.closed = true; this.listeners.clear(); }
}

beforeEach(() => {
  vi.useFakeTimers(); vi.setSystemTime(new Date('2026-10-15T00:00:00Z'));
  localStorage.clear(); sessionStorage.clear();
  TestBroadcastChannel.instances = [];
  vi.stubGlobal('BroadcastChannel', TestBroadcastChannel);
});
afterEach(() => {
  cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals();
  localStorage.clear(); sessionStorage.clear();
});

describe('same-seat tab presence', () => {
  it('discovers both tabs immediately and clears the survivor when the other closes, without broadcasting credentials', async () => {
    const first = renderHook(hook, { initialProps: active() }); await flush();
    expect(first.result.current).toBe(false);
    const second = renderHook(hook, { initialProps: active({ ...seat, token: 'another-local-token' }) }); await flush();
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
    const channels = TestBroadcastChannel.instances;
    expect(channels).toHaveLength(2);
    expect(channels.map(channel => channel.name)).toEqual([channelName, channelName]);
    const messages = channels.flatMap(channel => channel.messages);
    expect(new Set(messages.map(value => (value as { tabId: string }).tabId)).size).toBe(2);
    for (const message of messages) expect(Object.keys(message as object).sort()).toEqual(['kind', 'roomId', 'seat', 'tabId']);
    expect(JSON.stringify(messages)).not.toContain('token');
    expect(localStorage.length).toBe(0); expect(sessionStorage.length).toBe(0);
    second.unmount(); await flush();
    expect(first.result.current).toBe(false); expect(channels[1].closed).toBe(true);
    first.unmount(); expect(vi.getTimerCount()).toBe(0);
  });

  it('keeps the warning until every other matching page has left', async () => {
    const first = renderHook(hook, { initialProps: active() });
    const second = renderHook(hook, { initialProps: active() });
    const third = renderHook(hook, { initialProps: active() }); await flush();
    expect([first.result.current, second.result.current, third.result.current]).toEqual([true, true, true]);
    second.unmount(); await flush(); expect(first.result.current).toBe(true);
    third.unmount(); await flush(); expect(first.result.current).toBe(false);
  });

  it('does not treat another room, another seat, or the lobby as a conflict', async () => {
    const first = renderHook(hook, { initialProps: active() });
    const otherRoom = renderHook(hook, { initialProps: active({ ...seat, roomId: 'room-b' }) });
    const otherSeat = renderHook(hook, { initialProps: active({ ...seat, seat: 1 }) });
    const lobby = renderHook(hook, { initialProps: { session: null } }); await flush();
    expect([first, otherRoom, otherSeat, lobby].map(result => result.result.current)).toEqual([false, false, false, false]);
    expect(TestBroadcastChannel.instances).toHaveLength(3);
  });

  it.each([
    { name: 'ordinary and independent sessions', first: null, second: 'independent-session-a' },
    { name: 'duplicated independent sessions', first: 'independent-session-a', second: 'independent-session-a' },
    { name: 'different independent namespaces', first: 'independent-session-a', second: 'independent-session-b' },
  ])('detects a shared seat across $name', async scope => {
    const first = renderHook(hook, { initialProps: savedIn(scope.first) }); await flush();
    const second = renderHook(hook, { initialProps: savedIn(scope.second) }); await flush();
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
    const messages = TestBroadcastChannel.instances.flatMap(channel => channel.messages);
    expect(new Set(messages.map(value => (value as { tabId: string }).tabId)).size).toBe(2);
    expect(JSON.stringify(messages)).not.toContain('independent-session');
    expect(JSON.stringify(messages)).not.toContain(seat.token);
    second.unmount(); await flush(); expect(first.result.current).toBe(false);
  });

  it('keeps independent players with different seats or rooms separate', async () => {
    const first = renderHook(hook, { initialProps: savedIn('independent-session-a') });
    const otherSeat = renderHook(hook, { initialProps: savedIn('independent-session-b', { ...seat, seat: 1 }) });
    const otherRoom = renderHook(hook, { initialProps: savedIn('independent-session-c', { ...seat, roomId: 'room-b' }) }); await flush();
    expect([first, otherSeat, otherRoom].map(result => result.result.current)).toEqual([false, false, false]);
    expect(TestBroadcastChannel.instances).toHaveLength(3);
  });

  it('leaves the old identity before switching rooms or seats and rediscovers a restored seat', async () => {
    const first = renderHook(hook, { initialProps: active() });
    const second = renderHook(hook, { initialProps: active() }); await flush();
    expect(first.result.current).toBe(true);
    first.rerender(active({ ...seat, roomId: 'room-b' })); await flush();
    expect(first.result.current).toBe(false); expect(second.result.current).toBe(false);
    second.rerender(active({ ...seat, roomId: 'room-b' })); await flush();
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
    first.rerender(active({ ...seat, roomId: 'room-b', seat: 1 })); await flush();
    expect(first.result.current).toBe(false); expect(second.result.current).toBe(false);
    first.rerender(active({ ...seat, roomId: 'room-b' })); await flush();
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
    first.rerender({ session: null }); await flush();
    expect(first.result.current).toBe(false); expect(second.result.current).toBe(false);
  });

  it('ignores malformed, self, and unrelated identity messages', async () => {
    const first = renderHook(hook, { initialProps: active() }); await flush();
    const channel = TestBroadcastChannel.instances[0];
    const own = channel.messages[0] as Record<string, unknown>;
    const valid = { kind: 'hello', tabId: 'other-tab', roomId: seat.roomId, seat: seat.seat };
    act(() => {
      for (const message of [null, 'bad', {}, own, { ...valid, kind: 'unknown' }, { ...valid, tabId: '' },
        { ...valid, seat: -1 }, { ...valid, seat: 0.5 }, { ...valid, seat: '0' },
        { ...valid, roomId: 'room-b' }, { ...valid, seat: 1 }]) channel.receive(message);
    });
    expect(first.result.current).toBe(false); expect(channel.messages).toHaveLength(1);
  });

  it('retains hidden peers across minute-long timer throttling, expires suspended pages, and rediscovers them on return', async () => {
    const visibility = vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('hidden');
    const first = renderHook(hook, { initialProps: active() });
    const second = renderHook(hook, { initialProps: active() }); await flush();
    const paused = TestBroadcastChannel.instances[1];
    paused.suspended = true;
    await act(async () => { await vi.advanceTimersByTimeAsync(75_000); });
    expect(first.result.current).toBe(true);
    await act(async () => { await vi.advanceTimersByTimeAsync(60_000); });
    expect(first.result.current).toBe(false);
    paused.suspended = false; visibility.mockReturnValue('visible');
    await act(async () => { document.dispatchEvent(new Event('visibilitychange')); });
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
  });

  it('keeps live hidden peers present over repeated heartbeats', async () => {
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('hidden');
    const first = renderHook(hook, { initialProps: active() });
    const second = renderHook(hook, { initialProps: active() }); await flush();
    await act(async () => { await vi.advanceTimersByTimeAsync(300_000); });
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
  });

  it('withdraws on pagehide and rejoins on pageshow after a back-forward cache restore', async () => {
    const first = renderHook(hook, { initialProps: active() });
    const second = renderHook(hook, { initialProps: active() }); await flush();
    await act(async () => { window.dispatchEvent(new Event('pagehide')); });
    expect(first.result.current).toBe(false); expect(second.result.current).toBe(false);
    expect(TestBroadcastChannel.instances.every(channel => (channel.messages.at(-1) as { kind: string }).kind === 'bye')).toBe(true);
    await act(async () => { window.dispatchEvent(new Event('pageshow')); });
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
  });
});

describe('storage-event fallback', () => {
  it.each(['missing', 'denied'])('discovers and clears peers when BroadcastChannel is %s, leaving no persistent record', async failure => {
    vi.stubGlobal('BroadcastChannel', failure === 'missing' ? undefined : class { constructor() { throw new DOMException('denied', 'SecurityError'); } });
    const original = Storage.prototype.setItem;
    const published: string[] = [];
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
      original.call(this, key, value);
      if (this === localStorage && key === channelName) {
        published.push(value);
        queueMicrotask(() => window.dispatchEvent(new StorageEvent('storage', { key, newValue: value, storageArea: localStorage })));
      }
    });
    const first = renderHook(hook, { initialProps: active() }); await flush();
    const second = renderHook(hook, { initialProps: active() }); await flush();
    expect(first.result.current).toBe(true); expect(second.result.current).toBe(true);
    expect(TestBroadcastChannel.instances).toHaveLength(0);
    expect(localStorage.getItem(channelName)).toBeNull();
    expect(published.join('')).not.toContain(seat.token);
    second.unmount(); await flush(); expect(first.result.current).toBe(false);
  });

  it('ignores sessionStorage events, other keys, removals, and damaged JSON', async () => {
    vi.stubGlobal('BroadcastChannel', undefined);
    const first = renderHook(hook, { initialProps: active() });
    const presence = JSON.stringify({ kind: 'hello', tabId: 'peer', roomId: seat.roomId, seat: seat.seat });
    act(() => {
      for (const event of [
        { key: channelName, newValue: presence, storageArea: sessionStorage },
        { key: 'another-key', newValue: presence, storageArea: localStorage },
        { key: channelName, newValue: null, storageArea: localStorage },
        { key: channelName, newValue: 'broken', storageArea: localStorage },
      ]) window.dispatchEvent(new StorageEvent('storage', event));
    });
    expect(first.result.current).toBe(false);
  });

  it('degrades without a conflict or crash when both transports are denied', () => {
    vi.stubGlobal('BroadcastChannel', undefined);
    vi.spyOn(window, 'localStorage', 'get').mockImplementation(() => { throw new DOMException('blocked', 'SecurityError'); });
    const first = renderHook(hook, { initialProps: active() });
    expect(first.result.current).toBe(false); expect(vi.getTimerCount()).toBe(0);
  });

  it('keeps storage write failures from disrupting play or lifecycle cleanup', async () => {
    vi.stubGlobal('BroadcastChannel', undefined);
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new DOMException('quota', 'QuotaExceededError'); });
    const first = renderHook(hook, { initialProps: active() });
    await act(async () => { await vi.advanceTimersByTimeAsync(30_000); window.dispatchEvent(new Event('pagehide')); window.dispatchEvent(new Event('pageshow')); });
    expect(first.result.current).toBe(false);
    first.unmount(); expect(vi.getTimerCount()).toBe(0);
  });
});
