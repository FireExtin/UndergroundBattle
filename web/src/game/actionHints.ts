import type { Card, CardDefinition, LegalAction, View } from './types';

const labels = { standard: '标准', fast: '快速', actionFast: '行动阶段快速', privilege: '先手特权', other: '其他' };
type ActionType = keyof typeof labels;
const order: ActionType[] = ['standard', 'fast', 'actionFast', 'privilege', 'other'];

function actionType(action: LegalAction, source?: Card, definitions?: Map<string, CardDefinition>): ActionType {
  // These are the engine's built-in action kinds, independent of card text.
  if (['asset', 'deploy', 'conceal'].includes(action.kind)) return 'standard';
  if (action.kind === 'reveal') return 'fast';
  if (action.kind === 'privilege') return 'privilege';
  const ability = source?.cardId && definitions?.get(source.cardId)?.abilities?.find(item => item.key === action.abilityId && !item.triggered);
  return ability ? ability.timing : 'other';
}

/** Display current server candidates only. Never infer availability from costs, text or phase. */
export function actionHints(view: View, cards: Card[], definitions: Map<string, CardDefinition>, suspended: boolean) {
  const byCard = new Map<string, Set<ActionType>>();
  const types = new Set<ActionType>();
  const response = view.responseWindow;
  const composing = response?.myIntentId && response.stackTopId === view.stack.at(-1)?.id
    && view.stack.at(-1)?.resolutionState !== 'resolving'
    && response.members.some(member => member.playerId === view.you && member.status === 'composing');
  if (!suspended && view.status === 'playing' && !view.pause && !view.pendingChoice && !view.waitingChoice && (!response || composing)) {
    const sources = new Map(cards.map(card => [card.instanceId, card]));
    for (const action of view.legalActions) {
      if (['pass', 'choose', 'beginResponse', 'passResponse', 'cancelResponse', 'deck', 'ready', 'start', 'restart'].includes(action.kind)) continue;
      // targetId is a destination, not evidence that its card can act.
      const source = action.cardId ? sources.get(action.cardId) : undefined;
      if (action.cardId && (!source || source.faceDown && source.controller !== view.you)) continue;
      const type = actionType(action, source, definitions);
      types.add(type);
      if (source) {
        const cardTypes = byCard.get(source.instanceId) || new Set<ActionType>();
        cardTypes.add(type); byCard.set(source.instanceId, cardTypes);
      }
    }
  }
  return {
    cards: new Map([...byCard].map(([id, values]) => [id, `候选·${order.filter(type => values.has(type)).map(type => labels[type]).join('/')}`])),
    types: order.filter(type => types.has(type)).map(type => type === 'privilege' ? labels[type] : `${labels[type]}行动`),
  };
}
