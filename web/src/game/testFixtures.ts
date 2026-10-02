import type { Card, Catalog, Choice, View } from './types';
export const testCard: Card = { instanceId: 'instance-a', cardId: 'JC125', name: '无知路人', kind: 'character', owner: 'p0', controller: 'p0', exhausted: false, faceDown: false, cost: 0, text: '真实印刷文字', icons: { investigation: 0, combat: 0, influence: 1 }, defense: 1 };
export const testCatalog: Catalog = {
  rulesVersion: 'rules1', cardPoolVersion: 'pool1', engineVersion: 'engine1',
  decks: [{ id: 'watchers', name: '调查者', description: '控制与调查', cardCount: 50, cards: [{ cardId: 'JC125', count: 50 }] }],
  cards: [{ id: 'JC125', name: '无知路人', kind: 'character', cost: 0, text: '真实印刷文字', loyalty: [], permanentIcons: { investigation: 0, combat: 0, influence: 1 } }],
};
export const testView: View = {
  roomId: 'room-test', inviteCode: 'INVITE', version: 1, mode: 'duel', status: 'lobby', you: 'p0',
  players: [{ id: 'p0', seat: 0, name: '甲', team: 0, deckId: 'watchers', ready: false, eliminated: false, handCount: 1, deckCount: 49, score: 0 }],
  firstTeam: 0, activeTeam: 0, priorityTeam: 0, turn: 1, phase: 'action', step: 'action', winScore: 8,
  regions: [], hand: [testCard], assets: [], graveyard: [], scoreCards: [], stack: [], pendingChoice: null,
  legalActions: [{ id: 'ready', kind: 'ready', label: '准备' }], log: [], versions: { rules: '1', cardPool: '1', engine: '1' },
};
export const testChoice: Choice = {
  id: 'choice-one', kind: 'target', title: '选择目标', description: '请选择一个角色', playerId: 'p0', min: 1, max: 1,
  options: [{ id: 'a', label: '角色甲' }, { id: 'b', label: '角色乙' }],
};
