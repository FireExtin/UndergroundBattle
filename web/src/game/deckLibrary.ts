import type { CardDefinition, Catalog, Deck, SocietyDefinition } from './types';

export type DeckDraft = {
  id: string;
  name: string;
  description: string;
  societyId: string | null;
  cards: { cardId: string; count: number }[];
  rulesVersion: string;
  cardPoolVersion: string;
  engineVersion: string;
  updatedAt: string;
};

// Construction limits are explicit public catalog metadata, never inferred from
// a card ID, rules text, battlefield uniqueness, or the research-only card index.
export type DeckCardDefinition = CardDefinition & { deckCopyLimit?: number | null };
export type DeckCatalog = Catalog & {
  deckBuildRules?: {
    minimumCards: number;
    serviceCardCapacity: number;
    societySupported: boolean;
  };
  cards: DeckCardDefinition[];
};
export type DeckIssue = { code: string; message: string; cardId?: string };
export type DeckValidation = { valid: boolean; total: number; issues: DeckIssue[] };
export type DeckStorage = Pick<Storage, 'getItem' | 'setItem'>;
export const DECK_LIBRARY_STORAGE_KEY = 'hegemony.deckLibrary.v1';

// The live Rust catalog calls transaction cards "spell"; "event" is retained
// only for reading older public catalogs, not as a replacement runtime kind.
const playerKinds = new Set(['character', 'spell', 'event', 'attachment']);
export function isEditableDeckCard(card: DeckCardDefinition): boolean {
  return card.supported === true && playerKinds.has(card.kind);
}

export function selectableSocieties(catalog: DeckCatalog): SocietyDefinition[] {
  return catalog.deckBuildRules?.societySupported === true
    ? (catalog.societies || []).filter(card => card.kind === 'society' && card.supported !== false) : [];
}

export function societyColorCount(draft: DeckDraft, catalog: DeckCatalog, color: string): number {
  const normalized = (value: string) => value.trim().replace(/色$/, '');
  const ids = new Set(catalog.cards.filter(card => isEditableDeckCard(card) && normalized(card.color || '') === normalized(color)).map(card => card.id));
  return draft.cards.reduce((sum, entry) => sum + (ids.has(entry.cardId) && Number.isSafeInteger(entry.count) && entry.count > 0 ? entry.count : 0), 0);
}

