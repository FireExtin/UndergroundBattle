import { decodeDraft, publicDeckDraft } from './deckLibrary';
import type { DeckDraft } from './deckLibrary';

export type CollectionFilters = {
  search: string; faction: string; kind: string; cost: string; feature: string;
  implementation: string; selectedOnly: boolean; sort: string;
  density: 'compact' | 'comfortable';
};
export const defaultCollectionFilters: CollectionFilters = {
  search: '', faction: 'all', kind: 'all', cost: 'all', feature: 'all',
  implementation: 'implemented', selectedOnly: false, sort: 'catalog', density: 'compact',
};
const WORKSPACE_KEY = 'hegemony.deckWorkspace.v1';
const SELECTION_KEY = 'hegemony.lobbyDeckSelection.v1';
type Workspace = { draft: DeckDraft; dirty: boolean; filters: CollectionFilters };
export type LobbySelection = { kind: 'preset'; deckId: string } | { kind: 'draft'; draft: DeckDraft };
const record = (value: unknown): value is Record<string, unknown> => !!value && typeof value === 'object' && !Array.isArray(value);

function read(key: string): unknown {
  try {
    const raw = sessionStorage.getItem(key);
    return raw && raw.length <= 1_000_000 ? JSON.parse(raw) : null;
  } catch { return null; }
}
function write(key: string, value: unknown): boolean {
  try { sessionStorage.setItem(key, JSON.stringify(value)); return true; } catch { return false; }
}

export function readDeckWorkspace(): Workspace | null {
  const value = read(WORKSPACE_KEY);
  if (!record(value)) return null;
  const draft = decodeDraft(value.draft);
  if (!draft || !record(value.filters) || typeof value.dirty !== 'boolean') return null;
  const filters = { ...defaultCollectionFilters };
  for (const key of ['search', 'faction', 'kind', 'cost', 'feature'] as const) {
    if (typeof value.filters[key] === 'string') filters[key] = value.filters[key].slice(0, 500);
  }
  if (['implemented', 'unavailable', 'all'].includes(String(value.filters.implementation))) filters.implementation = String(value.filters.implementation);
  if (['catalog', 'cost-up', 'cost-down', 'selected', 'name'].includes(String(value.filters.sort))) filters.sort = String(value.filters.sort);
  filters.selectedOnly = value.filters.selectedOnly === true;
  filters.density = value.filters.density === 'comfortable' ? 'comfortable' : 'compact';
  return { draft, dirty: value.dirty, filters };
}
export function saveDeckWorkspace(value: Workspace): boolean {
  return write(WORKSPACE_KEY, { ...value, draft: publicDeckDraft(value.draft) });
}
export function readLobbySelection(): LobbySelection | null {
  const value = read(SELECTION_KEY);
  if (!record(value)) return null;
  if (value.kind === 'preset' && typeof value.deckId === 'string' && value.deckId.length <= 200) return { kind: 'preset', deckId: value.deckId };
  const draft = value.kind === 'draft' ? decodeDraft(value.draft) : null;
  return draft ? { kind: 'draft', draft } : null;
}
export function saveLobbySelection(value: LobbySelection): boolean {
  return write(SELECTION_KEY, value.kind === 'draft' ? { kind: 'draft', draft: publicDeckDraft(value.draft) } : value);
}
