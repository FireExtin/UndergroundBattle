import { useEffect, useMemo, useState } from 'react';
import { CardContent, CardTile } from './CardTile';
import { ChoicePanel } from './ChoicePanel';
import { DeckPicker } from './Lobby';
import { AutoPass } from './AutoPass';
import type { Action, Card, CardDefinition, Catalog, LegalAction, Player, Region, View } from './types';

const phaseNames: Record<string, string> = { start: '开始阶段', beginning: '开始阶段', action: '行动阶段', confrontation: '对抗阶段', conflict: '对抗阶段', end: '结束阶段', finished: '对局结束', lobby: '准备入席', investigation: '调查', combat: '战斗', influence: '势力', claim: '赢区窗口', win: '赢区窗口', fast: '快速行动窗口', draw: '抓牌', prepare: '重置与准备', mobility: '机动窗口', mulligan: '再调度', ready: '准备' };
export function phaseLabel(phase: string) {
  if (phaseNames[phase]) return phaseNames[phase];
  const region = /^region:(\d+):([^:]+)(?::(before|after))?$/.exec(phase);
  if (region) return `地区 ${Number(region[1]) + 1} · ${phaseNames[region[2]] || '对抗'}${region[3] ? ` · ${region[3] === 'before' ? '奖励前' : '奖励后'}快速窗口` : ''}`;
  const team = /^team:(\d+)$/.exec(phase);
  if (team) return `团队 ${Number(team[1]) + 1} 行动`;
  return /[\u4e00-\u9fff]/.test(phase) ? phase : '进行中';
}
function teamName(team: number, view: View) {
  const names = view.players.filter(player => player.team === team).map(player => player.name);
  return names.length ? names.join(' / ') : `团队 ${team + 1}`;
}
function ActionButtons({ actions, busy, onAction }: { actions: LegalAction[]; busy: boolean; onAction: (action: Action) => void }) {
  return <div className="hg-actions">{actions.map(action => <button type="button" key={action.id} data-action-id={action.id} data-action-kind={action.kind} className={`hg-button ${['start', 'restart', 'ready'].includes(action.kind) ? 'hg-button-primary' : action.kind === 'pass' ? 'hg-button-quiet' : 'hg-button-action'}`} disabled={busy} onClick={() => onAction(action)} title={action.description}>
    <span>{action.label}</span>{action.description && <small>{action.description}</small>}
  </button>)}</div>;
}

