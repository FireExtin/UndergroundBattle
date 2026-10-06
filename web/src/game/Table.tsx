import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import type { CSSProperties } from 'react';
import { CardContent, CardTile, sameNameTags, visibleCard } from './CardTile';
import { ChoicePanel } from './ChoicePanel';
import { DeckPicker } from './Lobby';
import { DeckLibrary } from './DeckLibraryPanel';
import { AutoPass } from './AutoPass';
import { ReadModal } from './ReadModal';
import { ResponseWindow, StackTargets } from './ResponseWindow';
import { actionSource, groupObjectActions } from './objectActions';
import type { ObjectActionGroup } from './objectActions';
import './table-desktop.css';
import './object-actions.css';
import './society.css';
import './archive-desktop.css';
import './lantern-table.css';
import { ArchiveArtwork, ArchivePresentation } from './ArchivePresentation';
import type { Action, Attachment, Card, CardDefinition, Catalog, LegalAction, Player, Region, View } from './types';

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
function regionLabel(index: number, regions: Region[]) {
  const region = regions.find(item => item.index === index);
  return `地区 ${index + 1}${region ? ` · ${region.name}` : ''}`;
}
function actionLabel(action: LegalAction, regions: Region[]) {
  if (action.region === undefined) return action.label;
  const destination = regionLabel(action.region, regions);
  const arrow = action.label.lastIndexOf('→');
  return arrow < 0 ? `${action.label} · ${destination}` : `${action.label.slice(0, arrow + 1)} ${destination}`;
}
/** Display-only split of a grouped label's cost clause; accessible names and payloads keep the full label. */
function costParts(label: string): [string, string | undefined] {
  const at = label.indexOf('（费用：');
  return at < 0 ? [label, undefined] : [label.slice(0, at).trim(), label.slice(at + 1).replace(/）$/, '')];
}
function ActionButtons({ actions, busy, onAction, regions = [], onRegionPreview, confirm = false, unavailable }: { actions: LegalAction[]; busy: boolean; onAction: (action: Action) => void; regions?: Region[]; onRegionPreview?: (region: number | null) => void; confirm?: boolean; unavailable?: (action: Action) => boolean }) {
  return <div className="hg-actions">{actions.map(action => <button type="button" key={action.id} data-action-id={action.id} data-action-kind={action.kind} data-action-region={action.region} aria-controls={action.region === undefined ? undefined : `hg-region-${action.region}`} className={`hg-button ${['start', 'restart', 'ready'].includes(action.kind) ? 'hg-button-primary' : action.kind === 'pass' ? 'hg-button-quiet' : 'hg-button-action'}${confirm ? ' hg-button-confirm' : ''}`} disabled={busy || unavailable?.(action)} onClick={() => onAction(action)} title={unavailable?.(action) ? '目标当前未显示，请等待牌桌更新。' : action.description}
    onMouseEnter={() => onRegionPreview?.(action.region ?? null)} onMouseLeave={() => onRegionPreview?.(null)} onFocus={() => onRegionPreview?.(action.region ?? null)} onBlur={() => onRegionPreview?.(null)} onPointerDown={() => onRegionPreview?.(action.region ?? null)}>
    <span>{confirm ? '确认 · ' : ''}{actionLabel(action, regions)}</span>{action.description && <small>{action.description}</small>}
  </button>)}</div>;
}

function ObjectActionButtons({ groups, busy, onSelect, onAction, regions, unavailable }: {
  groups: ObjectActionGroup[]; busy: boolean; onSelect: (group: ObjectActionGroup) => void;
  onAction: (action: Action) => void; regions: Region[]; unavailable: (action: Action) => boolean;
}) {
  return <div className="hg-object-actions">{groups.map(group => {
    const destination = group.actions.some(action => action.cardId && (action.targetId
      || regions.some(region => region.index === action.region)));
    const description = group.actions[0].description;
    const [name, cost] = costParts(group.label);
    return destination ? <button key={group.key} type="button" className="hg-button hg-button-action" aria-label={description ? `${group.label} · ${description}` : group.label} data-object-action={group.key} disabled={busy} onClick={() => onSelect(group)}>
      <span>{name}</span>{cost && <span className="hg-action-cost">{cost}</span>}{description && <small>{description}</small>}<small>点选高亮{group.actions.some(action => action.targetId) ? '目标' : '地区'} → 确认</small>
    </button> : <ActionButtons key={group.key} actions={group.actions} busy={busy} onAction={onAction} regions={regions} unavailable={unavailable} />;
  })}</div>;
}