export function validateDeckDraft(draft: DeckDraft, catalog: DeckCatalog): DeckValidation {
  const issues: DeckIssue[] = [];
  const issue = (code: string, message: string, cardId?: string) => issues.push({ code, message, ...(cardId ? { cardId } : {}) });
  if (!draft.name.trim()) issue('name', '请给牌组起一个名字。');
  if (draft.rulesVersion !== catalog.rulesVersion || draft.cardPoolVersion !== catalog.cardPoolVersion || draft.engineVersion !== catalog.engineVersion) {
    issue('version', '来源版本已变化，请明确按当前卡池重新校验。');
  }
  const rules = catalog.deckBuildRules;
  const rulesKnown = !!rules && Number.isSafeInteger(rules.minimumCards) && rules.minimumCards >= 1 && Number.isSafeInteger(rules.serviceCardCapacity) && rules.serviceCardCapacity >= rules.minimumCards;
  if (!rulesKnown) issue('metadata', '目录缺少构筑规则元数据；可编辑保存，当前仅原预组可开局。');
  const definitions = new Map(catalog.cards.map(card => [card.id, card]));
  const named = new Map<string, { count: number; limits: Set<number | null>; ids: string[]; missingLimit: boolean }>();
  let total = 0;
  for (const entry of draft.cards) {
    if (!Number.isSafeInteger(entry.count) || entry.count < 1) {
      issue('count', `「${entry.cardId}」的张数须为正整数。`, entry.cardId);
      continue;
    }
    total += entry.count;
    const card = definitions.get(entry.cardId);
    if (!card) { issue('unknown', `未知卡牌「${entry.cardId}」，请移除或等待目录支持。`, entry.cardId); continue; }
    if (!playerKinds.has(card.kind)) issue('kind', `「${card.name}」不是可加入玩家牌组的角色、事务或附属。`, card.id);
    if (card.supported !== true) issue('unsupported', `「${card.name}」尚未确认已实现，不能用于开局。`, card.id);
    const name = card.name.trim();
    const group = named.get(name) || { count: 0, limits: new Set<number | null>(), ids: [], missingLimit: false };
    group.count += entry.count;
    group.ids.push(card.id);
    if (card.deckCopyLimit === null || (typeof card.deckCopyLimit === 'number' && Number.isSafeInteger(card.deckCopyLimit) && card.deckCopyLimit >= 1)) group.limits.add(card.deckCopyLimit);
    else group.missingLimit = true;
    named.set(name, group);
  }
  if (!Number.isSafeInteger(total)) issue('count', '张数超出浏览器可准确计算的范围。');
  if (rulesKnown && rules) {
    if (total < rules.minimumCards) issue('minimum', `至少需要 ${rules.minimumCards} 张，当前 ${total} 张，还差 ${rules.minimumCards - total} 张。`);
    if (total > rules.serviceCardCapacity) issue('capacity', `当前服务最多接收 ${rules.serviceCardCapacity} 张，当前 ${total} 张；这是服务容量限制。`);
  }
  for (const [name, group] of named) {
    if (group.missingLimit) issue('copy-metadata', `「${name}」缺少明确的同名副本限制，暂不能确认合法。`, group.ids[0]);
    if (group.limits.size > 1) issue('copy-conflict', `「${name}」的同名副本元数据不一致，暂不能确认合法。`, group.ids[0]);
    if (!group.missingLimit && group.limits.size === 1) {
      const limit = [...group.limits][0];
      if (limit !== null && group.count > limit) issue('copies', `「${name}」同名最多 ${limit} 张，当前合计 ${group.count} 张。`, group.ids[0]);
    }
  }
  if (draft.societyId !== null) {
    const society = selectableSocieties(catalog).find(card => card.id === draft.societyId);
    if (!society) issue('society', '当前目录未开放此秘社，请清除选择或更换为已注册秘社。');
    else {
      if (!Number.isSafeInteger(society.startingHand) || society.startingHand < 0 || !Array.isArray(society.deckConstraints)) {
        issue('society-metadata', `「${society.name}」的起手或构筑要求尚未确认，暂不能用于开局。`);
      } else for (const constraint of society.deckConstraints) {
        if (constraint.kind === 'greenNeutralOrPrintedHumanCombat' && society.id === 'MSJC11') {
          for (const entry of draft.cards) {
            const card = catalog.cards.find(card => card.id === entry.cardId);
            if (card && !(['绿', '中立'].includes(card.color || '') || (card.kind === 'character' && card.subtypes?.includes('人类') && (card.permanentIcons?.combat ?? card.icons?.permanent.combat ?? 0) > 0))) {
              issue('society-combat-human', `「${society.name}」不允许「${card.name}」：异色牌须为印刷人类角色且具有永久战斗图标。`, card.id);
            }
          }
          continue;
        }
        if (constraint.kind !== 'minimumColor' || typeof constraint.color !== 'string' || !constraint.color.trim() || !Number.isSafeInteger(constraint.count) || constraint.count < 0) {
          issue('society-metadata', `「${society.name}」的构筑要求尚未确认，暂不能用于开局。`);
          continue;
        }
        const count = societyColorCount(draft, catalog, constraint.color);
        if (count < constraint.count) issue('society-color', `「${society.name}」要求至少 ${constraint.count} 张${constraint.color.trim().replace(/色$/, '')}色卡，当前 ${count} 张。`);
      }
    }
  }
  return { valid: issues.length === 0, total, issues };
}

export function publicDeckDraft(draft: DeckDraft): DeckDraft {
  return {
    id: draft.id, name: draft.name, description: draft.description, societyId: draft.societyId,
    cards: draft.cards.map(({ cardId, count }) => ({ cardId, count })),
    rulesVersion: draft.rulesVersion, cardPoolVersion: draft.cardPoolVersion, engineVersion: draft.engineVersion, updatedAt: draft.updatedAt,
  };
}

export function createDeckDraft(catalog: DeckCatalog, preset?: Deck): DeckDraft {
  return {
    id: `draft-${crypto.randomUUID()}`, name: preset ? `${preset.name}（副本）` : '我的牌组', description: preset?.description || '', societyId: null,
    cards: preset ? preset.cards.map(({ cardId, count }) => ({ cardId, count })) : [],
    rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion, engineVersion: catalog.engineVersion, updatedAt: new Date().toISOString(),
  };
}

