import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readPending, saveSession } from './api';
import { testCatalog, testView } from './testFixtures';
import { useGame } from './useGame';

const saved = { roomId: testView.roomId, seat: 0, token: 'quota-original-token', inviteCode: 'INVITE' };
const original = { roomId: saved.roomId, seat: 0, commandId: 'quota-legacy-original', expectedVersion: 7, action: { kind: 'game', action: { kind: 'ready' } } };
const legacyKey = 'hegemony.pending.v1';
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const bytes = (key: string, value: string) => 2 * (key.length + value.length);
function usedBytes(storage: Storage) {
  return Array.from({ length: storage.length }, (_, index) => storage.key(index)!).reduce((sum, key) => sum + bytes(key, storage.getItem(key)!), 0);
}
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); vi.useFakeTimers(); });
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });

describe('byte quota, actor invalidation and a fresh hook', () => {
  it.each([
    ['invalid_seat_token', 401, 'read'], ['room_not_found', 404, 'read'], ['unsupported_room_version', 410, 'read'],
    ['invalid_seat_token', 401, 'command'], ['room_not_found', 404, 'command'], ['unsupported_room_version', 410, 'command'],
  ] as const)(
    'retains a persistent recovery seat after %s (%i) on %s cannot retire a legacy record, then completes recovery after space is available', async (code, status, rejectionPath) => {
      saveSession(saved); localStorage.setItem(legacyKey, JSON.stringify(original));
      localStorage.setItem('quota-filler', 'x'.repeat(1024));
      const limit = usedBytes(localStorage);
      const write = Storage.prototype.setItem;
      const attempts: { key: string; previousBytes: number; nextBytes: number; rejected: boolean }[] = [];
      vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key: string, value: string) {
        if (this !== localStorage) return write.call(this, key, value);
        const previous = this.getItem(key); const previousBytes = previous === null ? 0 : bytes(key, previous);
        const nextBytes = bytes(key, String(value)); const rejected = usedBytes(this) - previousBytes + nextBytes > limit;
        attempts.push({ key, previousBytes, nextBytes, rejected });
        if (rejected) throw new DOMException('Byte quota exceeded', 'QuotaExceededError');
        write.call(this, key, value);
      });
      const requests: string[] = []; const commands: unknown[] = [];
      const next = { ...saved, roomId: 'new-quota-room', token: 'new-quota-token', view: { ...testView, roomId: 'new-quota-room', version: 1 } };
      vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
        requests.push(url);
        if (url.endsWith('/catalog')) return json(testCatalog);
        if (url === '/api/rooms') return json(next);
        if (url.endsWith('/commands')) {
          expect(new Headers(init?.headers).get('Authorization')).toBe(`Bearer ${saved.token}`);
          commands.push(JSON.parse(String(init?.body)));
        }
        if (url.includes('/new-quota-room/')) return json(next.view);
        if (rejectionPath === 'command' && !url.endsWith('/commands')) return json({ ...testView, version: 7 });
        return json({ error: code, message: 'definitive rejection' }, status);
      }));
      let hook = renderHook(useGame); await act(async () => {});
      expect(hook.result.current.session).toBeNull(); expect(readPending(saved)).toEqual(original);
      expect(attempts.some(attempt => attempt.key.startsWith('hegemony.pending.legacyConfirmed.v2.') && attempt.rejected)).toBe(true);
      expect(attempts.some(attempt => !attempt.rejected)).toBe(true); // The storage permits writes that fit, including shrink writes.
      hook.unmount(); hook = renderHook(useGame); await act(async () => {});
      console.info({ code, rejectionPath, limit, bytesAfterInvalidation: usedBytes(localStorage), savedSeatsAfterReload: hook.result.current.savedSeats, pendingAfterReload: readPending(saved), attempts });
      expect(hook.result.current.savedSeats).toEqual([saved]);
      expect(hook.result.current.resumeAvailable).toBe(true);
      await act(async () => { await hook.result.current.create('blocked', 'duel', 'watchers'); });
      expect(requests.filter(url => url === '/api/rooms')).toEqual([]);
      localStorage.removeItem('quota-filler'); // Release space without changing the actor or original pending.
      await act(async () => { hook.result.current.resume(hook.result.current.savedSeats[0]); });
      expect(readPending(saved)).toBeNull();
      await act(async () => { await hook.result.current.create('new', 'duel', 'watchers'); });
      expect(hook.result.current.session?.roomId).toBe(next.roomId);
      hook.unmount(); hook = renderHook(useGame); await act(async () => {});
      expect(hook.result.current.session?.roomId).toBe(next.roomId); expect(hook.result.current.uncertain).toBe(false);
      const body = { commandId: original.commandId, expectedVersion: original.expectedVersion, action: original.action };
      expect(commands).toEqual(rejectionPath === 'command' ? [body, body] : []);
    },
  );
});
