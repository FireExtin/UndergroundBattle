import { useState } from 'react';
import { Help } from './Help';
import { Lobby } from './Lobby';
import { RoomLobby, Table } from './Table';
import { useGame } from './useGame';
import { RoomManagement } from './RoomManagement';
import './game.css';

export function GameApp() {
  const game = useGame();
  const [help, setHelp] = useState(false);
  return <div className="hg-app">
    <header className="hg-header"><a href="/" className="hg-brand" onClick={event => { event.preventDefault(); if (!game.busy) game.leave(); }}><span className="hg-brand-mark">◈</span><span>隐秘世界 <b>霸权</b><small>SECRET WORLD · HEGEMONY</small></span></a><nav>
      {game.session && <span className={`hg-connection hg-connection-${game.connection}`} role="status" title="进行中的牌桌轮询同步；暂停的牌桌可手动查看最新状态。"><i />{game.connection === 'online' ? game.view?.pause ? '牌桌已暂停 · 已保存' : '牌桌已同步 · 轮询' : game.connection === 'connecting' ? '正在同步牌桌' : '连接中断 · 自动重连'}</span>}
      {game.session && <button className="hg-nav-button" disabled={game.busy} onClick={game.leave} title="保留当前座位，回到大厅选择两人或四人新牌桌">返回大厅 / 新建牌桌</button>}
      <button className="hg-nav-button" onClick={() => setHelp(true)}>上手指南 <span>?</span></button>
    </nav></header>
    <section className="hg-player-session" aria-label="玩家会话">
      <strong>{game.playerMode === 'independent' ? '独立玩家会话 · 当前标签' : '普通玩家会话'}</strong>
      <p>{game.playerMode === 'independent' ? '刷新保留此标签的席位。关闭标签可能失去此独立座位的恢复方式。复制标签可能沿用原席；需要另一位玩家时，请返回大厅后显式开始新的独立玩家会话。' : '在此浏览器恢复原席。需要在另一个标签作为另一位玩家入席时，请显式开始新的独立玩家会话。'}</p>
      {game.session && game.playerMode === 'ordinary' && <p>只切换当前标签；普通玩家的座位和牌桌恢复方式保留。</p>}
      {!game.session && game.playerMode === 'independent' && <p>开始新会话或切回普通后，此独立席位可能无法恢复；需要多席请另开标签。</p>}
      {(!game.session || game.playerMode === 'ordinary') && <div><button className="hg-button hg-button-quiet" disabled={game.playerModeLocked} onClick={game.startIndependentSession}>开始新的独立玩家会话</button>
        {game.playerMode === 'independent' && <button className="hg-button hg-button-quiet" disabled={game.playerModeLocked} onClick={game.useOrdinarySession}>切回普通玩家会话</button>}</div>}
    </section>
    {game.error && <div className="hg-error" role="alert"><span>{game.error}</span>{game.session && !game.uncertain && <button className="hg-button hg-button-quiet" disabled={game.busy} onClick={game.refresh}>查看最新牌桌</button>}<button onClick={game.dismissError} aria-label="关闭提示">×</button></div>}
    {game.uncertain && <div className="hg-unconfirmed" role="status"><span>{game.legacyRecoveryBlocked ? '旧行动缺少原座位记录，不能安全自动恢复；原记录已保留。' : '上一行动等待确认；牌桌将在恢复连接后继续。'}</span><button className="hg-button hg-button-primary" disabled={game.busy || game.legacyRecoveryBlocked && !game.session} onClick={game.legacyRecoveryBlocked ? game.refresh : game.retryPending}>{game.legacyRecoveryBlocked ? '查看最新牌桌' : '确认上一行动'}</button></div>}
    {game.resumeAvailable && <section className="hg-return-seat" aria-label="我的牌桌"><strong>我的牌桌 · 此浏览器</strong><p>已保留这些牌桌的原席位。未进入的牌桌不自动轮询；返回牌桌后可查看或恢复已暂停的对局。</p><div>{game.savedSeats.map(seat => <button key={`${seat.roomId}-${seat.seat}`} className="hg-button hg-button-quiet" onClick={() => game.resume(seat)}>回到牌桌 {seat.inviteCode} · 席位 {seat.seat + 1}</button>)}</div></section>}
    {game.view && <RoomManagement key={game.view.roomId} view={game.view} blocked={game.busy || game.uncertain || game.connection !== 'online'} onAction={game.act} refresh={game.refresh} />}
    {!game.session ? <Lobby independent={game.playerMode === 'independent'} catalog={game.catalog} busy={game.busy} onCreate={game.create} onJoin={game.join} onCreateDraft={game.createDraft} onJoinDraft={game.joinDraft} retry={game.retryCatalog} /> : !game.view ? <main className="hg-resume"><span className="hg-brand-mark">◈</span><h1>正在恢复你的座位</h1><p>同步手牌、牌桌与尚未完成的选择…</p><p className="hg-muted">连接恢复后会自动继续。</p><button className="hg-button hg-button-quiet" onClick={game.leave}>返回大厅，保留此座位</button></main> : game.view.status === 'lobby' ? <RoomLobby view={game.view} catalog={game.catalog} busy={game.busy || game.uncertain} onAction={game.act} /> : <Table modalPaused={help} view={game.view} catalog={game.catalog} busy={game.busy || game.uncertain || !!game.view.pause} uncertain={game.uncertain} connection={game.connection} onAction={game.act} autoPassEnabled={game.autoPassEnabled} onAutoPassEnabledChange={game.setAutoPassEnabled} />}
    <footer className="hg-footer"><span>受限已实现卡池 · 自组与预组 · 邀请制云端牌桌</span></footer>
    {help && <Help catalog={game.catalog} onClose={() => setHelp(false)} />}
  </div>;
}
