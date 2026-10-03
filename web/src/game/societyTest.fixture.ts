// Synthetic public projections for tests only; no production entry imports this registry.
import { testCatalog, testView } from './testFixtures';
import type { Catalog, Card, SocietyDefinition, View } from './types';

export const fixtureSociety: SocietyDefinition = {
  id: 'FIXTURE_SOCIETY_ALPHA', name: '合成秘社甲', kind: 'society', cost: 0, supported: true,
  text: '合成 UI 测试：支付费用并横置本秘社，选择一个已显示目标。',
  startingHand: 4, deckConstraints: [{ kind: 'minimumColor', color: '红', count: 2 }],
  unresolvedAbilities: { FIXTURE_UNRESOLVED: 'fixture-pending-interpretation' },
  abilities: [
    { key: 'FIXTURE_TARGET', label: '合成目标能力', timing: 'standard', triggered: false },
    { key: 'FIXTURE_UNRESOLVED', label: '合成未决能力', timing: 'fast', triggered: false },
  ],
};
export const fixtureSocietyCatalog: Catalog = {
  ...testCatalog, engineVersion: 'FIXTURE_UI_ONLY',
  deckBuildRules: { minimumCards: 50, serviceCardCapacity: 2048, societySupported: true },
  cards: [...testCatalog.cards.map(card => ({ ...card, supported: true, deckCopyLimit: null })),
    { id: 'FIXTURE_RED', name: '合成红色角色', kind: 'character', cost: 0, color: '红', text: '仅用于 UI 测试', supported: true, deckCopyLimit: null }],
  societies: [fixtureSociety, { ...fixtureSociety, id: 'FIXTURE_SOCIETY_BETA', name: '合成秘社丙', startingHand: 5, deckConstraints: [] }],
};
export const fixtureSocietyCard: Card = {
  instanceId: 'fixture-society-p0', cardId: fixtureSociety.id, name: fixtureSociety.name,
  owner: 'p0', controller: 'p0', kind: 'society', faceDown: false, exhausted: false, text: fixtureSociety.text,
};
export const fixtureSocietyView: View = {
  ...testView, roomId: 'FIXTURE_UI_ONLY', status: 'playing', version: 1,
  players: Array.from({ length: 4 }, (_, seat) => ({ ...testView.players[0], id: `p${seat}`, seat, name: ['甲', '乙', '丙', '丁'][seat], team: seat < 2 ? 0 : 1 })),
  mode: 'teams', firstTeam: 1,
  societyZones: [
    { id: 'society:p0', playerId: 'p0', card: fixtureSocietyCard },
    { id: 'society:p1', playerId: 'p1', card: null },
    { id: 'society:p2', playerId: 'p2', card: { ...fixtureSocietyCard, instanceId: 'fixture-society-p2', cardId: 'FIXTURE_SOCIETY_BETA', owner: 'p2', controller: 'p2', name: '合成秘社丙', exhausted: true } },
    { id: 'society:p3', playerId: 'p3', card: null },
  ],
  graveyard: [{ ...testView.hand[0], instanceId: 'fixture-grave-p1', owner: 'p1', controller: 'p1', name: '合成墓地目标' }],
  legalActions: [
    { id: 'fixture-target', kind: 'activate', cardId: fixtureSocietyCard.instanceId, abilityId: 'FIXTURE_TARGET', sourceZoneId: 'society:p0', targetId: 'fixture-grave-p1', label: '合成秘社甲：合成目标能力 → 合成墓地目标' },
    { id: 'pass', kind: 'pass', label: '让过' },
  ],
};