export function RoomLobby({ view, catalog, busy, onAction }: { view: View; catalog: Catalog | null; busy: boolean; onAction: (action: Action) => void }) {
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);
  const you = view.players.find(player => player.id === view.you);
  const capacity = view.mode === 'teams' ? 4 : 2;
  const yourDeck = catalog?.decks.find(deck => deck.id === you?.deckId);
  const starts = view.legalActions.filter(action => action.kind === 'start' || action.kind === 'ready');
  return <main className="hg-room-lobby">
    <div className="hg-room-welcome"><span className="hg-eyebrow">牌桌已经就绪</span><h1>等待秘社集结</h1><p>{view.mode === 'duel' ? '两人对决 · 3 个地区 · 8 分胜利' : '四人协作 · 5 个地区 · 团队 10 分胜利'}</p></div>
    <section className="hg-invite"><div><span className="hg-eyebrow">邀请伙伴入席</span><strong>{view.inviteCode}</strong><p>分享邀请码或邀请链接。每个浏览器独立占据一个座位。</p></div><button className="hg-button hg-button-primary" onClick={async () => {
      try { await navigator.clipboard.writeText(`${location.origin}/?invite=${encodeURIComponent(view.inviteCode)}`); setCopied(true); setCopyError(false); }
      catch { setCopyError(true); }
    }}>{copied ? '✓ 邀请链接已复制' : '复制邀请链接 ↗'}</button>{copyError && <p role="status">浏览器无法复制，请手动分享上方邀请码。</p>}</section>
    <div className="hg-seats">{Array.from({ length: capacity }, (_, seat) => {
      const player = view.players.find(item => item.seat === seat);
      const team = player?.team ?? (view.mode === 'teams' ? (seat < 2 ? 0 : 1) : seat);
      return <article key={seat} className={`hg-seat hg-team-${team} ${player ? '' : 'hg-seat-empty'}`}><span className="hg-eyebrow">席位 {seat + 1} · {team === you?.team ? '我方' : '对方'}</span>
        <span className="hg-player-avatar">{player ? player.name.slice(0, 1) : '+'}</span><strong>{player?.name || '等待伙伴'}{player?.id === view.you && <small> 你</small>}{seat === 0 && player && <small> 房主</small>}</strong>
        <span>{player ? catalog?.decks.find(deck => deck.id === player.deckId)?.name || '秘社牌组' : '用邀请码加入'}</span><span className={`hg-ready ${player?.ready ? 'hg-is-ready' : ''}`}>{player ? player.ready ? '✓ 已准备' : '尚未准备' : '空座'}</span>
      </article>;
    })}</div>
    <section className="hg-room-prepare"><div><span className="hg-eyebrow">你的牌组</span><h2>{yourDeck?.name || '选择秘社牌组'}</h2><p className="hg-muted">准备前可更换受限卡池预组。所有玩家准备后，由房主开始。</p></div>
      {catalog && <DeckPicker decks={catalog.decks} value={you?.deckId || ''} allowedIds={view.legalActions.filter(action => action.kind === 'deck').map(action => action.option || '')} disabled={busy || !view.legalActions.some(action => action.kind === 'deck')} onChange={id => {
        const action = view.legalActions.find(item => item.kind === 'deck' && item.option === id); if (action) onAction(action);
      }} />}
      <div className="hg-room-ready"><span>{view.players.filter(player => player.ready).length} / {capacity} 位玩家已准备</span><ActionButtons actions={starts} busy={busy} onAction={onAction} />{!starts.some(action => action.kind === 'start') && <p>{you?.seat === 0 ? '等所有座位入席并准备后，即可开始。' : '准备后等待房主开始对局。'}</p>}</div>
    </section>
  </main>;
}

function PlayerStrip({ player, view, catalog }: { player: Player; view: View; catalog: Catalog | null }) {
  const assets = view.assets.filter(card => card.controller === player.id);
  const loyalty = assets.reduce((result, card) => {
    const definition = catalog?.cards.find(item => item.id === card.cardId);
    const color = card.color || definition?.color;
    if (color) result.set(color, (result.get(color) || 0) + 1);
    return result;
  }, new Map<string, number>());
  return <div className={`hg-player-strip hg-team-${player.team}${player.id === view.you ? ' hg-player-you' : ''}`}>
    <span className="hg-player-avatar">{player.name.slice(0, 1)}</span><div><strong>{player.name}{player.id === view.you && <small> · 你</small>}</strong><span>{catalog?.decks.find(deck => deck.id === player.deckId)?.name || '秘社'}{player.eliminated ? ' · 已出局' : ''}</span></div>
    <dl><div><dt>手牌</dt><dd>{player.handCount}</dd></div><div><dt>牌库</dt><dd>{player.deckCount}</dd></div><div><dt>资产</dt><dd>{assets.length}</dd></div></dl>
    <span className="hg-loyalty-summary" title="资产提供的派系忠诚">{[...loyalty].map(([color, count]) => `${color} ${count}`).join(' · ') || '暂无派系资产'}</span>
  </div>;
}

