import type { LegalAction } from './types';

export type ObjectActionGroup = { key: string; label: string; actions: LegalAction[] };

// Only collapse destination variants. Ability, mode, costs and every other payload
// field remain part of the key; the selected original action is submitted intact.
export function groupObjectActions(actions: LegalAction[]): ObjectActionGroup[] {
  const groups = new Map<string, ObjectActionGroup>();
  for (const action of actions) {
    const payload = Object.fromEntries(Object.entries(action)
      .filter(([key]) => !['id', 'label', 'description', 'targetId', 'region'].includes(key))
      .sort(([a], [b]) => a.localeCompare(b)));
    const key = JSON.stringify([payload, action.targetId !== undefined, action.region !== undefined]);
    const group = groups.get(key);
    if (group) group.actions.push(action);
    else {
      const arrow = action.label.indexOf('→');
      const cost = action.label.indexOf('（费用：');
      groups.set(key, { key, actions: [action], label: arrow < 0 ? action.label
        : action.label.slice(0, arrow).trim() + (cost > arrow ? action.label.slice(cost) : '') });
    }
  }
  return [...groups.values()];
}

export function actionSource(action: LegalAction): string | undefined {
  return action.cardId || action.targetId;
}