export function RoomLobby({ view, catalog, busy, onAction }: { view: View; catalog: Catalog | null; busy: boolean; onAction: (action: Action) => void }) {
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);
  const you = view.players.find(player => player.id === view.you);
  const capacity = view.mode === 'teams' ? 4 : 2;
  const yourDeck = catalog?.decks.find(deck => deck.id === you?.deckId);
  const canChangeDeck = view.legalActions.some(action => action.kind === 'deck');
  const canChooseSavedDeck = !!catalog?.deckBuildRules && !!view.yourDeck && canChangeDeck;
  const starts = view.legalActions.filter(action => action.kind === 'start' || action.kind === 'ready');
  return <main className="hg-room-lobby">
    <div className="hg-room-welcome"><span className="hg-eyebrow">牌桌已经就绪</span><h1>等待秘社集结</h1><p>{view.mode === 'duel' ? '两人对决 · 3 个地区 · 8 分胜利' : '四人协作 · 5 个地区 · 团队 10 分胜利'}</p></div>
    <section className="hg-room-controls" aria-label="准备与开始"><div><span className="hg-eyebrow">席位 {(you?.seat ?? 0) + 1} · 你的牌组</span><h2>{you?.deckName || yourDeck?.name || '秘社牌组'}</h2></div>
      <div className="hg-room-ready"><span>{view.players.filter(player => player.ready).length} / {capacity} 位玩家已准备</span><ActionButtons actions={starts} busy={busy} onAction={onAction} />{!starts.some(action => action.kind === 'start') && <p>{you?.seat === 0 ? '等所有座位入席并准备后，即可开始。' : '准备后等待房主开始对局。'}</p>}</div>
    </section>
    <section className="hg-invite"><div><span className="hg-eyebrow">邀请伙伴入席</span><strong>{view.inviteCode}</strong><p>分享邀请码或邀请链接。伙伴使用自己的浏览器入席；本机再次加入会恢复原座位。</p></div><button className="hg-button hg-button-primary" onClick={async () => {
      try { await navigator.clipboard.writeText(`${location.origin}/?invite=${encodeURIComponent(view.inviteCode)}`); setCopied(true); setCopyError(false); }
      catch { setCopyError(true); }
    }}>{copied ? '✓ 邀请链接已复制' : '复制邀请链接 ↗'}</button>{copyError && <p role="status">浏览器无法复制，请手动分享上方邀请码。</p>}</section>
    <div className="hg-seats">{Array.from({ length: capacity }, (_, seat) => {
      const player = view.players.find(item => item.seat === seat);
      const team = player?.team ?? (view.mode === 'teams' ? (seat < 2 ? 0 : 1) : seat);
      return <article key={seat} className={`hg-seat hg-team-${team} ${player ? '' : 'hg-seat-empty'}`}><span className="hg-eyebrow">席位 {seat + 1} · {team === you?.team ? '我方' : '对方'}</span>
        <span className="hg-player-avatar">{player ? player.name.slice(0, 1) : '+'}</span><strong>{player?.name || '等待伙伴'}{player?.id === view.you && <small> 你</small>}{seat === 0 && player && <small> 房主</small>}</strong>
        <span>{player ? player.deckName || catalog?.decks.find(deck => deck.id === player.deckId)?.name || '秘社牌组' : '用邀请码加入'}</span><span className={`hg-ready ${player?.ready ? 'hg-is-ready' : ''}`}>{player ? player.ready ? '✓ 已准备' : '尚未准备' : '空座'}</span>
        {player && <section className="hg-lobby-society" aria-label={`${player.name}的秘社区`} data-society-zone={view.societyZones?.find(zone => zone.playerId === player.id)?.id}><strong>秘社区</strong><small>{view.societyZones?.some(zone => zone.playerId === player.id) ? '开局时同时公开' : '未开放'}</small></section>}
      </article>;
    })}</div>
    {catalog && <details className="hg-room-prepare"><summary>更换牌组 <span>当前：{you?.deckName || yourDeck?.name || '秘社牌组'}</span></summary><p className="hg-muted">选择其他牌组后将取消准备，须重新准备。编辑或保存草稿不会改变当前入席牌组。</p>
      <DeckPicker decks={catalog.decks} cards={catalog.cards} value={you?.deckId || ''} allowedIds={view.legalActions.filter(action => action.kind === 'deck').map(action => action.option || '')} disabled={busy || !canChangeDeck} onChange={id => {
        const action = view.legalActions.find(item => item.kind === 'deck' && item.option === id); if (!busy && action) onAction(action);
      }} />
      {canChooseSavedDeck && <DeckLibrary catalog={catalog} disabled={busy} selectedDraftId={view.yourDeck?.id} onSelectDraft={deckDraft => {
        if (!busy && canChooseSavedDeck) onAction({ kind: 'deck', deckDraft });
      }} />}
    </details>}
  </main>;
}

function PlayerMat({ player, view, catalog, definitions, selectedCard, selectedTarget, targetIds, picking, tagFor, onCard, onPreview, onReadDeckTop }: {
  player: Player; view: View; catalog: Catalog | null; definitions: Map<string, CardDefinition>;
  selectedCard: string | null; selectedTarget: string | null; targetIds: Set<string>; picking: boolean; tagFor: (card: Card) => string | undefined; onCard: (id: string) => void; onPreview: (id: string | null) => void; onReadDeckTop: () => void;
}) {
  const assets = view.assets.filter(card => card.controller === player.id);
  const mine = player.id === view.you;
  const societyZone = view.societyZones?.find(zone => zone.playerId === player.id);
  const ownTeam = view.players.find(item => item.id === view.you)?.team ?? 0;
  const renderCard = (card: Card) => <CardTile key={card.instanceId} card={card} definition={definitions.get(card.cardId || '')} viewerId={view.you} compact owner={view.players.find(item => item.id === card.owner)} controller={view.players.find(item => item.id === card.controller)} sameNameTag={tagFor(card)} selected={selectedCard === card.instanceId || selectedTarget === card.instanceId} targeted={targetIds.has(card.instanceId)} actionable={view.legalActions.some(action => actionSource(action) === card.instanceId)} onSelect={() => onCard(card.instanceId)} onPreview={() => onPreview(card.instanceId)} onPreviewEnd={() => onPreview(null)} />;
  const renderPile = (kind: 'graveyard' | 'scoreCards', label: string) => {
    const cards = view[kind].filter(card => card.owner === player.id);
    const hasTargets = picking && cards.some(card => targetIds.has(card.instanceId));
    return <details open={hasTargets || undefined} className={`hg-public-zones hg-table-pile hg-${kind}-pile${hasTargets ? ' hg-pile-targeted' : ''}`} data-pile-owner={player.id} data-pile-kind={kind} data-pile-targeted={hasTargets}>
      <summary aria-label={`查看${player.name}的${label}，${cards.length}张`}><span className="hg-pile-card" aria-hidden="true"><b>{kind === 'scoreCards' ? player.score : cards.length}</b><i>{kind === 'scoreCards' ? '分' : '张'}</i></span><span>{label}</span></summary>
      <div className="hg-pile-fan"><h3>{player.name} · {label}{hasTargets ? ' · 点选合法目标' : ''}</h3><div className="hg-zone-cards">{(hasTargets ? cards.filter(card => targetIds.has(card.instanceId)) : cards).map(renderCard)}{!cards.length && <p className="hg-empty">这里还没有牌</p>}</div></div>
    </details>;
  };
  return <section className={`hg-player-mat hg-team-${player.team}${mine ? ' hg-player-you' : ''}`} data-player-zone={player.id} data-team={player.team} data-side={player.team === ownTeam ? 'mine' : 'opponent'} aria-label={`${player.name}的玩家区`}>
    <div className="hg-player-strip"><span className="hg-piece-avatar" aria-hidden="true">{player.seat + 1}</span>{targetIds.has(player.id) ? <button type="button" className="hg-player-target" data-player-target={player.id} aria-pressed={selectedTarget === player.id} onClick={() => onCard(player.id)}>选择玩家{player.name}</button> : <strong>{player.name}{mine ? ' · 你' : player.team === ownTeam ? ' · 队友' : ' · 对方'}</strong>}<small>席位 {player.seat + 1} · {player.deckName || catalog?.decks.find(deck => deck.id === player.deckId)?.name || '秘社'}{player.eliminated ? ' · 已出局' : ''}</small><span className="hg-mat-resource">费用 <b>{assets.filter(card => !card.exhausted).length}</b> / {assets.length}</span></div>
    <div className="hg-seat-zones">
      <div className="hg-personal-piles"><div className="hg-table-pile hg-deck-pile" aria-label={`${player.name}的牌库，${player.deckCount}张`}><span className="hg-pile-card hg-card-back" aria-hidden="true"><span>◈</span><b>{player.deckCount}</b></span><span>牌库</span>{mine && view.privateDeckTop && <button type="button" className="hg-small-read" onClick={onReadDeckTop}>检视顶牌</button>}</div>{renderPile('graveyard', '墓地')}{renderPile('scoreCards', '计分')}</div>
      <div className="hg-mat-assets" aria-label={`${player.name}的资产`}><span className="hg-mat-zone-label">资产 · 费用与忠诚</span><div className="hg-assets" style={{ '--hg-card-count': Math.max(1, assets.length) } as CSSProperties}>{assets.map(renderCard)}{!assets.length && <span className="hg-table-empty">尚无资产</span>}</div></div>
      <section className="hg-society-zone" aria-label={`${player.name}的秘社区`} data-society-zone={societyZone?.id} data-society-player={player.id}>
        <strong>秘社区</strong>
        {societyZone?.card ? <><div className="hg-society-piece">{renderCard(societyZone.card)}</div><small>{societyZone.card.exhausted ? '已横置' : '未横置'}</small></> : <><span aria-hidden="true">◈</span><small>{!societyZone ? '未开放' : view.status === 'lobby' ? '开局时公开' : '未选择'}</small></>}
      </section>
    </div>
    {!mine && <div className="hg-other-hand" aria-label={`${player.name}的手牌，${player.handCount}张，身份隐藏`} data-hand-owner={player.id} data-hand-count={player.handCount}><span>手牌 {player.handCount}</span><div aria-hidden="true">{Array.from({ length: Math.min(player.handCount, 8) }, (_, index) => <i key={index} className="hg-card-back">◈</i>)}</div></div>}
  </section>;
}

