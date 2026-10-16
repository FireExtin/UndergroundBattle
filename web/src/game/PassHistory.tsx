import { phaseLabel } from './phaseLabel';
import type { PassRecord, View } from './types';
import './pass-history.css';

export function passRecordText(record: PassRecord) {
  const match = /^region:\d+:([^:]+)(?::(before|after))?$/.exec(record.step);
  const step = match ? `${phaseLabel(match[1])}${match[2] ? ` · ${match[2] === 'before' ? '奖励前' : '奖励后'}快速窗口` : ''}` : phaseLabel(record.step);
  const stage = [...new Set([phaseLabel(record.phase), step])].join(' · ');
  const region = record.region ? `地区 ${record.region.index + 1}${record.region.name ? ` · ${record.region.name}` : ''}` : '全局';
  const action = record.reason === 'timeout' ? '响应超时，自动让过' : record.reason === 'cancelResponse' ? '取消响应并让过' : record.reason === 'response' ? '响应让过' : '让过';
  const seat = /^p([0-3])$/.exec(record.playerId);
  const actor = `${record.playerName}${seat ? `（席位 ${Number(seat[1]) + 1}）` : ''}`;
  return `${actor} · 第 ${record.turn} 回合 · ${stage} · ${region} · ${action}`;
}

export function LatestPass({ history }: { history: View['passHistory'] }) {
  const latest = history?.at(-1);
  return latest ? <div className="hg-latest-pass" role="status" aria-label="最近让过"><b>最近让过</b><span>{passRecordText(latest)}</span></div> : null;
}

export function TableRecords({ view }: { view: View }) {
  const entries = [
    ...(view.passHistory || []).map(entry => ({ key: `pass:${entry.id}`, version: entry.version, text: passRecordText(entry), pass: true })),
    ...view.log.map((entry, index) => ({ key: `log:${entry.version}:${index}`, ...entry, pass: false })),
  ].sort((a, b) => a.version - b.version).reverse().slice(0, 50);
  return <details className="hg-table-records"><summary>记录 · {entries.length}</summary>
    <ol aria-label="牌桌行动记录">{entries.map(entry => <li key={entry.key} data-pass-record={entry.pass || undefined}>{entry.text}</li>)}</ol>
    <small>最近 50 条公开记录 · 规则 {view.versions.rules} · 卡池 {view.versions.cardPool} · 引擎 {view.versions.engine}</small>
  </details>;
}
