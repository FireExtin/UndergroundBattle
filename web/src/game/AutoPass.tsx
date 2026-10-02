import { useEffect, useRef, useState } from 'react';
import type { Action, View } from './types';

const PREFERENCE_KEY = 'hegemony.autoPass.v1';
function readPreference() {
  try { return localStorage.getItem(PREFERENCE_KEY) === 'true'; }
  catch { return false; }
}

/** Opt-in convenience for empty windows; every submission still follows the command pipeline. */
export function AutoPass({ view, busy, uncertain, connection, onAction }: {
  view: View; busy: boolean; uncertain: boolean;
  connection: 'connecting' | 'online' | 'offline'; onAction: (action: Action) => void;
}) {
  const [enabled, setEnabled] = useState(readPreference);
  const [lastAutoKey, setLastAutoKey] = useState('');
  const sent = useRef('');
  const submit = useRef(onAction);
  submit.current = onAction;
  const pass = view.legalActions.length === 1 && view.legalActions[0].kind === 'pass' ? view.legalActions[0] : null;
  const key = pass ? `${view.roomId}:${view.version}:${pass.id}` : '';
  const canPass = enabled && view.status === 'playing' && !view.pendingChoice && !view.waitingChoice
    && !!pass && !busy && !uncertain && connection === 'online';

  useEffect(() => {
    if (!canPass || !pass || sent.current === key) return;
    const timer = setTimeout(() => {
      if (sent.current === key) return;
      sent.current = key; setLastAutoKey(key);
      submit.current(pass);
    }, 550);
    return () => clearTimeout(timer);
    // New versions or changed eligibility cancel the pending window before a submission.
  }, [canPass, key]);

  const status = !enabled ? '关闭 · 由你手动让过'
    : view.status !== 'playing' ? '对局已结束，自动让过已暂停'
    : view.pendingChoice || view.waitingChoice ? '正在等待选择，自动让过已暂停'
    : uncertain || busy ? '等待上一行动确认'
    : connection !== 'online' ? '连接恢复后继续'
    : !pass ? '有可用行动或等待他人，自动让过已暂停'
    : lastAutoKey === key ? '已自动让过，等待牌桌同步'
    : '当前仅能让过，将自动继续';

  return <div className={`hg-auto-pass ${enabled ? 'hg-auto-pass-enabled' : ''}`}>
    <label><input type="checkbox" role="switch" aria-label="无可用行动时自动让过" checked={enabled} onChange={event => {
      setEnabled(event.target.checked);
      try { localStorage.setItem(PREFERENCE_KEY, String(event.target.checked)); } catch { /* This tab still remembers the preference. */ }
    }} /><span>无可用行动时自动让过</span></label>
    <span role="status">{status}</span><small>仅当唯一可用动作是“让过”时生效，可随时关闭。</small>
  </div>;
}
