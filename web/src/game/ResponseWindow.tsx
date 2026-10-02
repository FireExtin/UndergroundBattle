import type { StackEffect, View } from './types';

export function StackTargets({ effect, view }: { effect: StackEffect; view: View }) {
  if (!effect.targetSummaries?.length) return null;
  return <ul className="hg-stack-targets" aria-label="公开目标">{effect.targetSummaries.map((target, index) => {
    const owner = view.players.find(player => player.id === target.owner)?.name;
    return <li key={`${target.instanceId}-${index}`} data-target-instance={target.instanceId} data-target-valid={target.valid} data-target-status={target.status}>
      <span><b>{target.label}</b>{owner && <small> · {owner} 拥有</small>}{target.region !== undefined && <small> · 地区 {target.region + 1}</small>}</span>
      <em className={target.valid ? '' : 'hg-target-invalid'}>{target.status === 'guardAccepted' ? '已进入结算' : target.valid ? '有效目标' : target.invalidReason || '目标已失效'}</em>
    </li>;
  })}</ul>;
}

/** The server's stack and legal actions define the response window; card text is never parsed here. */
export function ResponseWindow({ view, busy, uncertain, connection, onSelectCard }: {
  view: View; busy: boolean; uncertain: boolean;
  connection: 'connecting' | 'online' | 'offline'; onSelectCard: (id: string) => void;
}) {
  const top = view.stack.at(-1);
  if (!top) return null;
  const names = (team: number) => view.players.filter(player => player.team === team).map(player => player.name).join(' / ') || `团队 ${team + 1}`;
  const responses = view.legalActions.filter(action => !['pass', 'choose', 'restart'].includes(action.kind));
  const responseCard = responses.find(action => action.cardId || action.targetId);
  const selection = view.pendingChoice || view.waitingChoice;
  const chooser = view.players.find(player => player.id === selection?.playerId)?.name || '另一位玩家';
  const state = selection ? 'choose' : busy || uncertain || connection !== 'online' ? 'sync'
    : responses.length ? 'respond' : view.legalActions.some(action => action.kind === 'pass') ? 'pass'
    : top.resolutionState === 'resolving' ? 'resolving' : 'wait';
  const status = view.pendingChoice ? `请你完成选择：${view.pendingChoice.title}`
    : view.waitingChoice ? `等待 ${chooser} 完成选择：${view.waitingChoice.title}`
    : uncertain ? '正在确认上一行动，暂时不能响应'
    : busy ? '正在确认你的响应'
    : connection !== 'online' ? '正在同步牌桌，连接恢复后可响应'
    : responses.length ? '你有可用响应，点选可行动的牌查看'
    : view.legalActions.some(action => action.kind === 'pass') ? '你当前只能让过，交出优先权'
    : top.resolutionState === 'resolving' ? '正在结算堆顶效果'
    : '等待持有优先权的玩家响应';
  return <section className="hg-response-window" aria-label="当前待结算效果与响应" data-response-state={state} data-stack-top={top.id}>
    <div className="hg-response-heading"><span>堆顶 · 待结算 {view.stack.length}{top.resolutionState === 'resolving' ? ' · 结算中' : ' · 响应窗口'}</span><span>优先权：{names(view.priorityTeam)}</span></div>
    <div className="hg-response-effect"><strong title={top.label}>{top.label}</strong><small>{view.players.find(player => player.id === top.controller)?.name || '玩家'} 发动</small></div>
    <StackTargets effect={top} view={view} />
    <div className="hg-response-status"><span role="status">{status}</span>{state === 'respond' && responseCard && <button type="button" className="hg-button hg-button-action" onClick={() => onSelectCard(responseCard.cardId || responseCard.targetId!)}>查看响应牌</button>}</div>
  </section>;
}
