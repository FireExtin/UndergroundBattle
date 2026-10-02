import { useEffect, useMemo, useState } from 'react';
import { CardContent, CardTile, visibleCard } from './CardTile';
import { ChoicePanel } from './ChoicePanel';
import { DeckPicker } from './Lobby';
import { AutoPass } from './AutoPass';
import { ReadModal } from './ReadModal';
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
    const magic = card.magic || definition?.magic;
    if (magic) result.set(magic, (result.get(magic) || 0) + 1);
    return result;
  }, new Map<string, number>());
  return <div className={`hg-player-strip hg-team-${player.team}${player.id === view.you ? ' hg-player-you' : ''}`}>
    <span className="hg-player-avatar">{player.name.slice(0, 1)}</span><div><strong>{player.name}{player.id === view.you && <small> · 你</small>}</strong><span>席位 {player.seat + 1} · {catalog?.decks.find(deck => deck.id === player.deckId)?.name || '秘社'}{player.eliminated ? ' · 已出局' : ''}</span></div>
    <dl><div><dt>手牌</dt><dd>{player.handCount}</dd></div><div><dt>牌库</dt><dd>{player.deckCount}</dd></div><div><dt>资产</dt><dd>{assets.length}</dd></div><div><dt>可用费用</dt><dd>{assets.filter(card => !card.exhausted).length}</dd></div></dl>
    <span className="hg-loyalty-summary" title="资产提供的派系忠诚">{[...loyalty].map(([color, count]) => `${color} ${count}`).join(' · ') || '暂无派系资产'}</span>
  </div>;
}

function RegionTile({ region, view, definitions, selected, available, onRegion, onCard, selectedCard }: {
  region: Region; view: View; definitions: Map<string, CardDefinition>; selected: boolean; available: boolean;
  onRegion: () => void; onCard: (id: string) => void; selectedCard: string | null;
}) {
  const renderSide = (team: number) => {
    const characters = region.characters.filter(card => view.players.find(player => player.id === card.controller)?.team === team);
    const mine = view.players.find(player => player.id === view.you)?.team === team;
    return <div className={`hg-region-side hg-team-${team}`} data-team={team} data-side={mine ? 'mine' : 'opponent'}><div className="hg-region-side-label"><span>{mine ? '我方' : '对方'} · {characters.length} 张角色</span><span>影响 <b>{region.influence[team] || 0}</b></span></div>
      <div className="hg-region-characters" aria-label={`${region.name}${mine ? '我方' : '对方'}角色，可在区域内滚动`}>{characters.map(card => <CardTile key={card.instanceId} card={card} definition={definitions.get(card.cardId || '')} viewerId={view.you} owner={view.players.find(player => player.id === card.owner)} actionable={view.legalActions.some(action => action.cardId === card.instanceId || action.targetId === card.instanceId)} compact selected={selectedCard === card.instanceId} onSelect={() => onCard(card.instanceId)} />)}{!characters.length && <span className="hg-region-vacant">尚无角色</span>}</div>
    </div>;
  };
  return <article className={`hg-region ${selected ? 'hg-selected-region' : ''} ${available ? 'hg-region-available' : ''}`}>
    {renderSide(1 - (view.players.find(player => player.id === view.you)?.team ?? 0))}
    <button type="button" className="hg-region-center" data-region-index={region.index} onClick={onRegion} aria-pressed={selected} aria-label={`查看地区${region.index + 1} ${region.name}`}>
      <span className="hg-region-number">0{region.index + 1}</span><span className="hg-region-title"><small>隐秘地区</small><strong>{region.name}</strong></span><span className="hg-region-value"><b>{region.points}</b><small>分</small></span>
      <span className="hg-region-threshold">控制阈值 <b>{region.threshold}</b>{available && <em>可执行行动</em>}</span>
    </button>
    {renderSide(view.players.find(player => player.id === view.you)?.team ?? 0)}
  </article>;
}