export function revalidateDraftVersions(draft: DeckDraft, catalog: DeckCatalog): DeckDraft {
  return { ...publicDeckDraft(draft), rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion, engineVersion: catalog.engineVersion, updatedAt: new Date().toISOString() };
}

export function setDraftCardCount(draft: DeckDraft, cardId: string, count: number): DeckDraft {
  if (!Number.isSafeInteger(count) || count < 0) return draft;
  const cards = draft.cards.filter(entry => entry.cardId !== cardId);
  if (count > 0) cards.push({ cardId, count });
  return { ...draft, cards };
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function decodeDraft(value: unknown): DeckDraft | null {
  if (!record(value) || typeof value.id !== 'string' || !value.id.trim() || typeof value.name !== 'string' || typeof value.description !== 'string' || !Array.isArray(value.cards)) return null;
  if (value.societyId !== null && (typeof value.societyId !== 'string' || !value.societyId.trim())) return null;
  if (typeof value.rulesVersion !== 'string' || typeof value.cardPoolVersion !== 'string' || typeof value.engineVersion !== 'string' || typeof value.updatedAt !== 'string' || !Number.isFinite(Date.parse(value.updatedAt))) return null;
  const cards: DeckDraft['cards'] = [];
  const seen = new Set<string>();
  for (const entry of value.cards) {
    if (!record(entry) || typeof entry.cardId !== 'string' || !entry.cardId.trim() || typeof entry.count !== 'number' || !Number.isSafeInteger(entry.count) || entry.count < 1 || seen.has(entry.cardId)) return null;
    seen.add(entry.cardId);
    cards.push({ cardId: entry.cardId, count: entry.count });
  }
  return { id: value.id, name: value.name, description: value.description, societyId: value.societyId, cards, rulesVersion: value.rulesVersion, cardPoolVersion: value.cardPoolVersion, engineVersion: value.engineVersion, updatedAt: value.updatedAt };
}

export function localDeckStorage(): DeckStorage | null {
  try { return typeof window === 'undefined' ? null : window.localStorage; } catch { return null; }
}

export function readDeckLibrary(storage: DeckStorage | null = localDeckStorage()): { drafts: DeckDraft[]; warning: string | null } {
  if (!storage) return { drafts: [], warning: '浏览器本地存储不可用；草稿仍可编辑，本次保存无法持久保留。' };
  try {
    const raw = storage.getItem(DECK_LIBRARY_STORAGE_KEY);
    if (!raw) return { drafts: [], warning: null };
    // This bounds malformed local data, not the formal deck construction rules.
    if (raw.length > 2_000_000) throw new Error('oversized storage');
    const envelope: unknown = JSON.parse(raw);
    if (!record(envelope) || envelope.version !== 1 || !Array.isArray(envelope.drafts)) throw new Error('unsupported storage');
    const drafts: DeckDraft[] = [];
    const ids = new Set<string>();
    let discarded = false;
    for (const value of envelope.drafts) {
      const draft = decodeDraft(value);
      if (!draft || ids.has(draft.id)) { discarded = true; continue; }
      ids.add(draft.id); drafts.push(draft);
    }
    return { drafts, warning: discarded ? '部分本地草稿格式损坏，已保留仍可读取的草稿。' : null };
  } catch {
    return { drafts: [], warning: '本地牌组库暂时无法读取，未覆盖原数据；可以新建草稿，保存时将重建牌组库。' };
  }
}

export function saveDeckLibrary(drafts: DeckDraft[], storage: DeckStorage | null = localDeckStorage()): string | null {
  if (!storage) return '浏览器本地存储不可用，草稿尚未保存。';
  if (drafts.some(draft => decodeDraft(draft) === null) || new Set(drafts.map(draft => draft.id)).size !== drafts.length) return '草稿格式有误，无法保存。';
  try {
    storage.setItem(DECK_LIBRARY_STORAGE_KEY, JSON.stringify({ version: 1, drafts: drafts.map(publicDeckDraft) }));
    return null;
  } catch { return '浏览器未能保存牌组，可能空间不足或存储被禁用；当前编辑仍保留在本页。'; }
}
