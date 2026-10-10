import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { clearPending, readPending, readPendings, retireSeatPendings, savePending, saveSession, type PendingCommand } from './api';
import { playerStorage, selectPlayerMode } from './playerStorage';

const seat = { roomId: 'first', seat: 0, token: 'opaque-original', inviteCode: 'FIRST' };
const original: PendingCommand = { roomId: seat.roomId, seat: seat.seat, commandId: 'original', expectedVersion: 7, action: { kind: 'game', action: { kind: 'ready' } } };
const legacyKey = 'hegemony.pending.v1';
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); saveSession(seat); });
afterEach(() => { vi.restoreAllMocks(); localStorage.clear(); sessionStorage.clear(); });

describe('command-scoped pending persistence and legacy recovery', () => {
  it('keeps two commands of the same actor independent and removes only the confirmed one', () => {
    const peer = { ...original, commandId: 'peer', action: { kind: 'deck', option: 'watchers' } };
    savePending(original); savePending(peer); clearPending(original);
    expect(readPending(seat)).toEqual(peer);
    clearPending(peer); expect(readPending()).toBeNull();
  });
  it('keeps rooms and seats separate, including identifiers containing separators', () => {
    const commands = [original, { ...original, seat: 1 }, { ...original, roomId: 'first.0.original' }, { ...original, commandId: 'original.0.original' }];
    commands.forEach(command => savePending(command));
    for (const command of commands) {
      const actor = { ...seat, roomId: command.roomId, seat: command.seat! };
      expect(readPending(actor)).not.toBeNull();
      clearPending(command);
    }
    expect(readPending()).toBeNull();
  });
  it('does not overwrite a stored identity with a different intent using the same ID', () => {
    savePending(original); savePending({ ...original, expectedVersion: 8 });
    expect(readPending(seat)).toEqual(original);
  });
  it('preserves a seatless old record without inferring its actor from a mutable saved session', () => {
    const { seat: _seat, ...old } = original;
    localStorage.setItem(legacyKey, JSON.stringify(old));
    const restored = readPending(seat)!;
    expect(restored).toEqual({ ...old, seat: undefined });
    savePending(restored);
    saveSession({ ...seat, seat: 1, token: 'peer-token' });
    expect(readPending({ ...seat, seat: 1 })).toEqual(restored);
    expect(readPending(seat)).toEqual(restored);
    clearPending(restored);
    expect(readPending()).toEqual(restored);
    expect(localStorage.getItem(legacyKey)).toBe(JSON.stringify(old));
    expect(playerStorage().keys().some(key => key.startsWith('hegemony.pending.v2.'))).toBe(false);
  });
  it('does not trust guessed modern copies or markers of the same seatless legacy ID', () => {
    savePending(original); savePending({ ...original, seat: 1 }); clearPending(original);
    const { seat: _seat, ...old } = original;
    localStorage.setItem(legacyKey, JSON.stringify(old));
    const peer = { ...original, commandId: 'other-ID' }; savePending(peer);
    expect(readPendings()).toEqual([{ ...old, seat: undefined }, peer]);
    expect(readPending({ ...seat, seat: 1 })?.seat).toBeUndefined();
  });
  it('retires all matching actor records in the captured scope while preserving other actors and scopes', () => {
    const ordinary = playerStorage();
    const second = { ...original, commandId: 'second' }; const peer = { ...original, seat: 1 }; const other = { ...original, roomId: 'other' };
    [original, second, peer, other].forEach(command => savePending(command, ordinary));
    selectPlayerMode('independent', 'independent-player-0001'); savePending(original);
    expect(retireSeatPendings(seat, ordinary)).toEqual([original, second]);
    expect(readPending(seat, ordinary)).toBeNull();
    expect(readPending({ ...seat, seat: 1 }, ordinary)).toEqual(peer);
    expect(readPending({ ...seat, roomId: 'other' }, ordinary)).toEqual(other);
    expect(readPending(seat)).toEqual(original);
  });
  it('a late legacy confirmation preserves both another legacy command and a new scoped command', () => {
    localStorage.setItem(legacyKey, JSON.stringify(original));
    savePending(readPending()!);
    const peer = { ...original, commandId: 'peer', expectedVersion: 8 };
    localStorage.setItem(legacyKey, JSON.stringify(peer)); savePending(peer);
    clearPending(original);
    expect(readPending(seat)).toEqual(peer);
    expect(localStorage.getItem(legacyKey)).toBe(JSON.stringify(peer));
  });
  it('a legacy writer replacing the slot during retirement cannot have its command deleted', () => {
    localStorage.setItem(legacyKey, JSON.stringify(original)); savePending(original);
    const peer = { ...original, commandId: 'peer' };
    const storage = playerStorage();
    const interleaved = { ...storage, setItem: (key: string, value: string) => {
      localStorage.setItem(legacyKey, JSON.stringify(peer)); storage.setItem(key, value);
    } };
    clearPending(original, interleaved);
    expect(readPending(seat)).toEqual(peer);
    expect(localStorage.getItem(legacyKey)).toBe(JSON.stringify(peer));
  });
  it('captures the storage scope so a late ordinary confirmation cannot clear an independent command', () => {
    const ordinary = playerStorage(); savePending(original, ordinary);
    selectPlayerMode('independent', 'independent-player-0001');
    saveSession(seat); savePending(original);
    clearPending(original, ordinary);
    expect(readPending(seat, ordinary)).toBeNull();
    expect(readPending(seat)).toEqual(original);
  });
  it('a malformed entry or unrelated legacy marker cannot hide a good command', () => {
    savePending(original);
    localStorage.setItem('hegemony.pending.v2.corrupt', '{broken');
    localStorage.setItem('hegemony.pending.legacyConfirmed.v2.wrong-identity', JSON.stringify(original));
    expect(readPending(seat)).toEqual(original);
    clearPending(original);
    localStorage.setItem(legacyKey, JSON.stringify(original));
    expect(readPending(seat)).toEqual(original);
  });
  it('keeps the sole legacy copy intact when migration storage is unavailable', () => {
    localStorage.setItem(legacyKey, JSON.stringify(original));
    const storage = playerStorage();
    const denied = { ...storage, setItem: () => { throw new Error('quota'); } };
    savePending(original, denied); clearPending(original, denied);
    expect(readPending(seat)).toEqual(original);
    expect(localStorage.getItem(legacyKey)).toBe(JSON.stringify(original));
  });
});
