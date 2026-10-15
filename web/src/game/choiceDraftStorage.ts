import { playerStorage } from './playerStorage';
import type { Choice } from './types';

export type ChoiceDraft = {
  selected: string[];
  top: string[];
  bottom: string[];
  allocations: Record<string, number>;
};
export type ChoiceDraftStore = { storage: Storage; key: string };

// Choice edits belong to this tab, even when ordinary seat credentials are shared.
// Capture the player namespace so later cleanup cannot touch a different player.
export function createChoiceDraftStore(roomId: string, viewerId: string): ChoiceDraftStore | undefined {
  try {
    return { storage: sessionStorage, key: `hegemony.choiceDraft.v1.${JSON.stringify([playerStorage().scope, roomId, viewerId])}` };
  } catch { return undefined; }
}

export function choiceDraftFingerprint(choice: Choice): string {
  return JSON.stringify([choice.id, choice.playerId, choice.kind, choice.min ?? 1,
    choice.max ?? choice.options.length, choice.amount ?? 0, !!choice.allowDecline,
    choice.options.map(option => [option.id, option.card?.instanceId ?? null]),
    choice.previewCards?.map(card => card.instanceId) ?? []]);
}

export function initialChoiceDraft(choice: Choice): ChoiceDraft {
  const ids = choice.options.map(option => option.id);
  return { selected: choice.kind === 'region_return' ? [...ids] : [], top: ids, bottom: [], allocations: {} };
}

function validDraft(value: unknown, choice: Choice): value is ChoiceDraft {
  if (!value || typeof value !== 'object') return false;
  const draft = value as ChoiceDraft;
  const ids = new Set(choice.options.map(option => option.id));
  const selection = (list: unknown): list is string[] => Array.isArray(list)
    && list.every(id => typeof id === 'string' && ids.has(id)) && new Set(list).size === list.length;
  if (!selection(draft.selected) || draft.selected.length > (choice.max ?? ids.size)
    || !selection(draft.top) || !selection(draft.bottom)) return false;
  const ordered = [...draft.top, ...draft.bottom];
  if (ordered.length !== ids.size || new Set(ordered).size !== ids.size) return false;
  if (!draft.allocations || typeof draft.allocations !== 'object' || Array.isArray(draft.allocations)) return false;
  const allocations = Object.entries(draft.allocations);
  return allocations.every(([id, amount]) => ids.has(id) && Number.isSafeInteger(amount) && amount >= 0)
    && allocations.reduce((total, [, amount]) => total + amount, 0) <= (choice.amount ?? 0);
}

export function readChoiceDraft(store: ChoiceDraftStore | undefined, choice: Choice): ChoiceDraft | null {
  if (!store) return null;
  try {
    const record = JSON.parse(store.storage.getItem(store.key) || 'null');
    return record?.fingerprint === choiceDraftFingerprint(choice) && validDraft(record.draft, choice) ? record.draft : null;
  } catch { return null; }
}

export function saveChoiceDraft(store: ChoiceDraftStore | undefined, choice: Choice, draft: ChoiceDraft) {
  if (!store) return;
  try { store.storage.setItem(store.key, JSON.stringify({ fingerprint: choiceDraftFingerprint(choice), draft })); }
  catch {
    // Avoid restoring an older edit if a newer write failed, for example at quota.
    try { store.storage.removeItem(store.key); } catch { /* Keep in-memory editing available. */ }
  }
}

// Only an authoritative view closes a draft. Unmounting or sending a command does not.
export function reconcileChoiceDraft(store: ChoiceDraftStore | undefined, choice: Choice | null) {
  if (!store || choice && readChoiceDraft(store, choice)) return;
  try { store.storage.removeItem(store.key); }
  catch { /* A denied storage operation must not block the table. */ }
}