export function Table({ view, catalog, busy, uncertain = false, connection = 'connecting', onAction }: { view: View; catalog: Catalog | null; busy: boolean; uncertain?: boolean; connection?: 'connecting' | 'online' | 'offline'; onAction: (action: Action) => void }) {
  const [selectedCard, setSelectedCard] = useState<string | null>(null);
  const [selectedRegion, setSelectedRegion] = useState<number | null>(null);
  const [reading, setReading] = useState<Card | null>(null);
  const definitions = useMemo(() => new Map(catalog?.cards.map(card => [card.id, card]) || []), [catalog]);
  const allCards = [...view.hand, ...view.assets, ...view.graveyard, ...view.scoreCards, ...view.regions.flatMap(region => region.characters)].map(item => visibleCard(item, view.you));
  const card = allCards.find(item => item.instanceId === selectedCard);
  const region = view.regions.find(item => item.index === selectedRegion);
  const you = view.players.find(player => player.id === view.you);
  const yourTeam = you?.team ?? 0;
  const teamOrder = [yourTeam, 1 - yourTeam];
  const seatOrder = [...view.players].sort((a, b) => teamOrder.indexOf(a.team) - teamOrder.indexOf(b.team) || a.seat - b.seat);
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
  const status = view.pendingChoice ? '请你完成下方选择' : view.waitingChoice ? `等待 ${choiceName || '另一位玩家'}：${view.waitingChoice.title}`
    : view.legalActions.length ? '轮到你参与行动' : `等待 ${teamName(view.priorityTeam, view)} 行动`;
  const browsingOnly = view.status !== 'playing' || !view.legalActions.some(action => action.kind !== 'choose') || !!view.waitingChoice;
  const nextStep = view.status === 'finished' ? '本局已结束，可回顾牌桌'
    : view.pendingChoice ? `下一步：${view.pendingChoice.title}`
    : view.waitingChoice ? `等待 ${choiceName || '另一位玩家'} 完成选择，可浏览牌桌`
    : !view.legalActions.length ? `等待 ${teamName(view.priorityTeam, view)} 行动，可浏览牌桌`
    : view.legalActions.length === 1 && globalActions[0]?.kind === 'pass' ? '下一步：让过，推进当前窗口'
    : '下一步：点选可行动的手牌或地区角色';
  return <main className="hg-table">
    {view.status === 'finished' && <section className="hg-victory" role="status"><span aria-hidden="true">✧</span><div><small>对局结束</small><h1>{view.winnerTeam === you?.team ? '你的秘社取得了霸权' : `${view.winnerTeam === undefined ? '本局' : teamName(view.winnerTeam, view)}赢得了霸权`}</h1><p>胜利目标 {view.winScore} 分 · 可以查看牌桌与行动记录，或由房主发起新一局。</p></div><ActionButtons actions={view.legalActions.filter(action => action.kind === 'restart')} busy={busy} onAction={onAction} /></section>}
    <section className="hg-scoreboard">{teamOrder.map(team => {
      const score = view.players.filter(player => player.team === team).reduce((total, player) => total + player.score, 0);
      return <div key={team} className={`hg-team-score hg-team-${team}`}><div><span className="hg-eyebrow">{team === you?.team ? '我方秘社' : '对方秘社'}</span><strong>{teamName(team, view)}</strong></div><span className="hg-score"><b>{score}</b><small>/ {view.winScore}</small></span><div className="hg-score-progress"><i style={{ width: `${Math.min(100, score / view.winScore * 100)}%` }} /></div></div>;
    })}<div className="hg-round"><span className="hg-eyebrow">第 {view.turn} 回合</span><strong>{phaseLabel(view.phase)}</strong><span>{phaseLabel(view.step)}</span></div></section>
    <div className="hg-phase-line"><span>先手 <b>{teamName(view.firstTeam, view)}</b></span><span>行动团队 <b>{teamName(view.activeTeam, view)}</b></span><span>优先权 <b>{teamName(view.priorityTeam, view)}</b></span><strong className={view.pendingChoice ? 'hg-your-choice' : ''}>{status}</strong></div>
    <AutoPass view={view} busy={busy} uncertain={uncertain} connection={connection} onAction={onAction} />
    <div className="hg-players">{seatOrder.map(player => <PlayerStrip key={player.id} player={player} view={view} catalog={catalog} />)}</div>
    {view.pendingChoice && <ChoicePanel key={view.pendingChoice.id} choice={view.pendingChoice} action={choiceAction} definitions={definitions} busy={busy} onSubmit={onAction} onReadCard={setReading} viewerId={view.you} playerLabels={Object.fromEntries(view.players.map(player => [player.id, player.name]))} />}
    <div className="hg-play-layout"><div className="hg-play-main">
      <section className="hg-board-section"><div className="hg-section-title"><h2>世界版图 <small>{view.regions.length} 个地区</small></h2><span className="hg-muted">上方对方 · 下方我方</span></div>
        <div className={`hg-board hg-board-${view.mode}`}>{view.regions.map(item => <RegionTile key={item.id} region={item} view={view} definitions={definitions} selected={selectedRegion === item.index} selectedCard={selectedCard} available={view.legalActions.some(action => action.region === item.index && (!selectedCard || action.cardId === selectedCard))} onRegion={() => { setSelectedRegion(item.index); setSelectedCard(null); }} onCard={selectCard} />)}</div>
        <p className="hg-swipe-hint">↔ 横向滑动查看全部地区</p>
      </section>
      <details className="hg-assets-section"><summary>秘社资产 <span>费用与忠诚始终显示在上方玩家栏 · 展开看牌</span></summary><div className="hg-asset-groups">{seatOrder.map(player => <div key={player.id} className={`hg-asset-group hg-team-${player.team}`}><h3>{player.name}{player.id === view.you ? ' · 你' : ''}<small>{view.assets.filter(item => item.controller === player.id).length} 张</small></h3><div className="hg-assets">{view.assets.filter(item => item.controller === player.id).map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} viewerId={view.you} compact selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}{!view.assets.some(item => item.controller === player.id) && <p className="hg-empty">暂无资产</p>}</div></div>)}</div></details>
      <details className="hg-public-zones"><summary>墓地与计分区 <span>墓地 {view.graveyard.length} · 计分牌 {view.scoreCards.length}</span></summary><div>{view.players.map(player => <section key={player.id}><h3>{player.name}</h3><h4>墓地</h4><div className="hg-zone-cards">{view.graveyard.filter(item => item.owner === player.id).map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} compact selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}</div><h4>计分区</h4><div className="hg-zone-cards">{view.scoreCards.filter(item => item.owner === player.id).map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} compact selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}</div></section>)}</div></details>
    </div><aside className="hg-table-aside">
      <section className={`hg-inspector ${card || region ? 'hg-inspector-active' : ''}`}>
        <div className="hg-section-title"><span className="hg-eyebrow">{browsingOnly ? '浏览牌桌 · 暂不可行动' : '当前可用行动'}</span>{(card || region) && <button className="hg-inspector-close" onClick={() => { setSelectedCard(null); setSelectedRegion(null); }} aria-label="关闭卡牌详情">×</button>}</div>
        {card ? <>
          <div className="hg-inspected-card" role="button" tabIndex={0} aria-label={`放大阅读${card.name}`} onClick={() => setReading(card)} onKeyDown={event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); setReading(card); } }}><CardContent card={card} definition={definitions.get(card.cardId || '')} /></div>
          <button type="button" className="hg-small-read" onClick={() => setReading(card)}>放大文字与图标 ↗</button>
          <p className="hg-muted">{view.players.find(player => player.id === card.owner)?.name} 拥有 · {view.players.find(player => player.id === card.controller)?.name} 操控{card.region !== undefined ? ` · 地区 ${card.region + 1}` : ''}</p>
          {lastHandCharacterToAsset && <p className="hg-strategy-note">转为资产后，手中将暂时没有角色。资产提供费用与忠诚，但不能参与对抗；每回合开始仍会抓一张牌。</p>}
          <ActionButtons actions={cardActions} busy={busy} onAction={onAction} />
          {!cardActions.length && <p className="hg-wait-note">{browsingOnly ? '可以阅读公开牌与自己的暗牌。当前等待其他玩家，此牌暂不可行动。' : '当前时点此牌没有可执行行动，可浏览其他卡牌。'}</p>}
        </> : region ? <>
          <h2>{region.name}</h2><p>控制阈值 {region.threshold} · 赢得后 {region.points} 分</p><p className="hg-rule-text">{definitions.get(region.cardId)?.text}</p>
          <button type="button" className="hg-small-read" onClick={() => setReading({ instanceId: region.id, cardId: region.cardId, name: region.name, kind: 'region', owner: view.you, controller: view.you, exhausted: false, faceDown: false })}>放大地区文字 ↗</button>
          <ActionButtons actions={regionActions} busy={busy} onAction={onAction} />{!regionActions.length && <p className="hg-wait-note">选择手牌可查看向此地区派遣的行动。</p>}
        </> : <><h2>{view.status === 'finished' ? '本局已结束' : view.pendingChoice ? '等待你的决定' : browsingOnly ? '正在等待其他玩家' : '选择一张可行动的牌'}</h2><p className="hg-wait-note">{nextStep}。手牌与下一步按钮始终在屏幕下方。</p></>}
      </section>
      <section className="hg-stack"><div className="hg-section-title"><h2>待结算效果</h2><span>{view.stack.length}</span></div>{view.stack.length ? <ol>{[...view.stack].reverse().map(effect => <li key={effect.id}><strong>{effect.label}</strong><small>{view.players.find(player => player.id === effect.controller)?.name}</small></li>)}</ol> : <p className="hg-empty">目前没有待结算效果。</p>}</section>
      <details className="hg-log"><summary>行动记录 <span>最新在上 · {view.log.length} 条</span></summary><ol aria-label="牌桌行动记录">{[...view.log].reverse().slice(0, 50).map((entry, index) => <li key={`${entry.version}-${index}`}><span className="hg-log-dot" /><p>{entry.text}</p></li>)}</ol>{!view.log.length && <p className="hg-empty">等待第一步行动。</p>}</details>
    </aside></div>
    <section className="hg-hand-dock" aria-label="固定手牌与下一步行动">
      <div className="hg-dock-action" data-next-step={view.status === 'finished' ? 'finished' : view.pendingChoice ? 'choose' : browsingOnly ? 'wait' : view.legalActions.length === 1 && globalActions[0]?.kind === 'pass' ? 'pass' : 'act'}><div><strong>{nextStep}</strong><small>{busy ? '正在确认上一行动…' : browsingOnly ? '浏览不会交出优先权，也不会显示他人的暗牌。' : '所有按钮均来自牌桌当前允许的行动。'}</small></div>
        {view.pendingChoice ? <button className="hg-button hg-button-primary" onClick={() => { const panel = document.querySelector<HTMLElement>('.hg-choice'); panel?.scrollIntoView({ behavior: 'smooth', block: 'center' }); panel?.focus(); }}>前往完成选择 ↑</button>
          : globalActions.length ? <ActionButtons actions={globalActions} busy={busy} onAction={onAction} />
          : <button className="hg-button hg-button-primary" disabled={browsingOnly || busy} onClick={() => document.querySelector<HTMLButtonElement>('.hg-card[data-card-actionable="true"]')?.focus()}>{browsingOnly ? '等待玩家 · 可浏览' : '选择卡牌行动'}</button>}
      </div>
      <section className="hg-hand-section"><div className="hg-section-title"><h2>你的手牌 <small>{view.hand.length} 张</small></h2><span className="hg-muted">横向滑动 · 点选查看全文与行动</span></div>
        <div className="hg-hand">{view.hand.map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} viewerId={view.you} compact actionable={view.legalActions.some(action => action.cardId === item.instanceId)} selected={selectedCard === item.instanceId} onSelect={() => selectCard(item.instanceId)} />)}{!view.hand.length && <p className="hg-empty">目前没有手牌。</p>}</div>
      </section>
    </section>
    {reading && <ReadModal card={reading} definition={definitions.get(reading.cardId || '')} viewerId={view.you} onClose={() => setReading(null)} />}
  </main>;
}
