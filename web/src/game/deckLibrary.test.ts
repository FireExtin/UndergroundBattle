import { describe, expect, it } from 'vitest';
import { createDeckDraft, DECK_LIBRARY_STORAGE_KEY, readDeckLibrary, revalidateDraftVersions, saveDeckLibrary, validateDeckDraft } from './deckLibrary';
import type { DeckCatalog, DeckDraft, DeckStorage } from './deckLibrary';

const catalog: DeckCatalog = {
  rulesVersion: 'rules-1', cardPoolVersion: 'pool-1', engineVersion: 'engine-1', decks: [],
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: false },
  cards: [
    { id: 'neutral', name: '不限同名示例', kind: 'character', cost: 0, text: '', color: '中立', supported: true, deckCopyLimit: null },
    { id: 'yellow', name: '共同名称', kind: 'character', cost: 1, text: '', color: '黄', supported: true, deckCopyLimit: 3 },
    { id: 'yellow-alt', name: '共同名称', kind: 'character', cost: 1, text: '', color: '黄', supported: true, deckCopyLimit: 3 },
    { id: 'green', name: '绿色事务', kind: 'spell', cost: 1, text: '', color: '绿', supported: true, deckCopyLimit: 3 },
    { id: 'single', name: '唯一构筑示例', kind: 'spell', cost: 1, text: '', supported: true, deckCopyLimit: 1 },
    { id: 'region', name: '地区', kind: 'region', cost: 0, text: '', supported: true, deckCopyLimit: 3 },
    { id: 'unsupported', name: '未实现', kind: 'character', cost: 0, text: '', supported: false, deckCopyLimit: 3 },
  ],
};
function draft(cards: DeckDraft['cards'] = [{ cardId: 'neutral', count: 50 }]): DeckDraft {
  return { ...createDeckDraft(catalog), name: '自组牌组', cards };
}
function memoryStorage(raw: string | null = null): DeckStorage {
  let value = raw;
  return { getItem: () => value, setItem: (_key, next) => { value = next; } };
}

describe('explicit construction rules', () => {
  it('copies and validates a supported preset containing the live spell kind', () => {
    const preset = { id: 'responders', name: '响应预组示例', description: '角色与事务', cardCount: 50, cards: [{ cardId: 'neutral', count: 47 }, { cardId: 'green', count: 3 }] };
    const copy = createDeckDraft(catalog, preset);
    expect(copy.cards).toEqual(preset.cards);
    expect(validateDeckDraft(copy, catalog)).toEqual({ valid: true, total: 50, issues: [] });
    const legacy = { ...catalog, cards: catalog.cards.map(card => card.id === 'green' ? { ...card, kind: 'event' } : card) };
    expect(validateDeckDraft(copy, legacy).valid).toBe(true);
  });
  it('accepts arbitrary faction mixing and only metadata-declared unlimited copies', () => {
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 44 }, { cardId: 'yellow', count: 3 }, { cardId: 'green', count: 3 }]), catalog)).toEqual({ valid: true, total: 50, issues: [] });
    const namedLikeException = { ...catalog, cards: catalog.cards.map(card => card.id === 'neutral' ? { ...card, id: 'JC125', text: '数量没有限制', deckCopyLimit: 3 } : card) };
    expect(validateDeckDraft(draft([{ cardId: 'JC125', count: 50 }]), namedLikeException).issues).toEqual(expect.arrayContaining([expect.objectContaining({ code: 'copies' })]));
  });
  it('aggregates limits by printed name across IDs and repeated entries', () => {
    const result = validateDeckDraft(draft([{ cardId: 'neutral', count: 46 }, { cardId: 'yellow', count: 2 }, { cardId: 'yellow-alt', count: 2 }]), catalog);
    expect(result.valid).toBe(false);
    expect(result.issues.find(issue => issue.code === 'copies')?.message).toContain('合计 4 张');
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 46 }, { cardId: 'yellow', count: 2 }, { cardId: 'yellow', count: 2 }]), catalog).valid).toBe(false);
  });
  it('uses an explicit unique construction limit rather than battlefield uniqueness', () => {
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 48 }, { cardId: 'single', count: 2 }]), catalog).issues.some(issue => issue.code === 'copies')).toBe(true);
    const battlefieldUnique = { ...catalog, cards: catalog.cards.map(card => ({ ...card, unique: true })) };
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 47 }, { cardId: 'yellow', count: 3 }]), battlefieldUnique).valid).toBe(true);
  });
  it('blocks missing and conflicting metadata without guessing an exception', () => {
    const legacy = { ...catalog, deckBuildRules: undefined };
    expect(validateDeckDraft(draft(), legacy).issues.some(issue => issue.code === 'metadata')).toBe(true);
    const noLimit = { ...catalog, cards: catalog.cards.map(card => ({ ...card, deckCopyLimit: undefined })) };
    const unconfirmed = validateDeckDraft(draft(), noLimit);
    expect(unconfirmed.valid).toBe(false);
    expect(unconfirmed.issues.some(issue => issue.code === 'copy-metadata')).toBe(true);
    expect(unconfirmed.issues.some(issue => issue.code === 'copies')).toBe(false);
    const conflicting = { ...catalog, cards: catalog.cards.map(card => card.id === 'yellow-alt' ? { ...card, deckCopyLimit: 1 } : card) };
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 48 }, { cardId: 'yellow', count: 1 }, { cardId: 'yellow-alt', count: 1 }]), conflicting).issues.some(issue => issue.code === 'copy-conflict')).toBe(true);
  });
  it('blocks unknown, region, unimplemented, and secret-society choices', () => {
    for (const [cardId, code] of [['missing', 'unknown'], ['region', 'kind'], ['unsupported', 'unsupported']]) {
      const result = validateDeckDraft(draft([{ cardId: 'neutral', count: 49 }, { cardId, count: 1 }]), catalog);
      expect(result.valid).toBe(false); expect(result.issues.some(issue => issue.code === code)).toBe(true);
    }
    expect(validateDeckDraft({ ...draft(), societyId: 'not-open' } as unknown as DeckDraft, catalog).issues.some(issue => issue.code === 'society')).toBe(true);
  });
  it('separates the formal minimum from the advertised service capacity', () => {
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 49 }]), catalog).issues.find(issue => issue.code === 'minimum')?.message).toContain('还差 1 张');
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 2048 }]), catalog).valid).toBe(true);
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 2049 }]), catalog).issues.find(issue => issue.code === 'capacity')?.message).toContain('服务容量限制');
    expect(validateDeckDraft(draft([{ cardId: 'neutral', count: 1.5 }]), catalog).issues.some(issue => issue.code === 'count')).toBe(true);
  });
  it('preserves old provenance until an explicit version revalidation', () => {
    const previous = { ...draft(), cardPoolVersion: 'old-pool', engineVersion: 'old-engine' };
    expect(validateDeckDraft(previous, catalog).issues.some(issue => issue.code === 'version')).toBe(true);
    const refreshed = revalidateDraftVersions(previous, catalog);
    expect(previous.cardPoolVersion).toBe('old-pool');
    expect(refreshed.cards).toEqual(previous.cards);
    expect(validateDeckDraft(refreshed, catalog).valid).toBe(true);
  });
});

