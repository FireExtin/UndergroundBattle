import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { choiceDraftFingerprint, createChoiceDraftStore, initialChoiceDraft, readChoiceDraft, reconcileChoiceDraft, saveChoiceDraft } from './choiceDraftStorage';
import { selectPlayerMode } from './playerStorage';
import { testCard, testChoice } from './testFixtures';
import type { Choice } from './types';

beforeEach(() => { localStorage.clear(); sessionStorage.clear(); });
afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals(); localStorage.clear(); sessionStorage.clear(); });
const selected = () => ({ ...initialChoiceDraft(testChoice), selected: ['b'] });

describe('tab-local choice draft validation', () => {
  it.each([
    ['unknown selection', { selected: ['missing'] }],
    ['duplicate selection', { selected: ['a', 'a'] }],
    ['too many selected targets', { selected: ['a', 'b'] }],
    ['missing ordered card', { top: ['a'] }],
    ['duplicate ordered card', { top: ['a'], bottom: ['a'] }],
    ['unknown ordered card', { top: ['a'], bottom: ['missing'] }],
    ['unknown damage target', { allocations: { missing: 0 } }],
    ['negative damage', { allocations: { a: -1 } }],
    ['fractional damage', { allocations: { a: 0.5 } }],
    ['excessive damage', { allocations: { a: 2, b: 2 } }],
    ['string damage', { allocations: { a: '1' } }],
  ])('rejects %s in stored edits', (_, change) => {
    const choice = { ...testChoice, kind: 'damage', amount: 3 };
    const store = createChoiceDraftStore('room', 'p0')!;
    sessionStorage.setItem(store.key, JSON.stringify({ fingerprint: choiceDraftFingerprint(choice), draft: { ...initialChoiceDraft(choice), ...change } }));
    expect(readChoiceDraft(store, choice)).toBeNull();
    reconcileChoiceDraft(store, choice);
    expect(sessionStorage.getItem(store.key)).toBeNull();
  });

  it.each(['broken json', 'null', '{}', '{"fingerprint":"wrong","draft":{}}'])('ignores and cleans malformed or obsolete data: %s', raw => {
    const store = createChoiceDraftStore('room', 'p0')!;
    sessionStorage.setItem(store.key, raw);
    expect(readChoiceDraft(store, testChoice)).toBeNull();
    reconcileChoiceDraft(store, testChoice);
    expect(sessionStorage.getItem(store.key)).toBeNull();
  });

  it.each([
    ['new choice', { id: 'next-choice' }],
    ['new chooser', { playerId: 'p1' }],
    ['new kind', { kind: 'mulligan' }],
    ['minimum', { min: 0 }],
    ['maximum', { max: 2 }],
    ['amount', { amount: 1 }],
    ['decline permission', { allowDecline: true }],
    ['new preview', { previewCards: [testCard] }],
    ['removed option', { options: [testChoice.options[0]] }],
  ] satisfies [string, Partial<Choice>][])('does not restore a stale draft after changing %s', (_, change) => {
    const store = createChoiceDraftStore('room', 'p0')!;
    saveChoiceDraft(store, testChoice, selected());
    expect(readChoiceDraft(store, { ...testChoice, ...change })).toBeNull();
  });

  it('captures the player namespace and keeps independent and ordinary drafts separate', () => {
    const ordinary = createChoiceDraftStore('room', 'p0')!;
    saveChoiceDraft(ordinary, testChoice, selected());
    selectPlayerMode('independent', 'independent-player-1234');
    const independent = createChoiceDraftStore('room', 'p0')!;
    expect(readChoiceDraft(independent, testChoice)).toBeNull();
    saveChoiceDraft(independent, testChoice, { ...selected(), selected: ['a'] });
    reconcileChoiceDraft(ordinary, null);
    expect(readChoiceDraft(independent, testChoice)?.selected).toEqual(['a']);
    selectPlayerMode('ordinary');
    expect(readChoiceDraft(createChoiceDraftStore('room', 'p0'), testChoice)).toBeNull();
    expect(localStorage.length).toBe(0);
  });

  it('keeps the ordinary draft in its captured tab when a second tab closes its choice', () => {
    const firstTab = createChoiceDraftStore('room', 'p0')!;
    saveChoiceDraft(firstTab, testChoice, selected());
    const secondTabData = new Map<string, string>();
    const secondTabStorage = {
      getItem: (key: string) => secondTabData.get(key) ?? null,
      setItem: (key: string, value: string) => { secondTabData.set(key, value); },
      removeItem: (key: string) => { secondTabData.delete(key); },
    };
    vi.stubGlobal('sessionStorage', secondTabStorage);
    const secondTab = createChoiceDraftStore('room', 'p0')!;
    expect(readChoiceDraft(secondTab, testChoice)).toBeNull();
    saveChoiceDraft(secondTab, testChoice, { ...selected(), selected: ['a'] });
    reconcileChoiceDraft(secondTab, null);
    expect(readChoiceDraft(firstTab, testChoice)?.selected).toEqual(['b']);
  });

  it('clears an older persisted edit when saving a newer edit fails', () => {
    const store = createChoiceDraftStore('room', 'p0')!;
    saveChoiceDraft(store, testChoice, selected());
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new DOMException('full', 'QuotaExceededError'); });
    expect(() => saveChoiceDraft(store, testChoice, { ...selected(), selected: ['a'] })).not.toThrow();
    expect(readChoiceDraft(store, testChoice)).toBeNull();
  });

  it('does not fall back to another player when the namespace cannot be read', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new DOMException('denied', 'SecurityError'); });
    expect(createChoiceDraftStore('room', 'p0')).toBeUndefined();
    expect(() => reconcileChoiceDraft(undefined, null)).not.toThrow();
  });
});
