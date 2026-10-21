import raw from '../../../rust-game/data/cards.json';
import type { DeckCatalog } from '../game/deckLibrary';
import { UW_PLAYER_MAPPINGS, UW_SOCIETY_MAPPINGS } from '../game/uwDeckManifest';

// Print data comes from the repository. Only the public server metadata needed
// by the existing deck validator is supplied here; this is not a rule oracle.
export const importCatalog: DeckCatalog = {
  rulesVersion: raw.rulesVersion, cardPoolVersion: raw.cardPoolVersion, engineVersion: raw.engineVersion,
  decks: [], deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: true },
  cards: raw.cards.map(card => ({ ...card, cost: card.cost ?? 0, supported: true,
    deckCopyLimit: (card.keywords as string[] | undefined)?.includes('唯一') ? 1 : card.id === 'JC125' ? null : 3,
    permanentIcons: card.icons.permanent, temporaryIcons: card.icons.temporary,
  })) as unknown as DeckCatalog['cards'],
  societies: UW_SOCIETY_MAPPINGS.map(item => ({ id: item.targetId, name: item.name, kind: 'society', cost: 0, text: '', supported: true,
    startingHand: 6, deckConstraints: item.targetId === 'MSJC09' ? [] : [{ kind: 'minimumColor', color: ({ MSJC03: '蓝', MSJC04: '红', MSJC05: '灰', MSJC07: '黑' } as Record<string, string>)[item.targetId], count: 25 }],
    unresolvedAbilities: {},
  })),
};
export const mapping = (id: string) => UW_PLAYER_MAPPINGS.find(item => item.targetId === id)!;
export const entry = (id: string, copies = 1) => ({ templatePath: mapping(id).sourcePath, copies });
export const uwText = (cards: unknown[], councils?: unknown[]) => JSON.stringify({ deckName: 'UW导入牌组', cards, ...(councils ? { councils } : {}) });
export function validImportText() {
  let left = 50;
  const cards = UW_PLAYER_MAPPINGS.filter(item => !item.quarantine).flatMap(item => {
    const copyLimit = importCatalog.cards.find(card => card.id === item.targetId)!.deckCopyLimit!;
    const count = Math.min(left, copyLimit); left -= count;
    return count ? [entry(item.targetId, count)] : [];
  });
  if (left) throw new Error('Insufficient reviewed test identities');
  return uwText(cards);
}
