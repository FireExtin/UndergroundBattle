import type { Action, View } from './types';

export const passTargets = [
  { value: 'action', label: '行动阶段' },
  { value: 'confrontation', label: '对抗阶段' },
  { value: 'end', label: '结束（本回合结束）' },
] as const;
export type PassTarget = typeof passTargets[number]['value'];
export type PassRun = { roomId: string; you: string; turn: number; target: PassTarget; manualEpoch: number };
type Context = { uncertain: boolean; connection: 'connecting' | 'online' | 'offline'; interacting: boolean; manualEpoch: number };
const phaseOrder: Record<string, number> = { start: 0, action: 1, confrontation: 2, end: 3 };

/** These phases are projected directly from Rust Window; end includes cleanup. */
export function passRunStop(view: View, run: PassRun, context: Context): string {
  if (view.roomId !== run.roomId || view.you !== run.you) return '牌桌或席位已改变，连续让过已停止';
  if (view.turn !== run.turn) return '本回合已结束，连续让过已停止';
  if (view.status !== 'playing') return '对局已结束，连续让过已停止';
  if (view.pause) return '牌桌已暂停，连续让过已停止';
  if (view.pendingChoice || view.waitingChoice) return '遇到选择，连续让过已停止，请先完成选择';
  if (view.responseWindow?.members.some(member => member.playerId === view.you && member.status === 'composing')) return '正在编写响应，连续让过已停止';
  if (context.interacting || context.manualEpoch !== run.manualEpoch) return '已开始手动操作，连续让过已停止';
  if (context.connection !== 'online') return '连接中断，连续让过已停止';
  if (context.uncertain) return '上一行动尚未确认，连续让过已停止';
  if (view.players.find(player => player.id === view.you)?.eliminated) return '你已出局，连续让过已停止';
  const phase = phaseOrder[view.phase];
  if (phase === undefined) return '当前阶段尚未确定，连续让过已停止';
  if (run.target !== 'end' && phase >= phaseOrder[run.target]) return `已到达${passTargets.find(target => target.value === run.target)?.label}，连续让过已停止`;
  return '';
}

/** Never begin/cancel a response, select cards, or pay a cost on the player's behalf. */
export function continuousPassAction(view: View, responseRemainingMs: number | null): Action | null {
  const window = view.responseWindow;
  if (window) {
    const member = window.members.find(item => item.playerId === view.you);
    return window.stackTopId === view.stack.at(-1)?.id && member?.status === 'undecided'
      && responseRemainingMs !== null && responseRemainingMs > 0
      ? { kind: 'passResponse', windowId: window.id } : null;
  }
  const passes = view.legalActions.filter(action => action.kind === 'pass');
  return passes.length === 1 ? passes[0] : null;
}