describe('browser deck library storage', () => {
  it('saves invalid drafts while storing only public draft fields', () => {
    const storage = memoryStorage();
    const incomplete = { ...draft([{ cardId: 'missing-old-card', count: 2 }]), token: 'do-not-store', hand: ['private-card'], cards: [{ cardId: 'missing-old-card', count: 2, instanceId: 'private-instance' }] };
    expect(saveDeckLibrary([incomplete], storage)).toBeNull();
    const raw = storage.getItem(DECK_LIBRARY_STORAGE_KEY)!;
    expect(raw).not.toMatch(/do-not-store|private-card|private-instance|token|hand/);
    expect(readDeckLibrary(storage).drafts[0].cards).toEqual([{ cardId: 'missing-old-card', count: 2 }]);
    expect(validateDeckDraft(readDeckLibrary(storage).drafts[0], catalog).valid).toBe(false);
  });
  it('recovers good entries alongside malformed drafts and ignores unknown stored fields', () => {
    const good = { ...draft(), token: 'private-token' };
    const storage = memoryStorage(JSON.stringify({ version: 1, drafts: [good, { ...draft(), id: 'bad', cards: [{ cardId: 'neutral', count: -1 }] }, { ...draft(), id: 'secret', societyId: 'not-supported' }] }));
    const read = readDeckLibrary(storage);
    expect(read.drafts).toHaveLength(1); expect(read.warning).toContain('格式损坏');
    expect(read.drafts[0]).not.toHaveProperty('token');
  });
  it('does not overwrite malformed or future-version data merely by reading it', () => {
    for (const raw of ['{broken', JSON.stringify({ version: 2, drafts: [draft()] })]) {
      const storage = memoryStorage(raw);
      expect(readDeckLibrary(storage).drafts).toEqual([]);
      expect(readDeckLibrary(storage).warning).toContain('未覆盖原数据');
      expect(storage.getItem(DECK_LIBRARY_STORAGE_KEY)).toBe(raw);
    }
  });
  it('reports unavailable or quota-limited storage without throwing', () => {
    const storage: DeckStorage = { getItem: () => { throw new Error('denied'); }, setItem: () => { throw new Error('quota'); } };
    expect(readDeckLibrary(storage).warning).toBeTruthy();
    expect(saveDeckLibrary([draft()], storage)).toContain('未能保存');
    expect(saveDeckLibrary([draft()], null)).toContain('不可用');
  });
});