function RegionTile({ region, view, definitions, selected, available, onRegion, onCard, selectedCard }: {
  region: Region; view: View; definitions: Map<string, CardDefinition>; selected: boolean; available: boolean;
  onRegion: () => void; onCard: (id: string) => void; selectedCard: string | null;
}) {
  const renderSide = (team: number) => {
    const characters = region.characters.filter(card => view.players.find(player => player.id === card.controller)?.team === team);
    return <div className={`hg-region-side hg-team-${team}`}><div className="hg-region-side-label"><span>{teamName(team, view)}</span><span>影响 <b>{region.influence[team] || 0}</b></span></div>
      <div className="hg-region-characters">{characters.map(card => <CardTile key={card.instanceId} card={card} definition={definitions.get(card.cardId || '')} compact selected={selectedCard === card.instanceId} onSelect={() => onCard(card.instanceId)} />)}{!characters.length && <span className="hg-region-vacant">尚无角色</span>}</div>
    </div>;
  };
  return <article className={`hg-region ${selected ? 'hg-selected-region' : ''} ${available ? 'hg-region-available' : ''}`}>
    {renderSide(1 - (view.players.find(player => player.id === view.you)?.team ?? 0))}
    <button type="button" className="hg-region-center" onClick={onRegion} aria-pressed={selected} aria-label={`查看地区${region.index + 1} ${region.name}`}>
      <span className="hg-region-number">0{region.index + 1}</span><span className="hg-region-title"><small>隐秘地区</small><strong>{region.name}</strong></span><span className="hg-region-value"><b>{region.points}</b><small>分</small></span>
      <span className="hg-region-threshold">控制阈值 <b>{region.threshold}</b>{available && <em>可执行行动</em>}</span>
    </button>
    {renderSide(view.players.find(player => player.id === view.you)?.team ?? 0)}
  </article>;
}

