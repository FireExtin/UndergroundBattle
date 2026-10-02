import type { CardDefinition, Deck } from './types';

// Reviewed public names/colors only: docs/factions/README.md, “官方颜色及实际覆盖”.
// Runtime availability comes exclusively from the live catalog, not the research index.
export const factions = [
  { id: 'yellow', color: '黄', name: '帷幕守望' },
  { id: 'green', color: '绿', name: '猎魔人' },
  { id: 'blue', color: '蓝', name: '王座会' },
  { id: 'red', color: '红', name: '鸣钟教派' },
  { id: 'gray', color: '灰', name: '国家机构' },
  { id: 'white', color: '白', name: '圣贤' },
  { id: 'black', color: '黑', name: '方碑序列' },
  { id: 'purple', color: '紫', name: '梦境行者' },
] as const;
export const neutral = { id: 'neutral', color: '褐', name: '中立（无派系）' } as const;
const unknown = { id: 'unknown', color: '—', name: '颜色未标' } as const;
export type Faction = typeof factions[number] | typeof neutral | typeof unknown;
export type ColorCount = { faction: Faction; count: number };

function factionFor(color?: string): Faction {
  const normalized = color?.trim().replace(/色$/, '');
  if (normalized === '中立' || normalized === '褐') return neutral;
  return factions.find(faction => faction.color === normalized) || unknown;
}
function playerDefinitions(cards: CardDefinition[]) {
  return [...new Map(cards.filter(card => card.supported !== false && !['region', 'society'].includes(card.kind))
    .map(card => [card.id, card])).values()];
}

export function factionCoverage(cards: CardDefinition[]) {
  const counts = new Map<string, number>();
  const players = playerDefinitions(cards);
  for (const card of players) {
    const id = factionFor(card.color).id;
    counts.set(id, (counts.get(id) || 0) + 1);
  }
  return {
    factions: factions.map(faction => ({ faction, count: counts.get(faction.id) || 0 })),
    neutralCount: counts.get(neutral.id) || 0,
    unknownCount: counts.get(unknown.id) || 0,
    playerCount: players.length,
  };
}

export function deckColors(deck: Deck, cards: CardDefinition[]) {
  const definitions = new Map(playerDefinitions(cards).map(card => [card.id, card]));
  const counts = new Map<string, ColorCount>();
  for (const entry of deck.cards) {
    const faction = factionFor(definitions.get(entry.cardId)?.color);
    const previous = counts.get(faction.id);
    counts.set(faction.id, { faction, count: (previous?.count || 0) + entry.count });
  }
  const colors = [...counts.values()].filter(item => item.count > 0);
  const main = colors.filter(item => item.faction.id !== 'neutral' && item.faction.id !== 'unknown')
    .sort((a, b) => b.count - a.count || factions.findIndex(faction => faction.id === a.faction.id) - factions.findIndex(faction => faction.id === b.faction.id))[0]?.faction || neutral;
  const order = [...factions.map(faction => faction.id), 'neutral', 'unknown'];
  colors.sort((a, b) => a.faction.id === main.id ? -1 : b.faction.id === main.id ? 1 : order.indexOf(a.faction.id) - order.indexOf(b.faction.id));
  return { colors, main, total: colors.reduce((sum, item) => sum + item.count, 0) };
}
