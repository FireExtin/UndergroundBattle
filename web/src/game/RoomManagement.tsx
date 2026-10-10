import { useState } from 'react';
import type { Action, View } from './types';
import './room-management.css';

export function RoomManagement({ view, blocked, onAction, refresh }: {
  view: View; blocked: boolean; onAction: (action: Action) => void; refresh: () => void;
}) {
  const [copyStatus, setCopyStatus] = useState('');
  if (!view.canPause) return null;
  const pauser = view.pause && view.players.find(player => player.seat === view.pause!.pausedBy)?.name;
  return <section className="hg-room-management" aria-label="保存与继续牌桌">
    <strong>{view.pause ? `牌桌已暂停${pauser ? ` · ${pauser}` : ''}` : '保存此桌，约好时间继续打'}</strong>
    <p>{view.pause ? '已确认的牌桌、手牌、堆栈和选择已保存在服务器。暂停时不计时、不自动让过；恢复后只接续剩余响应时间。' : '任一已入席玩家都可暂停或恢复此桌。返回大厅、关闭页面不会自动暂停；要改天继续，请先暂停并保存。'}</p>
    <p>未提交的本地编辑不属于存档。伙伴用原浏览器的“我的牌桌”恢复原席，也可通过同一邀请链接找到此桌；邀请码不能找回丢失的座位凭证。</p>
    <div>
      <button className="hg-button hg-button-primary" disabled={blocked} onClick={() => onAction({ kind: view.pause ? 'resumeRoom' : 'pauseRoom' })}>{view.pause ? '恢复对局' : '暂停并保存'}</button>
      <button className="hg-button hg-button-quiet" disabled={blocked} onClick={refresh}>查看最新状态</button>
      <button className="hg-button hg-button-quiet" onClick={async () => {
        try { await navigator.clipboard.writeText(`${location.origin}/?invite=${encodeURIComponent(view.inviteCode)}`); setCopyStatus('此桌邀请链接已复制'); }
        catch { setCopyStatus(`无法复制，请分享邀请码 ${view.inviteCode}`); }
      }}>复制此桌邀请链接</button>
    </div>
    {view.pause && <p>暂停的牌桌不自动轮询。伙伴恢复后，点“查看最新状态”或刷新页面同步。</p>}
    {copyStatus && <p role="status">{copyStatus}</p>}
  </section>;
}
