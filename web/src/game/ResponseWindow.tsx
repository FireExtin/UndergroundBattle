import { useEffect, useMemo, useState } from 'react';
import type { Action, StackEffect, View } from './types';
import './response-intent.css';

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
export function ResponseWindow({ view, busy, uncertain, connection, onSelectCard, onAction }: {
  view: View; busy: boolean; uncertain: boolean;
  connection: 'connecting' | 'online' | 'offline'; onSelectCard: (id: string) => void;
  onAction?: (action: Action) => void;
}) {
  const top = view.stack.at(-1);
  const window = view.responseWindow?.stackTopId === top?.id ? view.responseWindow : null;
  const member = window?.members.find(item => item.playerId === view.you);
  const selection = view.pendingChoice || view.waitingChoice;
  const clockSample = useMemo(() => member?.deadlineMs !== undefined && view.serverNowMs !== undefined
    ? { remainingMs: member.deadlineMs - view.serverNowMs, sampledAt: performance.now() } : null,
  [window?.id, member?.deadlineMs, view.serverNowMs]);
  const [, setClockTick] = useState(0);
  const remainingNow = () => clockSample ? Math.max(0, clockSample.remainingMs - (view.pause ? 0 : Math.max(0, performance.now() - clockSample.sampledAt))) : null;
  const remainingMs = remainingNow();
  const counting = !view.pause && !!window && member?.status === 'undecided' && !selection && remainingMs !== null && remainingMs > 0;
  useEffect(() => {
    if (!counting) return;
    const timer = setInterval(() => setClockTick(tick => tick + 1), 100);
    return () => clearInterval(timer);
  }, [clockSample, counting]);
  if (!top) return null;
  const names = (team: number) => view.players.filter(player => player.team === team).map(player => player.name).join(' / ') || `团队 ${team + 1}`;
  const responses = view.legalActions.filter(action => !['pass', 'choose', 'restart'].includes(action.kind));
  const responseCard = responses.find(action => action.cardId || action.targetId);
  const chooser = view.players.find(player => player.id === selection?.playerId)?.name || '另一位玩家';
  const choosingMembers = window?.members.filter(item => item.playerId !== view.you && item.status === 'composing').map(item => view.players.find(player => player.id === item.playerId)?.name || '玩家').join(' / ');
  const decidingMembers = window?.members.filter(item => item.playerId !== view.you && item.status === 'undecided').map(item => view.players.find(player => player.id === item.playerId)?.name || '玩家').join(' / ');
  const intentStatus = member?.status === 'composing' ? '正在选择响应，确认前可取消'
    : member?.status === 'undecided' ? remainingMs === null ? '正在同步响应决定时限' : remainingMs > 0 ? '请决定是否连锁' : '等待服务器确认响应决定'
    : member?.status === 'passed' ? choosingMembers ? `已让过，等待 ${choosingMembers} 完成响应选择` : decidingMembers ? `已让过，等待 ${decidingMembers} 决定是否连锁` : '已让过，等待服务器确认优先权'
    : choosingMembers ? `等待 ${choosingMembers} 完成响应选择` : '等待持有优先权的玩家决定是否连锁';
  const state = selection ? 'choose' : busy || uncertain || connection !== 'online' ? 'sync'
    : window ? member?.status || 'wait'
    : responses.length ? 'respond' : view.legalActions.some(action => action.kind === 'pass') ? 'pass'
    : top.resolutionState === 'resolving' ? 'resolving' : 'wait';
  const status = view.pause ? '牌桌已暂停；恢复后接续剩余响应时间'
    : view.pendingChoice ? `请你完成选择：${view.pendingChoice.title}`
    : view.waitingChoice ? `等待 ${chooser} 完成选择：${view.waitingChoice.title}`
    : uncertain ? '正在确认上一行动，暂时不能响应'
    : busy ? '正在确认你的响应'
    : connection !== 'online' ? '正在同步牌桌，连接恢复后可响应'
    : window ? intentStatus
    : responses.length ? '你有可用响应，点选可行动的牌查看'
    : view.legalActions.some(action => action.kind === 'pass') ? '你当前只能让过，交出优先权'
    : top.resolutionState === 'resolving' ? '正在结算堆顶效果'
    : '等待持有优先权的玩家响应';
  const commandsEnabled = !view.pause && !!onAction && !busy && !uncertain && connection === 'online' && !selection;
  const decisionEnabled = commandsEnabled && member?.status === 'undecided' && remainingMs !== null && remainingMs > 0;
  const decide = (begin: boolean) => {
    if (!window || !decisionEnabled || !remainingNow() || (begin && !window.canBegin)) return;
    onAction?.(begin ? { kind: 'beginResponse', windowId: window.id, intentId: crypto.randomUUID() } : { kind: 'passResponse', windowId: window.id });
  };
  const cancel = () => {
    if (!window?.myIntentId || member?.status !== 'composing' || !commandsEnabled) return;
    onAction?.({ kind: 'cancelAndPass', windowId: window.id, intentId: window.myIntentId });
  };
  return <section className="hg-response-window" aria-label="当前待结算效果与响应" data-response-state={state} data-stack-top={top.id}>
    <div className="hg-response-heading"><span>堆顶 · 待结算 {view.stack.length}{top.resolutionState === 'resolving' ? ' · 结算中' : ' · 响应窗口'}</span><span>优先权：{names(view.priorityTeam)}</span></div>
    <div className="hg-response-effect"><strong title={top.label}>{top.label}</strong><small>{view.players.find(player => player.id === top.controller)?.name || '玩家'} 发动</small></div>
    <StackTargets effect={top} view={view} />
    <div className="hg-response-status"><span role="status">{status}</span><div className="hg-response-intent-actions">
      {window && !selection && member?.status === 'undecided' && <><button type="button" className="hg-button hg-button-action" disabled={!decisionEnabled || !window.canBegin} onClick={() => decide(true)}>连锁</button><button type="button" className="hg-button hg-button-quiet" disabled={!decisionEnabled} onClick={() => decide(false)}>不连锁，让过</button></>}
      {window && !selection && member?.status === 'composing' && <button type="button" className="hg-button hg-button-quiet" disabled={!commandsEnabled || !window.myIntentId} onClick={cancel}>取消并让过</button>}
      {(window ? state === 'composing' : state === 'respond') && responseCard && <button type="button" className="hg-button hg-button-action" onClick={() => onSelectCard(responseCard.cardId || responseCard.targetId!)}>查看响应牌</button>}
    </div></div>
    {window && !selection && <div className="hg-response-intent-meta">{member?.status === 'undecided' && remainingMs !== null && <span role="timer" aria-label="连锁决定倒计时">连锁决定 · 剩余 {Math.ceil(remainingMs / 1000)} 秒</span>}<small aria-label="响应决定状态">{window.members.map(item => `${view.players.find(player => player.id === item.playerId)?.name || '玩家'} · ${item.status === 'composing' ? '正在选择响应' : item.status === 'passed' ? '已让过' : '尚未决定'}`).join(' / ')}</small></div>}
  </section>;
}
