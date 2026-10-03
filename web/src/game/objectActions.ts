import type { LegalAction } from './types';

export type ObjectActionGroup = { key: string; label: string; actions: LegalAction[] };

// Only collapse destination variants. Payload fields preserve ability/mode/costs;
// display differences also split groups so older room metadata cannot hide a choice.
export function groupObjectActions(actions: LegalAction[]): ObjectActionGroup[] {
  const groups = new Map<string, ObjectActionGroup>();
  for (const action of actions) {
    const payload = Object.fromEntries(Object.entries(action)
      .filter(([key]) => !['id', 'label', 'description', 'targetId', 'region'].includes(key))
      .sort(([a], [b]) => a.localeCompare(b)));
    const arrow = action.label.indexOf('→');
    const cost = action.label.indexOf('（费用：');
    const label = arrow < 0 ? action.label
      : action.label.slice(0, arrow).trim() + (cost > arrow ? action.label.slice(cost) : '');
    const key = JSON.stringify([payload, action.targetId !== undefined, action.region !== undefined, label, action.description]);
    const group = groups.get(key);
    if (group) group.actions.push(action);
    else groups.set(key, { key, actions: [action], label });
  }
  return [...groups.values()];
}

export function actionSource(action: LegalAction): string | undefined {
  return action.cardId || action.targetId;
}
