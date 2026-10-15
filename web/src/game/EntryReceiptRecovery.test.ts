import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { testCatalog, testView } from './testFixtures';
import { playerStorage, selectPlayerMode } from './playerStorage';

const entryKey = 'hegemony.entry.v1';
const seat = { roomId: testView.roomId, inviteCode: testView.inviteCode, token: 'entry-recovery-test-token', seat: 0 };
const session = { ...seat, view: testView };
const json = (value: unknown) => new Response(JSON.stringify(value), { status: 200 });
let api: typeof import('./api');

beforeEach(async () => {
  localStorage.clear(); sessionStorage.clear(); vi.resetModules();
  api = await import('./api');
});
afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals(); });

describe('entry receipt recovery', () => {
  it.each([
    ['an HTML login response', () => new Response('<html>Login</html>', { status: 200 })],
    ['a missing table', () => json(seat)],
    ['a table for another room', () => json({ ...session, view: { ...testView, roomId: 'other-room' } })],
    ['a table for another seat', () => json({ ...session, view: { ...testView, you: 'p1' } })],
    ['a missing seat credential', () => json({ ...session, token: '' })],
  ] as const)('retains the exact request through %s and a page reload', async (_label, invalidReply) => {
    const posts: string[] = [];
    let recovered = false;
    vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
      if (!init?.body) return json({ ...testCatalog, entryIdempotency: true });
      posts.push(String(init.body));
      return recovered ? json(session) : invalidReply();
    }));
    await api.getCatalog();
    await expect(api.createRoom('恢复玩家', 'duel', 'watchers')).rejects.toMatchObject({ status: 0 });
    expect(posts).toHaveLength(2);
    expect(posts[1]).toBe(posts[0]);
    const receipt = localStorage.getItem(entryKey);
    expect(receipt).not.toBeNull();
    expect(api.readSession()).toBeNull();

    vi.resetModules(); api = await import('./api'); recovered = true;
    await api.getCatalog();
    await expect(api.createRoom('恢复玩家', 'duel', 'watchers')).resolves.toEqual(session);
    expect(posts).toHaveLength(3);
    expect(posts[2]).toBe(posts[0]);
    expect(api.readSession()).toEqual(seat);
    expect(localStorage.getItem(entryKey)).toBeNull();
  });

  it('persists the usable seat before clearing its successful entry receipt', async () => {
    const originalRemove = Storage.prototype.removeItem;
    const seatsAtReceiptRemoval: unknown[] = [];
    vi.spyOn(Storage.prototype, 'removeItem').mockImplementation(function (this: Storage, key: string) {
      if (this === localStorage && key === entryKey) seatsAtReceiptRemoval.push(api.readSession());
      originalRemove.call(this, key);
    });
    vi.stubGlobal('fetch', vi.fn(async () => json(session)));
    await expect(api.joinRoom('INVITE', '恢复玩家', 'watchers')).resolves.toEqual(session);
    expect(seatsAtReceiptRemoval).toEqual([seat]);
    expect(api.readSavedSeats()).toEqual([seat]);
    expect(api.readActiveSession()).toEqual(seat);
  });

  it('keeps the entry identity when seat persistence fails and recovers it after a reload', async () => {
    const originalSet = Storage.prototype.setItem;
    let full = true;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key: string, value: string) {
      if (this === localStorage && full && key === 'hegemony.seats.v1') throw new DOMException('full', 'QuotaExceededError');
      originalSet.call(this, key, value);
    });
    const posts: string[] = [];
    vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
      posts.push(String(init?.body)); return json(session);
    }));
    await expect(api.joinRoom('INVITE', '恢复玩家', 'watchers')).resolves.toEqual(session);
    expect(api.readSession()).toBeNull();
    expect(localStorage.getItem(entryKey)).not.toBeNull();

    full = false; vi.resetModules(); api = await import('./api');
    await expect(api.joinRoom('INVITE', '恢复玩家', 'watchers')).resolves.toEqual(session);
    expect(posts).toHaveLength(2);
    expect(posts[1]).toBe(posts[0]);
    expect(api.readActiveSession()).toEqual(seat);
    expect(localStorage.getItem(entryKey)).toBeNull();
  });

  it('does not clear a newer entry receipt when an older request completes', async () => {
    let finishFirst: ((response: Response) => void) | undefined;
    vi.stubGlobal('fetch', vi.fn((_url: string, init?: RequestInit) => {
      const { name } = JSON.parse(String(init?.body));
      if (name === '先发玩家') return new Promise<Response>(resolve => { finishFirst = resolve; });
      return Promise.reject(new Error('lost acknowledgement'));
    }));
    const first = api.createRoom('先发玩家', 'duel', 'watchers');
    await expect(api.createRoom('后发玩家', 'duel', 'watchers')).rejects.toMatchObject({ status: 0 });
    const newer = localStorage.getItem(entryKey);
    expect(newer).not.toBeNull();
    finishFirst!(json(session));
    await first;
    expect(localStorage.getItem(entryKey)).toBe(newer);
  });

  it('retires the original entry after retrying a failed save so the next explicit create is new', async () => {
    const originalSet = Storage.prototype.setItem;
    let full = true;
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key: string, value: string) {
      if (full && key === 'hegemony.seats.v1') throw new DOMException('full', 'QuotaExceededError');
      originalSet.call(this, key, value);
    });
    const posts: { requestId: string }[] = [];
    vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
      posts.push(JSON.parse(String(init?.body))); return json(session);
    }));
    await api.createRoom('恢复玩家', 'duel', 'watchers');
    expect(api.isSessionSaved(seat)).toBe(false);
    expect(localStorage.getItem(entryKey)).not.toBeNull();

    full = false;
    expect(api.saveSession({ ...seat })).toBe(true);
    expect(localStorage.getItem(entryKey)).toBeNull();
    await api.createRoom('恢复玩家', 'duel', 'watchers');
    expect(posts).toHaveLength(2);
    expect(posts[1].requestId).not.toBe(posts[0].requestId);
  });

  it('keeps the newer active identity when an older entry arrives after it', async () => {
    let finishFirst: ((response: Response) => void) | undefined;
    const newerSeat = { ...seat, roomId: 'newer-room', inviteCode: 'NEWER', token: 'newer-entry-test-token' };
    const newerSession = { ...newerSeat, view: { ...testView, roomId: 'newer-room', inviteCode: 'NEWER' } };
    vi.stubGlobal('fetch', vi.fn((_url: string, init?: RequestInit) => {
      const { name } = JSON.parse(String(init?.body));
      if (name === '先发玩家') return new Promise<Response>(resolve => { finishFirst = resolve; });
      return Promise.resolve(json(newerSession));
    }));
    const first = api.createRoom('先发玩家', 'duel', 'watchers');
    await api.createRoom('后发玩家', 'duel', 'watchers');
    expect(api.readActiveSession()).toEqual(newerSeat);
    finishFirst!(json(session)); await first;
    expect(api.readActiveSession()).toEqual(newerSeat);
    expect(api.readSavedSeats()).toEqual(expect.arrayContaining([seat, newerSeat]));
    expect(localStorage.getItem(entryKey)).toBeNull();
  });

  it('saves a late ordinary acknowledgement only in its original namespace', async () => {
    let finish: ((response: Response) => void) | undefined;
    vi.stubGlobal('fetch', vi.fn(() => new Promise<Response>(resolve => { finish = resolve; })));
    const ordinary = playerStorage();
    const entry = api.createRoom('普通玩家', 'duel', 'watchers');
    selectPlayerMode('independent', 'independent-entry-test-0001');
    const independentSeat = { ...seat, seat: 1, token: 'independent-entry-test-token' };
    expect(api.saveSession(independentSeat)).toBe(true);
    const independent = playerStorage();
    independent.setItem(entryKey, JSON.stringify({ intent: 'independent-intent', requestId: 'independent-request' }));
    finish!(json(session)); await entry;
    expect(api.readSession()).toEqual(independentSeat);
    expect(api.isSessionSaved(independentSeat)).toBe(true);
    expect(api.readSession(ordinary)).toEqual(seat);
    expect(api.isSessionSaved(seat, ordinary)).toBe(true);
    expect(ordinary.getItem(entryKey)).toBeNull();
    expect(independent.getItem(entryKey)).not.toBeNull();
  });

  it('does not report persistence when a storage write cannot be read back', () => {
    const storage = playerStorage();
    const dropped = { ...storage, setItem: (key: string, value: string) => {
      if (key !== 'hegemony.seats.v1') storage.setItem(key, value);
    } };
    expect(api.saveSession(seat, dropped)).toBe(false);
    expect(api.isSessionSaved(seat, dropped)).toBe(false);
    expect(api.readSession(dropped)).toEqual(seat);
  });
});