function RegionTile({ region, view, definitions, selected, targeted, available, onRegion, onCard, onPreview, selectedCard, targetIds, selectedTarget, tagFor }: {
  region: Region; view: View; definitions: Map<string, CardDefinition>; selected: boolean; targeted: boolean; available: boolean;
  onRegion: () => void; onCard: (id: string) => void; onPreview: (id: string | null) => void; selectedCard: string | null; targetIds: Set<string>; selectedTarget: string | null;
  tagFor: (card: Card) => string | undefined;
}) {
  const mountedTag = (item: Attachment) => { const tag = tagFor(item); return tag && <small className="hg-instance-tag">#{tag}</small>; };
  // Mounted objects keep their names; seat, control and instance details are display-only extras.
  const seatOf = (id: string) => view.players.find(player => player.id === id);
  const mountedSeat = (item: Attachment) => { const controller = seatOf(item.controller); return controller && <span className={`hg-piece-avatar hg-team-${controller.team}`} aria-hidden="true">{controller.seat + 1}</span>; };
  const mountedDescription = (item: Attachment) => {
    const owner = seatOf(item.owner); const controller = seatOf(item.controller); const tag = tagFor(item);
    const named = (player: Player) => `${player.name}${player.id === view.you ? '（你）' : ''}`;
    return [owner && `${named(owner)} 拥有${controller && controller.id !== owner.id ? `，${named(controller)} 操控` : ''}`, tag && `同名对象 #${tag}`].filter(Boolean).join('；') || undefined;
  };
  const yourTeam = view.players.find(player => player.id === view.you)?.team ?? 0;
  const renderSide = (team: number) => {
    const characters = region.characters.filter(card => view.players.find(player => player.id === card.controller)?.team === team);
    const mine = yourTeam === team;
    const icons = region.iconsByTeam?.[team];
    return <div className={`hg-region-side hg-team-${team}`} data-team={team} data-side={mine ? 'mine' : 'opponent'}><div className="hg-region-side-label"><span>{mine ? '我方' : '对方'} · {characters.length} 张</span>{icons && <span className="hg-region-contest-icons" role="img" aria-label={`当前对抗图标：${mine ? '我方' : '对方'}，调查${icons.investigation}，战斗${icons.combat}，势力${icons.influence}`}>◈ <b>{icons.investigation}</b><i>·</i>⚔ <b>{icons.combat}</b><i>·</i>⚑ <b>{icons.influence}</b></span>}<span>势力标志 <b>{region.influence[team] || 0}</b></span></div>
      <div className="hg-region-characters" style={{ '--hg-card-count': Math.max(1, characters.length) } as CSSProperties} aria-label={`${region.name}${mine ? '我方' : '对方'}角色`}>{characters.map(card => <div key={card.instanceId} className="hg-region-piece"><CardTile card={card} definition={definitions.get(card.cardId || '')} viewerId={view.you} owner={view.players.find(player => player.id === card.owner)} controller={view.players.find(player => player.id === card.controller)} sameNameTag={tagFor(card)} actionable={view.legalActions.some(action => actionSource(action) === card.instanceId)} compact attachmentCount={view.attachments?.filter(attachment => attachment.hostId === card.instanceId).length} selected={selectedCard === card.instanceId || selectedTarget === card.instanceId} targeted={targetIds.has(card.instanceId)} onSelect={() => onCard(card.instanceId)} onPreview={() => onPreview(card.instanceId)} onPreviewEnd={() => onPreview(null)} />{view.attachments?.filter(item => item.hostId === card.instanceId).map(item => <button type="button" key={item.instanceId} className="hg-mounted-object" data-attachment-instance={item.instanceId} data-card-targeted={targetIds.has(item.instanceId)} aria-pressed={selectedCard === item.instanceId || selectedTarget === item.instanceId} onClick={() => onCard(item.instanceId)} onMouseEnter={() => onPreview(item.instanceId)} onMouseLeave={() => onPreview(null)} aria-label={`查看附属${visibleCard(item, view.you).name}`} aria-description={mountedDescription(item)} data-control-differs={item.controller !== item.owner || undefined}>{mountedSeat(item)}{visibleCard(item, view.you).name}{mountedTag(item)}</button>)}</div>)}{!characters.length && <span className="hg-region-vacant">争夺区</span>}</div>
    </div>;
  };
  return <article id={`hg-region-${region.index}`} aria-label={regionLabel(region.index, view.regions)} data-region-targeted={targeted} data-region-available={available} className={`hg-region ${selected ? 'hg-selected-region' : ''} ${targeted ? 'hg-targeted' : ''} ${available ? 'hg-region-available' : ''}`}>
    {renderSide(1 - yourTeam)}
    <button type="button" className="hg-region-center" data-region-index={region.index} onClick={onRegion} aria-pressed={selected} aria-label={`查看地区${region.index + 1} ${region.name}`}>
      <ArchiveArtwork key={region.cardId} card={{ instanceId: region.id, cardId: region.cardId, name: region.name, kind: 'region', owner: view.you, controller: view.you, exhausted: false, faceDown: false }} />
      <span className="hg-region-number">{region.index + 1}</span><span className="hg-region-title"><small className={region.skipConfrontation ? 'hg-region-skip' : undefined}>{region.skipConfrontation ? '本回合略过对抗比较' : '隐秘地区'}</small><strong>{region.name}</strong></span><span className="hg-region-value"><b>{region.points}</b><small>分</small></span>
      <span className="hg-region-threshold">控制阈值 <b>{region.threshold}</b>{available && <em>合法落点</em>}</span>
    </button>
    {!!view.attachments?.some(item => item.hostId === region.id) && <div className="hg-region-attachments" aria-label={`${region.name}的地区附属`}>{view.attachments.filter(item => item.hostId === region.id).map(item => <button type="button" key={item.instanceId} className="hg-mounted-object" data-attachment-instance={item.instanceId} data-attachment-host={region.id} data-card-targeted={targetIds.has(item.instanceId)} aria-pressed={selectedCard === item.instanceId || selectedTarget === item.instanceId} onClick={() => onCard(item.instanceId)} onMouseEnter={() => onPreview(item.instanceId)} onMouseLeave={() => onPreview(null)} aria-label={`查看地区附属${visibleCard(item, view.you).name}`} aria-description={mountedDescription(item)} data-control-differs={item.controller !== item.owner || undefined}>{mountedSeat(item)}{visibleCard(item, view.you).name}{mountedTag(item)}</button>)}</div>}
    {renderSide(yourTeam)}
  </article>;
}

type TableProps = { view: View; catalog: Catalog | null; busy: boolean; uncertain?: boolean; connection?: 'connecting' | 'online' | 'offline'; modalPaused?: boolean; onAction: (action: Action) => void; autoPassEnabled?: boolean; onAutoPassEnabledChange?: (enabled: boolean) => void };

export function Table(props: TableProps) {
  // Even equal versions/instance IDs in another room or seat start a new local interaction.
  return <ArchivePresentation.Provider value><TableSurface key={JSON.stringify([props.view.roomId, props.view.you])} {...props} /></ArchivePresentation.Provider>;
}

function TableSurface({ view, catalog, busy, uncertain = false, connection = 'connecting', modalPaused = false, onAction, autoPassEnabled = false, onAutoPassEnabledChange = () => {} }: TableProps) {
  const [selectedCard, setSelectedCard] = useState<string | null>(null);
  const [selectedTarget, setSelectedTarget] = useState<string | null>(null);
  const [selectedRegion, setSelectedRegion] = useState<number | null>(null);
  const [previewRegion, setPreviewRegion] = useState<number | null>(null);
  const [hoveredCard, setHoveredCard] = useState<string | null>(null);
  const [reading, setReading] = useState<{ instanceId: string; roomId: string; viewerId: string; privateDeckTop?: boolean } | null>(null);
  const [draft, setDraft] = useState<{ key: string; version: number } | null>(null);
  const [targetNotice, setTargetNotice] = useState('');
  const submittedDraft = useRef(false);
  const tableRoot = useRef<HTMLElement>(null);
  const nextFocus = useRef<'targets' | 'actions' | 'confirm' | null>(null);
  const definitions = useMemo(() => new Map<string, CardDefinition>([...(catalog?.cards || []), ...(catalog?.societies || [])].map(card => [card.id, card])), [catalog]);
  const attachments = view.attachments || [];
  const societyCards = view.players.flatMap(player => {
    const card = view.societyZones?.find(zone => zone.playerId === player.id)?.card;
    return card ? [card] : [];
  });
  const allCards = [...view.hand, ...view.assets, ...view.graveyard, ...view.scoreCards, ...(view.revealedHands || []).flatMap(hand => hand.cards), ...view.regions.flatMap(region => region.characters), ...attachments, ...societyCards].map(item => visibleCard(item, view.you));
  const presentedIds = new Set([...allCards.map(card => card.instanceId), ...view.players.map(player => player.id), ...view.regions.map(region => region.id)]);
  // Same-name tags cover table pieces only; revealed hands remain text readers.
  const sameNames = sameNameTags([...view.hand, ...view.assets, ...view.graveyard, ...view.scoreCards, ...view.regions.flatMap(region => region.characters), ...attachments, ...societyCards]);
  const tagFor = (item: Card) => sameNames.get(item.instanceId);
  const zoneOf = (item: Card) => {
    const within = (cards: Card[]) => cards.some(candidate => candidate.instanceId === item.instanceId);
    const home = view.regions.find(candidate => within(candidate.characters));
    const mounted = attachments.find(candidate => candidate.instanceId === item.instanceId);
    return home ? regionLabel(home.index, view.regions) : mounted ? `附属${mounted.region !== undefined ? ` · ${regionLabel(mounted.region, view.regions)}` : ''}`
      : within(view.hand) ? '你的手牌' : within(view.assets) ? '资产区' : within(view.graveyard) ? '墓地' : within(view.scoreCards) ? '计分区' : within(societyCards) ? '秘社区' : undefined;
  };
  const playerName = (id: string) => view.players.find(player => player.id === id)?.name || '玩家';
  // Shown only when it adds a distinction the tile cannot: differing control or a same-name instance.
  const objectContext = (item: Card) => {
    const tag = tagFor(item);
    if (item.controller === item.owner && !tag) return undefined;
    return [`${playerName(item.owner)} 拥有${item.controller !== item.owner ? ` · ${playerName(item.controller)} 操控` : ''}`, zoneOf(item), tag && `同名对象 #${tag}`].filter(Boolean).join(' · ');
  };
  const unavailable = (action: Action) => !!action.targetId && !presentedIds.has(action.targetId)
    || action.region !== undefined && !view.regions.some(region => region.index === action.region);
  const card = allCards.find(item => item.instanceId === selectedCard);
  const hover = allCards.find(item => item.instanceId === hoveredCard);
  const readCard = (item: Card) => setReading({ instanceId: item.instanceId, roomId: view.roomId, viewerId: view.you });
  const readingRegion = view.regions.find(item => item.id === reading?.instanceId);
  const regionCard: Card | undefined = readingRegion ? { instanceId: readingRegion.id, cardId: readingRegion.cardId, name: readingRegion.name, kind: 'region', owner: view.you, controller: view.you, exhausted: false, faceDown: false } : undefined;
  // An open reader resolves only against the latest authorized projection, never an old face payload.
  const readingCard = reading?.roomId === view.roomId && reading.viewerId === view.you
    ? reading.privateDeckTop
      ? view.privateDeckTop?.instanceId === reading.instanceId ? visibleCard(view.privateDeckTop, view.you) : undefined
      : allCards.find(item => item.instanceId === reading.instanceId)
      || view.pendingChoice?.options.map(option => option.card && visibleCard(option.card, view.you)).find(item => item?.instanceId === reading.instanceId)
      || regionCard : undefined;
  const attachedCards = card ? attachments.filter(item => item.hostId === card.instanceId) : [];
  const attachmentContext = (item: Attachment, includeInstance = true) => {
    const host = allCards.find(candidate => candidate.instanceId === item.hostId);
    const hostRegion = view.regions.find(candidate => candidate.id === item.hostId);
    return `${playerName(item.owner)} 拥有${item.controller !== item.owner ? ` · ${playerName(item.controller)} 操控` : ''} · 附着于 ${hostRegion ? `地区${hostRegion.name}` : host ? `${playerName(host.owner)} 的${host.name}` : '宿主'}${item.region !== undefined ? ` · ${regionLabel(item.region, view.regions)}` : ''}${includeInstance && tagFor(item) ? ` · 同名对象 #${tagFor(item)}` : ''}`;
  };
  const selectedAttachment = attachments.find(item => item.instanceId === selectedCard);
  const readingAttachment = attachments.find(item => item.instanceId === readingCard?.instanceId);
  const societyContext = (item: Card) => {
    const zone = view.societyZones?.find(zone => zone.card?.instanceId === item.instanceId);
    return zone ? `${view.players.find(player => player.id === zone.playerId)?.name || '玩家'}的秘社区 · 不属于地区` : undefined;
  };
  const region = view.regions.find(item => item.index === selectedRegion);
  const you = view.players.find(player => player.id === view.you);
  const yourTeam = you?.team ?? 0;
  const teammates = view.players.filter(player => player.team === yourTeam).sort((a, b) => a.seat - b.seat);
  const opponents = view.players.filter(player => player.team !== yourTeam).sort((a, b) => a.seat - b.seat);
  useEffect(() => { if (selectedCard && !card) { setSelectedCard(null); setSelectedTarget(null); } }, [selectedCard, card]);
  useEffect(() => { if (reading && !readingCard) setReading(null); }, [reading, readingCard]);
  useEffect(() => { setPreviewRegion(null); }, [view.version, selectedCard]);
  const sourceActions = view.legalActions.filter(action => action.kind !== 'choose' && selectedCard && actionSource(action) === selectedCard);
  const groups = groupObjectActions(sourceActions);
  const activeGroup = draft?.version === view.version && !view.pendingChoice && !view.waitingChoice
    ? groups.find(group => group.key === draft.key) : undefined;
  const destinations = activeGroup?.actions || sourceActions;
  const targetIds = new Set(destinations.filter(action => action.cardId === selectedCard && action.targetId && (selectedRegion === null || action.region === selectedRegion)).map(action => action.targetId!));
  const regionIds = new Set(destinations.filter(action => action.cardId === selectedCard && action.region !== undefined && (!selectedTarget || action.targetId === selectedTarget)).map(action => action.region!));
  const actionBusy = busy || uncertain || !!view.pendingChoice || !!view.waitingChoice;
  useEffect(() => {
    if (draft && !activeGroup) {
      setDraft(null); setSelectedTarget(null); setSelectedRegion(null);
      setTargetNotice('牌桌已更新，请重新选择动作与目标。');
    }
  }, [draft, activeGroup]);
  const beginTargeting = (group: ObjectActionGroup) => {
    if (actionBusy) return;
    submittedDraft.current = false;
    nextFocus.current = 'targets';
    setDraft({ key: group.key, version: view.version });
    setSelectedTarget(null); setSelectedRegion(null); setTargetNotice(''); setHoveredCard(null);
  };
  const cancelTargeting = () => { nextFocus.current = 'actions'; setDraft(null); setSelectedTarget(null); setSelectedRegion(null); setTargetNotice('已取消选目标，可选择其他动作。'); };
  const selectCard = (id: string) => {
    if (selectedCard && targetIds.has(id) && presentedIds.has(id)) {
      const matching = activeGroup || (groups.length === 1 ? groups[0] : undefined);
      if (matching && !actionBusy) {
        if (!activeGroup) beginTargeting(matching);
        nextFocus.current = 'confirm';
        setSelectedTarget(id); setHoveredCard(null); return;
      }
    }
    if (activeGroup) { setTargetNotice('此对象不是当前动作的合法目标，请点选高亮对象或取消。'); return; }
    setSelectedCard(id); setSelectedTarget(null); setSelectedRegion(null); setDraft(null); setTargetNotice('');
  };
  const selectRegion = (index: number) => {
    if (activeGroup) {
      if (!actionBusy && regionIds.has(index)) { nextFocus.current = 'confirm'; setSelectedRegion(index); setTargetNotice(''); }
      else setTargetNotice('此地区不是当前动作的合法目标，请点选高亮对象或取消。');
      return;
    }
    setSelectedCard(null); setDraft(null);
    setSelectedRegion(index); setSelectedTarget(null);
  };
  const clearSelection = () => { setSelectedCard(null); setSelectedTarget(null); setSelectedRegion(null); setDraft(null); setTargetNotice(''); };
  const globalActions = view.legalActions.filter(action => !['choose', 'deck', 'ready', 'start', 'restart'].includes(action.kind)
    && (actionSource(action) ? !allCards.some(card => card.instanceId === actionSource(action)) && !view.regions.some(region => region.id === actionSource(action))
      : !view.regions.some(region => region.index === action.region)));
  const cardActions = sourceActions;
  const lastHandCharacterToAsset = card?.kind === 'character' && view.hand.some(item => item.instanceId === selectedCard) && view.hand.filter(item => item.kind === 'character').length === 1 && cardActions.some(action => action.kind === 'asset' && action.cardId === selectedCard);
  const regionActions = view.legalActions.filter(action => action.kind !== 'choose' && (actionSource(action) === region?.id || (!actionSource(action) && action.region === selectedRegion)));
  const needsTarget = !!activeGroup?.actions.some(action => action.targetId);
  const needsRegion = !!activeGroup?.actions.some(action => action.region !== undefined);
  const missingDestinations = !!activeGroup?.actions.some(unavailable);
  const matchingActions = activeGroup?.actions.filter(action => !unavailable(action) && (!needsTarget || action.targetId === selectedTarget) && (!needsRegion || action.region === selectedRegion)) || [];
  const chosenTarget = allCards.find(item => item.instanceId === selectedTarget);
  const chosenTag = chosenTarget && tagFor(chosenTarget);
  const chosenZone = chosenTarget && chosenTag ? zoneOf(chosenTarget) : undefined;
  const targetLabel = chosenTarget ? `${view.players.find(player => player.id === chosenTarget.owner)?.name || '玩家'}的${chosenTarget.name}${chosenTag ? ` #${chosenTag}${chosenZone ? ` · ${chosenZone}` : ''}` : ''}`
    : view.players.find(player => player.id === selectedTarget)?.name || (selectedTarget ? '所选目标' : '尚未选择');
  const confirmTarget = (action: Action) => {
    if (!activeGroup || actionBusy || submittedDraft.current || unavailable(action) || !matchingActions.includes(action as LegalAction)) return;
    submittedDraft.current = true;
    setDraft(null); setSelectedTarget(null); setSelectedRegion(null);
    onAction(action);
  };
  const focusTarget = () => {
    const target = tableRoot.current?.querySelector<HTMLElement>('[data-card-targeted="true"], [data-player-target], [data-region-available="true"] .hg-region-center');
    target?.focus(); target?.scrollIntoView?.({ block: 'start', inline: 'nearest' });
  };
  useLayoutEffect(() => {
    const focus = nextFocus.current;
    nextFocus.current = null;
    if (!focus || readingCard || view.pendingChoice || modalPaused) return;
    if (focus === 'actions') tableRoot.current?.querySelector<HTMLButtonElement>('[data-object-action]')?.focus();
    else if (focus === 'confirm' && matchingActions.length) tableRoot.current?.querySelector<HTMLButtonElement>('[aria-label="确认目标行动"] button')?.focus();
    else focusTarget();
  });
  useEffect(() => {
    if (!activeGroup || readingCard || modalPaused) return;
    const escape = (event: KeyboardEvent) => { if (event.key === 'Escape') { event.preventDefault(); cancelTargeting(); } };
    document.addEventListener('keydown', escape);
    return () => document.removeEventListener('keydown', escape);
  });
  const choiceAction = view.legalActions.find(action => action.kind === 'choose' && (!action.choiceId || action.choiceId === view.pendingChoice?.id));
  const choicePlayer = view.pendingChoice?.playerId || view.waitingChoice?.playerId;
  const choiceName = view.players.find(player => player.id === choicePlayer)?.name;
  const status = view.pendingChoice ? '请你完成下方选择' : view.waitingChoice ? `等待 ${choiceName || '另一位玩家'}：${view.waitingChoice.title}` : view.legalActions.length ? '轮到你参与行动' : `等待 ${teamName(view.priorityTeam, view)} 行动`;
  const browsingOnly = view.status !== 'playing' || !view.legalActions.some(action => action.kind !== 'choose') || !!view.waitingChoice;
  const nextStep = view.status === 'finished' ? '本局已结束，可回顾牌桌' : view.pendingChoice ? `下一步：${view.pendingChoice.title}` : view.waitingChoice ? `等待 ${choiceName || '另一位玩家'} 完成选择，可浏览牌桌` : activeGroup ? matchingActions.length ? '下一步：确认所选目标，或取消' : `下一步：点选高亮${needsTarget && !selectedTarget ? '目标' : '地区'}，或取消` : !view.legalActions.length ? `等待 ${teamName(view.priorityTeam, view)} 行动，可浏览牌桌` : view.legalActions.length === 1 && globalActions[0]?.kind === 'pass' ? '下一步：让过，推进当前窗口' : societyCards.length ? '下一步：点选手牌、角色、附属、秘社或地区查看动作' : '下一步：点选手牌、角色、附属或地区查看动作';
  const renderMat = (player: Player) => <PlayerMat key={player.id} player={player} view={view} catalog={catalog} definitions={definitions} selectedCard={selectedCard} selectedTarget={selectedTarget} targetIds={targetIds} picking={!!activeGroup} tagFor={tagFor} onCard={selectCard} onPreview={setHoveredCard} onReadDeckTop={() => { if (view.privateDeckTop) setReading({ instanceId: view.privateDeckTop.instanceId, roomId: view.roomId, viewerId: view.you, privateDeckTop: true }); }} />;
  return <main ref={tableRoot} className={`hg-table hg-desktop-table hg-archive-table hg-lantern-table hg-desktop-${view.mode}`} data-table-layout="overhead" data-table-presentation="lamplit">
    <div className="hg-table-status"><div className="hg-round"><b>第 {view.turn} 回合</b><span>{phaseLabel(view.phase)} · {phaseLabel(view.step)}</span></div><div className="hg-table-tallies">{[yourTeam, 1 - yourTeam].map(team => <span key={team} className={`hg-team-tally hg-team-${team}`}>{team === yourTeam ? '我方' : '对方'} <b>{view.players.filter(player => player.team === team).reduce((total, player) => total + player.score, 0)}</b> / {view.winScore}</span>)}</div><div className="hg-phase-line"><span>先手 <b>{teamName(view.firstTeam, view)}</b></span><span>行动团队 <b>{teamName(view.activeTeam, view)}</b></span><span>优先权 <b>{teamName(view.priorityTeam, view)}</b></span><strong className={view.pendingChoice ? 'hg-your-choice' : ''}>{status}</strong></div><details className="hg-table-records"><summary>记录 · {view.log.length}</summary><ol aria-label="牌桌行动记录">{[...view.log].reverse().slice(0, 50).map((entry, index) => <li key={`${entry.version}-${index}`}>{entry.text}</li>)}</ol><small>规则 {view.versions.rules} · 卡池 {view.versions.cardPool} · 引擎 {view.versions.engine}</small></details></div>
    {(view.revealedHands || []).map(hand => <section key={hand.playerId} className="hg-revealed-hand" aria-label="公开展示的手牌"><strong>公开展示的手牌 · {view.players.find(player => player.id === hand.playerId)?.name || '玩家'}</strong><div className="hg-hand">{hand.cards.map(item => <button key={item.instanceId} type="button" className="hg-small-read" onClick={() => readCard(item)} aria-label={`放大阅读展示的${item.name}`}><CardContent card={item} definition={definitions.get(item.cardId || '')} viewerId={view.you} /></button>)}</div></section>)}
    <section className="hg-team-edge hg-opponent-edge" aria-label="对方玩家区" data-side="opponent">{opponents.map(renderMat)}</section>
    <section className="hg-battlefield" aria-label="中央世界与争夺区，可上下滚动查看双方角色" tabIndex={0} title="中央地区可上下滚动；手牌与行动栏保持在下方。"><div className="hg-world-deck" aria-label="世界牌库"><span className="hg-card-back">⌖{view.worldDeckCount !== undefined && <b>{view.worldDeckCount}</b>}</span><small>世界牌库</small><small className="hg-battlefield-scroll-hint">↕ 滚动查看</small></div><div className={`hg-board hg-board-${view.mode}`}>{view.regions.map(item => <RegionTile key={item.id} region={item} view={view} definitions={definitions} selected={selectedRegion === item.index} targeted={previewRegion === item.index || (!!activeGroup && regionIds.has(item.index))} selectedCard={selectedCard} targetIds={targetIds} selectedTarget={selectedTarget} available={activeGroup ? regionIds.has(item.index) : view.legalActions.some(action => action.region === item.index && (!selectedCard || action.cardId === selectedCard))} onRegion={() => selectRegion(item.index)} onCard={selectCard} onPreview={setHoveredCard} tagFor={tagFor} />)}</div></section>
    <section className="hg-team-edge hg-mine-edge" aria-label="我方玩家区" data-side="mine">{teammates.map(renderMat)}</section>
    <section className="hg-hand-dock" aria-label="固定手牌与下一步行动">
      <div className={`hg-response-layer ${view.stack.length ? 'hg-response-layer-active' : ''}`}><ResponseWindow view={view} busy={busy} uncertain={uncertain} connection={connection} onAction={onAction} onSelectCard={id => { clearSelection(); setSelectedCard(id); }} />{view.stack.length > 1 && <details className="hg-stack"><summary>其余待结算效果 · {view.stack.length - 1}</summary><ol>{[...view.stack].reverse().slice(1).map(effect => <li key={effect.id}><strong>{effect.label}</strong><small>{view.players.find(player => player.id === effect.controller)?.name}</small><StackTargets effect={effect} view={view} /></li>)}</ol></details>}</div>
      <div className="hg-dock-action" data-targeting={!!activeGroup} data-next-step={view.status === 'finished' ? 'finished' : view.pendingChoice ? 'choose' : browsingOnly ? 'wait' : view.legalActions.length === 1 && globalActions[0]?.kind === 'pass' ? 'pass' : 'act'}><div><strong>{nextStep}</strong><small>{busy ? '正在确认上一行动…' : '悬停阅读 · 点选行动 · 横置角色旋转 90°'}</small></div><AutoPass view={view} busy={busy} uncertain={uncertain} connection={connection} onAction={onAction} enabled={autoPassEnabled} onEnabledChange={onAutoPassEnabledChange} />
        {view.pendingChoice ? <button className="hg-button hg-button-primary" onClick={() => document.querySelector<HTMLElement>('.hg-choice')?.focus()}>前往完成选择 ↑</button> : globalActions.length ? <ActionButtons actions={globalActions} busy={actionBusy} onAction={onAction} unavailable={unavailable} /> : <button className="hg-button hg-button-primary" disabled={browsingOnly || busy} onClick={() => document.querySelector<HTMLButtonElement>('.hg-card[data-card-actionable="true"]')?.focus()}>{browsingOnly ? '等待玩家 · 可浏览' : '选择卡牌行动'}</button>}
      </div>
      <section className="hg-hand-section" aria-label="你的手牌"><span className="hg-hand-label">你的手牌 <b>{view.hand.length}</b></span><div className="hg-hand" style={{ '--hg-card-count': Math.max(1, view.hand.length) } as CSSProperties}>{view.hand.map(item => <CardTile key={item.instanceId} card={item} definition={definitions.get(item.cardId || '')} viewerId={view.you} compact sameNameTag={tagFor(item)} actionable={view.legalActions.some(action => actionSource(action) === item.instanceId)} selected={selectedCard === item.instanceId} targeted={targetIds.has(item.instanceId)} onSelect={() => selectCard(item.instanceId)} onPreview={() => setHoveredCard(item.instanceId)} onPreviewEnd={() => setHoveredCard(null)} />)}{!view.hand.length && <p className="hg-empty">目前没有手牌。</p>}</div></section>
    </section>
    {(card || region) && <aside className="hg-inspector hg-inspector-active" aria-label="选牌行动"><div className="hg-section-title"><span className="hg-eyebrow">{browsingOnly ? '浏览牌桌 · 暂不可行动' : '当前可用行动'}</span><button className="hg-inspector-close" onClick={clearSelection} aria-label="关闭卡牌详情">×</button></div>
      {card ? <><div className="hg-inspected-card" role="button" tabIndex={0} aria-label={`放大阅读${card.name}`} onClick={() => readCard(card)} onKeyDown={event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); readCard(card); } }}><CardContent card={card} definition={definitions.get(card.cardId || '')} viewerId={view.you} /></div><button type="button" className="hg-small-read" onClick={() => readCard(card)}>放大文字与图标 ↗</button>{selectedAttachment ? <p className="hg-attachment-context">{attachmentContext(selectedAttachment)}</p> : objectContext(card) && <p className="hg-object-context">{objectContext(card)}</p>}{attachedCards.length > 0 && <section className="hg-attached-cards" aria-label="此角色的附属"><h3>附属 · {attachedCards.length} 张</h3>{attachedCards.map(item => <button key={item.instanceId} type="button" className="hg-attachment-link" data-attachment-instance={item.instanceId} data-attachment-host={item.hostId} onClick={() => readCard(item)} aria-label={`放大阅读附属${visibleCard(item, view.you).name}`}><span><strong>{visibleCard(item, view.you).name}</strong><small>{attachmentContext(item, false)}</small>{tagFor(item) && <small>同名对象 #{tagFor(item)}</small>}</span><span>阅读 ↗</span></button>)}{attachedCards.map(item => <button key={`action-${item.instanceId}`} type="button" className="hg-small-read" onClick={() => selectCard(item.instanceId)} aria-label={`查看附属动作${visibleCard(item, view.you).name}`}>查看{visibleCard(item, view.you).name}的动作</button>)}</section>}{lastHandCharacterToAsset && <p className="hg-strategy-note">转为资产后，手中将暂时没有角色。资产提供费用与忠诚，但不能参与对抗。</p>}<ObjectActionButtons groups={activeGroup ? [] : groups} busy={actionBusy} onSelect={beginTargeting} onAction={onAction} regions={view.regions} unavailable={unavailable} />{!cardActions.length && <p className="hg-wait-note">当前时点此牌没有可执行行动，可继续浏览牌桌。</p>}</> : region && <><h2>{regionLabel(region.index, view.regions)}</h2><p>控制阈值 {region.threshold} · 赢得后 {region.points} 分</p>{region.skipConfrontation && <p className="hg-skip-explanation">本回合略过对抗比较。</p>}<p className="hg-rule-text">{definitions.get(region.cardId)?.text}</p><button type="button" className="hg-small-read" onClick={() => readCard({ instanceId: region.id, cardId: region.cardId, name: region.name, kind: 'region', owner: view.you, controller: view.you, exhausted: false, faceDown: false })}>放大地区文字 ↗</button><section className="hg-attached-cards" aria-label="此地区的附属">{attachments.filter(item => item.hostId === region.id).map(item => <button type="button" key={item.instanceId} className="hg-attachment-link" onClick={() => selectCard(item.instanceId)} aria-label={`查看地区附属动作${visibleCard(item, view.you).name}`}><strong>{visibleCard(item, view.you).name}</strong><small>{attachmentContext(item, false)}</small>{tagFor(item) && <small>同名对象 #{tagFor(item)}</small>}</button>)}</section><ActionButtons actions={regionActions} busy={actionBusy} onAction={onAction} regions={view.regions} onRegionPreview={setPreviewRegion} unavailable={unavailable} />{!regionActions.length && <p className="hg-wait-note">选择手牌可查看向此地区派遣的行动。</p>}</>}
      {targetNotice && <p role="status" className="hg-target-notice">{targetNotice}</p>}
      {activeGroup && <section className="hg-object-targeting" aria-label="点选合法目标" data-targeting-action={activeGroup.key}>
        <strong>{activeGroup.label}</strong><p role="status">{needsTarget && `目标：${targetLabel}`}{needsRegion && ` ${selectedRegion === null ? '尚未选择地区' : regionLabel(selectedRegion, view.regions)}`}</p>
        {chosenTarget && <button type="button" className="hg-small-read" onClick={() => readCard(chosenTarget)}>放大阅读所选目标</button>}
        <p>点选高亮对象后确认。未高亮的对象不会改变目标。</p>
        <div className="hg-target-navigation"><button type="button" className="hg-button hg-button-quiet" onClick={focusTarget}>前往高亮对象</button><button type="button" className="hg-button hg-button-quiet" onClick={cancelTargeting}>取消选目标</button></div>
        {missingDestinations && <p role="status">部分目标当前未显示，无法选择这些目标。请等待牌桌更新，或取消后重新选择动作。</p>}
        {matchingActions.length ? <div aria-label="确认目标行动"><ActionButtons confirm actions={matchingActions} busy={actionBusy} onAction={confirmTarget} regions={view.regions} onRegionPreview={setPreviewRegion} /></div> : <button type="button" className="hg-button hg-button-primary" disabled>点选高亮对象后确认</button>}
      </section>}
    </aside>}
    {hover && !reading && !activeGroup && <aside className="hg-card-hover" aria-label={`悬停阅读${hover.name}`} data-hover-card={hover.instanceId}><CardContent card={hover} definition={definitions.get(hover.cardId || '')} viewerId={view.you} /></aside>}
    {view.pendingChoice && <div className="hg-table-choice-layer"><ChoicePanel modal modalActive={!readingCard && !modalPaused} key={JSON.stringify([view.pendingChoice.id, view.pendingChoice.options.map(option => [option.id, option.card?.instanceId])])} choice={view.pendingChoice} action={choiceAction} definitions={definitions} busy={busy} onSubmit={onAction} onReadCard={readCard} viewerId={view.you} playerLabels={Object.fromEntries(view.players.map(player => [player.id, player.name]))} /></div>}
    {view.status === 'finished' && <section className="hg-victory" role="status"><span aria-hidden="true">✧</span><div><small>对局结束</small><h1>{view.winnerTeam === you?.team ? '你的秘社取得了霸权' : `${view.winnerTeam === undefined ? '本局' : teamName(view.winnerTeam, view)}赢得了霸权`}</h1><p>胜利目标 {view.winScore} 分 · 可以回顾牌桌，或由房主发起新一局。</p></div><ActionButtons actions={view.legalActions.filter(action => action.kind === 'restart')} busy={busy} onAction={onAction} /></section>}
    {readingCard && <ReadModal card={readingCard} definition={definitions.get(readingCard.cardId || '')} viewerId={view.you} context={reading?.privateDeckTop ? "你的牌库顶牌 · 仅你可见" : readingAttachment ? attachmentContext(readingAttachment) : societyContext(readingCard) ?? objectContext(readingCard)} onClose={() => setReading(null)} />}
  </main>;
}