export function Table({ view, catalog, busy, uncertain = false, connection = 'connecting', onAction }: { view: View; catalog: Catalog | null; busy: boolean; uncertain?: boolean; connection?: 'connecting' | 'online' | 'offline'; onAction: (action: Action) => void }) {
  const [selectedCard, setSelectedCard] = useState<string | null>(null);
  const [selectedRegion, setSelectedRegion] = useState<number | null>(null);
  const definitions = useMemo(() => new Map(catalog?.cards.map(card => [card.id, card]) || []), [catalog]);
  const allCards = [...view.hand, ...view.assets, ...view.graveyard, ...view.scoreCards, ...view.regions.flatMap(region => region.characters)];
  const card = allCards.find(item => item.instanceId === selectedCard);
  const region = view.regions.find(item => item.index === selectedRegion);
  const you = view.players.find(player => player.id === view.you);
  useEffect(() => { if (selectedCard && !card) setSelectedCard(null); }, [selectedCard, card]);
  const selectCard = (id: string) => { setSelectedCard(id); setSelectedRegion(null); };
  const globalActions = view.legalActions.filter(action => !['choose', 'deck', 'ready', 'start', 'restart'].includes(action.kind) && !action.cardId && !action.targetId && action.region === undefined);
  const cardActions = view.legalActions.filter(action => action.kind !== 'choose' && selectedCard && (action.cardId === selectedCard || action.targetId === selectedCard));
  const lastHandCharacterToAsset = card?.kind === 'character'
    && view.hand.some(item => item.instanceId === selectedCard)
    && view.hand.filter(item => item.kind === 'character').length === 1
    && cardActions.some(action => action.kind === 'asset' && action.cardId === selectedCard);
  const regionActions = view.legalActions.filter(action => action.kind !== 'choose' && action.region === selectedRegion);
  const choiceAction = view.legalActions.find(action => action.kind === 'choose' && (!action.choiceId || action.choiceId === view.pendingChoice?.id));
  const choicePlayer = view.pendingChoice?.playerId || view.waitingChoice?.playerId;
  const choiceName = view.players.find(player => player.id === choicePlayer)?.name;
  const status = view.pendingChoice ? '请完成下方选择' : view.waitingChoice ? `等待 ${choiceName || '另一位玩家'}：${view.waitingChoice.title}`
    : view.legalActions.length ? '轮到你参与行动' : `等待 ${teamName(view.priorityTeam, view)} 行动`;
  return <main className="hg-table">
    {view.status === 'finished' && <section className="hg-victory" role="status"><span aria-hidden="true">✧</span><div><small>对局结束</small><h1>{view.winnerTeam === you?.team ? '你的秘社取得了霸权' : `${view.winnerTeam === undefined ? '本局' : teamName(view.winnerTeam, view)}赢得了霸权`}</h1><p>胜利目标 {view.winScore} 分 · 可以查看牌桌与行动记录，或由房主发起新一局。</p></div><ActionButtons actions={view.legalActions.filter(action => action.kind === 'restart')} busy={busy} onAction={onAction} /></section>}
    <section className="hg-scoreboard">{[0, 1].map(team => {
      const score = view.players.filter(player => player.team === team).reduce((total, player) => total + player.score, 0);
      return <div key={team} className={`hg-team-score hg-team-${team}`}><div><span className="hg-eyebrow">{team === you?.team ? '我方秘社' : '对方秘社'}</span><strong>{teamName(team, view)}</strong></div><span className="hg-score"><b>{score}</b><small>/ {view.winScore}</small></span><div className="hg-score-progress"><i style={{ width: `${Math.min(100, score / view.winScore * 100)}%` }} /></div></div>;
    })}<div className="hg-round"><span className="hg-eyebrow">第 {view.turn} 回合</span><strong>{phaseLabel(view.phase)}</strong><span>{phaseLabel(view.step)}</span></div></section>
    <div className="hg-phase-line"><span>先手 <b>{teamName(view.firstTeam, view)}</b></span><span>行动团队 <b>{teamName(view.activeTeam, view)}</b></span><span>优先权 <b>{teamName(view.priorityTeam, view)}</b></span><strong className={view.pendingChoice ? 'hg-your-choice' : ''}>{status}</strong></div>
    <AutoPass view={view} busy={busy} uncertain={uncertain} connection={connection} onAction={onAction} />
    <div className="hg-players">{view.players.map(player => <PlayerStrip key={player.id} player={player} view={view} catalog={catalog} />)}</div>
    {view.pendingChoice && <ChoicePanel key={view.pendingChoice.id} choice={view.pendingChoice} action={choiceAction} definitions={definitions} busy={busy} onSubmit={onAction} />}
    <div className="hg-play-layout"><div className="hg-play-main">
      <section className="hg-board-section"><div className="hg-section-title"><h2>世界版图 <small>{view.regions.length} 个地区</small></h2><span className="hg-muted">上方对方 · 下方我方</span></div>
        <div className={`hg-board hg-board-${view.mode}`}>{view.regions.map(item => <RegionTile key={item.id} region={item} view={view} definitions={definitions} selected={selectedRegion === item.index} selectedCard={selectedCard} available={view.legalActions.some(action => action.region === item.index && (!selectedCard || action.cardId === selectedCard))} onRegion={() => { setSelectedRegion(item.index); setSelectedCard(null); }} onCard={selectCard} />)}</div>
        <p className="hg-swipe-hint">↔ 横向滑动查看全部地区</p>
      </section>
      <section className="hg-hand-section"><div className="hg-section-title"><h2>你的手牌 <small>{view.hand.length} 张</small></h2><span className="hg-muted">横向查看 · 点选卡牌行动</span></div>
        <div className="hg-hand">{view.hand.map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}{!view.hand.length && <p className="hg-empty">目前没有手牌。</p>}</div>
      </section>
      <section className="hg-assets-section"><div className="hg-section-title"><h2>秘社资产</h2><span className="hg-muted">派系忠诚与费用来源</span></div><div className="hg-asset-groups">{view.players.map(player => <div key={player.id} className={`hg-asset-group hg-team-${player.team}`}><h3>{player.name}{player.id === view.you ? ' · 你' : ''}<small>{view.assets.filter(item => item.controller === player.id).length} 张</small></h3><div className="hg-assets">{view.assets.filter(item => item.controller === player.id).map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} compact selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}{!view.assets.some(item => item.controller === player.id) && <p className="hg-empty">暂无资产</p>}</div></div>)}</div></section>
      <details className="hg-public-zones"><summary>墓地与计分区 <span>墓地 {view.graveyard.length} · 计分牌 {view.scoreCards.length}</span></summary><div>{view.players.map(player => <section key={player.id}><h3>{player.name}</h3><h4>墓地</h4><div className="hg-zone-cards">{view.graveyard.filter(item => item.owner === player.id).map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} compact selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}</div><h4>计分区</h4><div className="hg-zone-cards">{view.scoreCards.filter(item => item.owner === player.id).map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} compact selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}</div></section>)}</div></details>
    </div><aside className="hg-table-aside">
      <section className={`hg-inspector ${card || region ? 'hg-inspector-active' : ''}`}><div className="hg-section-title"><span className="hg-eyebrow">当前可用行动</span>{(card || region) && <button className="hg-inspector-close" onClick={() => { setSelectedCard(null); setSelectedRegion(null); }} aria-label="关闭卡牌详情">×</button>}</div>{card ? <><div className="hg-inspected-card"><CardContent card={card} definition={definitions.get(card.cardId || '')} /></div><p className="hg-muted">{view.players.find(player => player.id === card.controller)?.name} 操控{card.region !== undefined ? ` · 地区 ${card.region + 1}` : ''}</p>{lastHandCharacterToAsset && <p className="hg-strategy-note">转为资产后，手中将暂时没有角色。资产提供费用与忠诚，但不能参与对抗；每回合开始仍会抓一张牌。</p>}<ActionButtons actions={cardActions} busy={busy} onAction={onAction} />{!cardActions.length && <p className="hg-wait-note">当前时点此牌没有可执行行动。</p>}</> : region ? <><h2>{region.name}</h2><p>控制阈值 {region.threshold} · 赢得后 {region.points} 分</p><p className="hg-rule-text">{definitions.get(region.cardId)?.text}</p><ActionButtons actions={regionActions} busy={busy} onAction={onAction} />{!regionActions.length && <p className="hg-wait-note">选择手牌可查看向此地区派遣的行动。</p>}</> : <><h2>{view.status === 'finished' ? '本局已结束' : view.pendingChoice ? '等待你的决定' : '选择一张牌'}</h2><p className="hg-wait-note">{view.status === 'finished' ? '展开记录，回顾这场秘社交锋。' : view.pendingChoice ? '完成牌桌上方的选择后，游戏继续。' : '点选手牌、资产或地区角色。此处会显示该牌当前可执行的行动。'}</p></>}
        {!!globalActions.length && <div className="hg-general-actions"><span className="hg-eyebrow">牌桌行动</span><ActionButtons actions={globalActions} busy={busy} onAction={onAction} /></div>}
      </section>
      <section className="hg-stack"><div className="hg-section-title"><h2>待结算效果</h2><span>{view.stack.length}</span></div>{view.stack.length ? <ol>{[...view.stack].reverse().map(effect => <li key={effect.id}><strong>{effect.label}</strong><small>{view.players.find(player => player.id === effect.controller)?.name}</small></li>)}</ol> : <p className="hg-empty">目前没有待结算效果。</p>}</section>
      <section className="hg-log"><div className="hg-section-title"><h2>行动记录</h2><span>最新在上</span></div><ol aria-label="牌桌行动记录">{[...view.log].reverse().slice(0, 50).map((entry, index) => <li key={`${entry.version}-${index}`}><span className="hg-log-dot" /><p>{entry.text}</p></li>)}</ol>{!view.log.length && <p className="hg-empty">等待第一步行动。</p>}</section>
    </aside></div>
  </main>;
}
