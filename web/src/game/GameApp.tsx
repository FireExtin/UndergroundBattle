import { useState } from 'react';
import { Help } from './Help';
import { Lobby } from './Lobby';
import { RoomLobby, Table } from './Table';
import { useGame } from './useGame';
import './game.css';

export function GameApp() {
  const game = useGame();
  const [help, setHelp] = useState(false);
  return <div className="hg-app">
    <header className="hg-header"><a href="/" className="hg-brand"><span className="hg-brand-mark">◈</span><span>隐秘世界 <b>霸权</b><small>SECRET WORLD · HEGEMONY</small></span></a><nav>
      {game.session && <span className={`hg-connection hg-connection-${game.connection}`} role="status" title="前台约每1.5秒轮询同步；操作提交后立即更新。"><i />{game.connection === 'online' ? '牌桌已同步 · 轮询' : game.connection === 'connecting' ? '正在同步牌桌' : '连接中断 · 自动重连'}</span>}
      <button className="hg-nav-button" onClick={() => setHelp(true)}>上手指南 <span>?</span></button>
    </nav></header>
    {game.error && <div className="hg-error" role="alert"><span>{game.error}</span><button onClick={game.dismissError} aria-label="关闭提示">×</button></div>}
    {game.uncertain && <div className="hg-unconfirmed" role="status"><span>上一行动等待确认；牌桌将在恢复连接后继续。</span><button className="hg-button hg-button-primary" disabled={game.busy} onClick={game.retryPending}>确认上一行动</button></div>}
    {game.resumeAvailable && <div className="hg-return-seat"><span>此浏览器保存着你的牌桌座位。</span><button className="hg-button hg-button-quiet" onClick={game.resume}>回到原牌桌 →</button></div>}
    {!game.session ? <Lobby catalog={game.catalog} busy={game.busy} onCreate={game.create} onJoin={game.join} retry={game.retryCatalog} /> : !game.view ? <main className="hg-resume"><span className="hg-brand-mark">◈</span><h1>正在恢复你的座位</h1><p>同步手牌、牌桌与尚未完成的选择…</p><p className="hg-muted">连接恢复后会自动继续。</p><button className="hg-button hg-button-quiet" onClick={game.leave}>返回大厅，保留此座位</button></main> : game.view.status === 'lobby' ? <RoomLobby view={game.view} catalog={game.catalog} busy={game.busy || game.uncertain} onAction={game.act} /> : <Table view={game.view} catalog={game.catalog} busy={game.busy || game.uncertain} uncertain={game.uncertain} connection={game.connection} onAction={game.act} />}
    <footer className="hg-footer"><span>受限真实卡池 · 50 张自组预组 · 邀请制云端牌桌</span><div>{game.session && <button onClick={game.leave}>返回大厅 · 座位将保留</button>}</div></footer>
    {help && <Help onClose={() => setHelp(false)} />}
  </div>;
}
