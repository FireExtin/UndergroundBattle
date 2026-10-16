import { useEffect, useMemo, useRef, useState } from 'react';
import type { Action, View } from './types';
import { continuousPassAction, passRunStop, passTargets } from './continuousPass';
import type { PassRun, PassTarget } from './continuousPass';
import './continuous-pass.css';

const noop = () => {};

/** Opt-in convenience for empty windows; every submission still follows the command pipeline. */
export function AutoPass({ view, busy, uncertain, connection, onAction, enabled, onEnabledChange, interacting = false, manualEpoch = 0, onActivityChange = noop }: {
  view: View; busy: boolean; uncertain: boolean;
  connection: 'connecting' | 'online' | 'offline'; onAction: (action: Action) => void;
  enabled: boolean; onEnabledChange: (enabled: boolean) => void;
  interacting?: boolean; manualEpoch?: number; onActivityChange?: (active: boolean) => void;
}) {
  const [target, setTarget] = useState<PassTarget>('end');
  // Deliberately local: refresh, navigation and a changed table/seat never re-arm a run.
  const [run, setRun] = useState<PassRun | null>(null);
  const [notice, setNotice] = useState('');
  const [lastAutoKey, setLastAutoKey] = useState('');
  const sent = useRef('');
  const submit = useRef(onAction);
  submit.current = onAction;
  const context = { uncertain, connection, interacting, manualEpoch };
  const stopReason = run ? passRunStop(view, run, context) : '';
  const active = !!run && !stopReason;
  const member = view.responseWindow?.members.find(item => item.playerId === view.you);
  const clock = useMemo(() => member?.deadlineMs !== undefined && view.serverNowMs !== undefined
    ? { remaining: member.deadlineMs - view.serverNowMs, at: performance.now() } : null,
  [view.responseWindow?.id, member?.deadlineMs, view.serverNowMs]);
  const remaining = () => clock ? Math.max(0, clock.remaining - (performance.now() - clock.at)) : null;
  const legacyPass = enabled && !run && !view.responseWindow && view.legalActions.length === 1 && view.legalActions[0].kind === 'pass' ? view.legalActions[0] : null;
  const pass = active ? continuousPassAction(view, remaining()) : legacyPass;
  const key = pass ? JSON.stringify([view.roomId, view.you, view.version, pass.kind, pass.windowId]) : '';
  const canPass = (active || enabled && !run) && !view.pause && view.status === 'playing' && !view.pendingChoice && !view.waitingChoice
    && !interacting && !!pass && !busy && !uncertain && connection === 'online';
  const latest = useRef({ canPass, key, pass, active, remaining });
  latest.current = { canPass, key, pass, active, remaining };

  useEffect(() => {
    if (run && stopReason) { setRun(null); setNotice(stopReason); }
  }, [run, stopReason]);
  useEffect(() => { onActivityChange(active); return () => onActivityChange(false); }, [active, onActivityChange]);

  useEffect(() => {
    if (!canPass || !pass || sent.current === key) return;
    const timer = setTimeout(() => {
      const current = latest.current;
      if (!current.canPass || current.key !== key || !current.pass || sent.current === key) return;
      if (current.active && current.pass.kind === 'passResponse' && !((current.remaining() ?? 0) > 0)) return;
      sent.current = key; setLastAutoKey(key);
      submit.current(current.pass);
    }, 550);
    return () => clearTimeout(timer);
    // New versions or changed eligibility cancel the pending window before a submission.
  }, [canPass, key]);

  const status = active ? '连续模式已开启'
    : !enabled ? '关闭 · 由你手动让过'
    : view.pause ? '牌桌已暂停，自动让过已暂停'
    : view.status !== 'playing' ? '对局已结束，自动让过已暂停'
    : view.pendingChoice || view.waitingChoice ? '正在等待选择，自动让过已暂停'
    : view.responseWindow ? '连锁决定期间，自动让过已暂停'
    : uncertain || busy ? '等待上一行动确认'
    : connection !== 'online' ? '连接恢复后继续'
    : !pass ? '有可用行动或等待他人，自动让过已暂停'
    : lastAutoKey === key ? '已自动让过，等待牌桌同步'
    : '当前仅能让过，将自动继续';

  const proposed: PassRun = { roomId: view.roomId, you: view.you, turn: view.turn, target, manualEpoch };
  const startBlocked = passRunStop(view, proposed, context);
  const cancel = () => { setRun(null); setNotice('已取消连续让过'); onActivityChange(false); };

  return <div className={`hg-auto-pass ${enabled ? 'hg-auto-pass-enabled' : ''}`}>
    <label><input type="checkbox" role="switch" aria-label="无可用行动时自动让过" checked={enabled} onChange={event => { if (run) cancel(); onEnabledChange(event.target.checked); }} /><span>无可用行动时自动让过</span></label>
    <span role="status">{status}</span><small>仅当唯一可用动作是“让过”时生效，可随时关闭。</small>
    <section className="hg-continuous-pass" aria-label="让过到指定阶段" data-continuous-pass={active ? 'active' : 'off'}>
      <label>让过到<select aria-label="连续让过的目标阶段" value={target} disabled={!!run || busy} onChange={event => setTarget(event.target.value as PassTarget)}>{passTargets.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select></label>
      {run ? <button type="button" className="hg-button hg-button-primary" onClick={cancel}>取消连续让过</button>
        : <button type="button" className="hg-button hg-button-quiet" disabled={busy || !!startBlocked} onClick={() => { if (busy || startBlocked) return; onEnabledChange(false); setNotice(''); setRun(proposed); }}>一键连续让过</button>}
      <span role="status" aria-label="连续让过状态">{active ? `连续让过已开启 · 至${passTargets.find(option => option.value === run?.target)?.label}${busy || uncertain ? ' · 等待上一行动确认' : !pass ? ' · 等待你的合法窗口' : lastAutoKey === key ? ' · 已提交让过，等待同步' : ''}` : notice || startBlocked || '连续让过已关闭'}</span>
      <small>仅让过你的合法窗口；遇到选择、暂停或断线停止，刷新后关闭。“结束”处理本回合结束窗口，不自动让过下一回合。</small>
    </section>
  </div>;
}
