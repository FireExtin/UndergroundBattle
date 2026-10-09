import type { Card, CardDefinition, Icons, Player } from './types';
import './attachments.css';
import { ArchiveArtwork, useArchivePresentation } from './ArchivePresentation';

/** Match the server's concealed projection: only its current controller sees print. */
export function visibleCard(card: Card, viewerId: string): Card {
  if (!card.faceDown || card.controller === viewerId) {
    return card.faceDown && (card.convertedTemporaryIcons || card.currentKill !== undefined || card.currentRetreat !== undefined) ? { ...card, convertedTemporaryIcons: undefined, currentKill: undefined, currentRetreat: undefined } : card;
  }
  return { ...card, cardId: undefined, name: '暗藏者', kind: 'hidden', cost: undefined, effectiveCost: undefined,
    text: undefined, icons: undefined, convertedTemporaryIcons: undefined, currentKill: undefined, currentRetreat: undefined, defense: undefined, damage: undefined, shield: undefined,
    wounds: undefined, timeMarkers: undefined, color: undefined, magic: undefined, currentSubtypes: undefined, currentRenown: undefined, currentCombatGlory: undefined, currentBarrier: undefined, currentDamagePrevention: undefined, currentSpiritProtection: undefined, currentPrintedDefense: undefined };
}

/** Display-only short form of a server instance ID; never used to match, merge or infer objects. */
export function instanceTag(instanceId: string, digits = 6) {
  return instanceId.length <= digits + 2 ? instanceId : `…${instanceId.slice(-digits)}`;
}

/**
 * Display-only tags for presented objects sharing a public name. Face-down objects always group as
 * 暗藏者, so concealed print never affects a tag. Suffixes widen until every distinct instance differs.
 */
export function sameNameTags(cards: Card[]) {
  const groups = new Map<string, Set<string>>();
  for (const card of cards) {
    const label = card.faceDown ? '暗藏者' : card.name;
    groups.set(label, (groups.get(label) || new Set<string>()).add(card.instanceId));
  }
  const tags = new Map<string, string>();
  for (const group of groups.values()) {
    if (group.size < 2) continue;
    const ids = [...group];
    const longest = Math.max(...ids.map(id => id.length));
    let digits = 6;
    while (digits < longest && new Set(ids.map(id => instanceTag(id, digits))).size < ids.length) digits += 1;
    for (const id of ids) tags.set(id, instanceTag(id, digits));
  }
  return tags;
}

const emptyIcons: Icons = { investigation: 0, combat: 0, influence: 0 };
// Hegemony printed P11: these are the generic hidden entity's icons,
// not the concealed card's printed identity or a client-side modifier result.
const hiddenIcons: Icons = { investigation: 0, combat: 0, influence: 1 };
const hiddenRule = '暗藏者 · 基础势力 1 · 不受任何伤害';
export function IconStrip({ icons, temporary = false, label }: { icons?: Icons; temporary?: boolean; label?: string }) {
  const values = icons || emptyIcons;
  return <span className={`hg-icons ${temporary ? 'hg-icons-temporary' : ''}`} aria-label={`${label || (temporary ? '先手图标' : '图标')}：调查${values.investigation}，战斗${values.combat}，势力${values.influence}`}>
    <span title="调查">◈ <b>{values.investigation}</b></span>
    <span title="战斗">⚔ <b>{values.combat}</b></span>
    <span title="势力">⚑ <b>{values.influence}</b></span>
  </span>;
}

export function CardContent({ card: source, definition: sourceDefinition, compact = false, viewerId, attachmentCount = 0 }: { card: Card; definition?: CardDefinition; compact?: boolean; viewerId?: string; attachmentCount?: number }) {
  const archive = useArchivePresentation();
  const projected = viewerId ? visibleCard(source, viewerId) : source;
  // Explicit reading still uses controller authorization; compact backs stay public.
  const card = archive && compact && projected.faceDown
    ? { ...projected, cardId: undefined, name: '暗藏者', kind: 'hidden' } : projected;
  const definition = card.cardId ? sourceDefinition : undefined;
  if (card.kind === 'sealed') return <>
    <span className="hg-card-top"><span className="hg-card-kind">已封印 · 场外空白牌</span></span>
    <strong className="hg-card-name">{card.name}</strong>
    {!compact && archive && <ArchiveArtwork key={card.instanceId} card={card} />}
    <span className="hg-card-text">公开放置在载体上，不在场内；当前没有卡牌能力、图标或防御，不能成为效果目标。载体离场或翻暗时回到拥有者手牌。</span>
  </>;
  const hidden = !card.cardId && card.faceDown;
  const asset = card.kind === 'asset';
  const region = card.kind === 'region';
  const attachment = card.kind === 'attachment';
  const society = card.kind === 'society';
  const subtitle = definition && typeof definition.subtitle === 'string' ? definition.subtitle : undefined;
  const startingHand = society && definition && 'startingHand' in definition && typeof definition.startingHand === 'number' && Number.isSafeInteger(definition.startingHand) && definition.startingHand >= 0 ? definition.startingHand : undefined;
  const concealedCompact = compact && card.kind === 'hidden';
  const printedCost = card.cost ?? definition?.cost ?? 0;
  const actualCost = card.effectiveCost ?? printedCost;
  const adjustedCost = card.effectiveCost !== undefined && actualCost !== printedCost;
  const printedIcons = definition?.icons?.permanent || definition?.permanentIcons;
  const permanent = printedIcons || card.icons;
  const temporary = definition?.icons?.temporary || definition?.temporaryIcons;
  const hasTemporary = temporary && Object.values(temporary).some(n => n > 0);
  const effectiveStats = !compact && !card.faceDown && card.kind === 'character' && card.region !== undefined;
  const defense = card.defense ?? definition?.defense;
  const type = hidden ? '身份未公开' : archive && asset ? '资产' : card.currentSubtypes ? `角色 · ${card.currentSubtypes.join(' · ')}` : attachment ? `附属${definition?.subtypes?.length ? ` · ${definition.subtypes.join(' · ')}` : ''}` : definition?.type || definition?.subtypes?.join(' · ') || ({ character: '角色', event: '事件', spell: '咒术', region: '地区', asset: '资产', society: '秘社', hidden: '暗藏角色' }[card.kind] || '卡牌');
  return <>
    <span className="hg-card-top"><span className="hg-card-kind">{type}</span>{!hidden && !asset && !region && !society && <span className={`hg-cost${adjustedCost ? ' hg-cost-adjusted' : ''}`} title={adjustedCost ? `当前费用 ${actualCost} · 印刷费用 ${printedCost}` : '费用'} aria-label={adjustedCost ? `当前费用 ${actualCost}，印刷费用 ${printedCost}` : `费用 ${actualCost}`}>{actualCost}</span>}</span>
    <strong className="hg-card-name">{hidden ? '暗藏者' : card.name}</strong>
    {!compact && subtitle && <span className="hg-card-subtitle">{subtitle}</span>}
    {!compact && startingHand !== undefined && <span className="hg-society-hand">起手 {startingHand} 张</span>}
    {!compact && !hidden && !asset && !region && !society && adjustedCost && <span className="hg-cost-explanation">当前费用 {actualCost} · 印刷费用 {printedCost}</span>}
    {archive ? <ArchiveArtwork key={`${card.instanceId}:${card.cardId}:${card.faceDown}`} card={card} /> : !compact && <span className="hg-card-art" aria-hidden="true"><span>{hidden ? '？' : card.kind === 'event' ? '✧' : card.kind === 'region' ? '⌖' : '◈'}</span><i /></span>}
    {hidden ? <><IconStrip icons={hiddenIcons} /><span className="hg-card-text">{hiddenRule}。{card.exhausted ? '已横置，不参与对抗。' : '在地区内且未横置时参与势力对抗。'}</span></> : <>
      {!society && <span className="hg-card-affiliation">{region ? `赢得 ${definition?.points ?? '—'} 分 · 控制阈值 ${definition?.threshold ?? '—'}` : asset ? `${card.color || '无派系'}${card.magic ? ` · ${card.magic}` : ''}` : <>{definition?.loyaltyText || (definition?.loyalty?.length ? `忠诚 ${definition.loyalty.join(' / ')}` : '无忠诚要求')}{(card.magic || definition?.magic) && ` · ${card.magic || definition?.magic}`}</>}</span>}
      {!asset && !region && !attachment && !society && (effectiveStats && card.icons ? <>
        <span className="hg-card-stat-row"><small>当前有效</small><IconStrip icons={card.icons} label="当前有效图标" /></span>
        {printedIcons && <span className="hg-card-stat-row"><small>印刷图标</small><IconStrip icons={printedIcons} label="印刷图标" /></span>}
      </> : <IconStrip icons={concealedCompact ? hiddenIcons : compact ? card.icons || permanent : permanent} />)}
      {!compact && !society && hasTemporary && <span className="hg-temporary-label">{card.convertedTemporaryIcons ? '印刷临时' : '先手'} <IconStrip icons={temporary} temporary label={card.convertedTemporaryIcons ? '印刷临时图标' : undefined} /></span>}
      {!compact && !card.faceDown && card.convertedTemporaryIcons && <span className="hg-card-stat-row"><small>具现化：临时转永久</small><IconStrip icons={card.convertedTemporaryIcons} label="本地区临时转永久图标" /></span>}
      {!compact && <span className="hg-card-text">{asset ? '未横置时可提供 1 费用，并提供所示派系与魔法忠诚。' : card.text || definition?.text || '此牌没有额外能力。'}</span>}
      <span className="hg-card-footer">
        {!card.faceDown && card.region !== undefined && typeof card.currentKill === 'number' && <span>本回合杀伤 {card.currentKill}</span>}
        {!card.faceDown && card.region !== undefined && card.currentRetreat && <span>本回合撤回</span>}
        {!card.faceDown && card.region !== undefined && typeof card.currentSpiritProtection === 'boolean' && <span title="灵体按本地区双方领域数量比较防止伤害">灵体：当前{card.currentSpiritProtection ? '防止伤害' : '不防止伤害'}</span>}
        {!card.faceDown && card.region !== undefined && card.currentDamagePrevention && <span>本回合防止伤害</span>}
        {!card.faceDown && card.region !== undefined && typeof card.currentPrintedDefense === 'number' && <span>本回合印刷防御 {card.currentPrintedDefense}</span>}
        {!card.faceDown && card.region !== undefined && card.currentRenown && <span>声望</span>}
        {!card.faceDown && card.region !== undefined && card.currentCombatGlory && <span>威名</span>}
        {!card.faceDown && card.region !== undefined && card.currentBarrier && <span>屏障</span>}
        {society && card.usedOncePerGame?.map(key => <span key={key}>
          {definition?.abilities?.find(ability => ability.key === key)?.label || '每局一次能力'} · 本局已使用
        </span>)}
        {!society && !concealedCompact && typeof defense === 'number' && <span>{effectiveStats && typeof card.defense === 'number' && typeof definition?.defense === 'number' ? `当前防御 ${card.defense} · ${typeof card.currentPrintedDefense === 'number' ? '原始印刷防御' : '印刷防御'} ${definition.defense}` : `防御 ${defense}`}</span>}
        {attachmentCount > 0 && <span className="hg-card-attachment-count" title="点选宿主后可查看附属" aria-label={`附属 ${attachmentCount} 张`}>附属 {attachmentCount}</span>}
        {!society && !concealedCompact && !!card.damage && <span className="hg-hurt hg-damage-marker">伤害 {card.damage}</span>}
        {!society && !concealedCompact && !!card.wounds && <span className="hg-hurt hg-wound-marker">创伤 {card.wounds}</span>}
        {!society && !concealedCompact && !!card.shield && <span className="hg-shield-marker">护盾 {card.shield}</span>}
        {!society && !asset && !card.faceDown && !!card.timeMarkers && <span className="hg-time-marker">时间 {card.timeMarkers}</span>}
        {card.exhausted && <span className="hg-exhausted-label">已横置</span>}
        {card.faceDown && <span>{hiddenRule}{card.exhausted ? ' · 横置不参与对抗' : ''}</span>}
      </span>
    </>}
  </>;
}

export function CardTile({ card: source, definition: sourceDefinition, selected, onSelect, compact = false, viewerId, owner, controller, sameNameTag, actionable = false, targeted = false, onPreview, onPreviewEnd, attachmentCount = 0 }: {
  card: Card; definition?: CardDefinition; selected?: boolean; onSelect?: () => void; compact?: boolean;
  viewerId?: string; owner?: Player; controller?: Player; sameNameTag?: string; actionable?: boolean; targeted?: boolean; onPreview?: () => void; onPreviewEnd?: () => void;
  attachmentCount?: number;
}) {
  const archive = useArchivePresentation();
  const card = viewerId ? visibleCard(source, viewerId) : source;
  const definition = card.cardId ? sourceDefinition : undefined;
  const color = archive && compact && card.faceDown ? '' : card.color || definition?.color || '';
  const palette = color.includes('黄') ? 'gold' : color.includes('红') ? 'red' : color.includes('蓝') ? 'blue' : color.includes('绿') ? 'green' : color.includes('紫') ? 'violet' : 'neutral';
  // Owner and controller are public projection fields; the chip names both whenever they differ.
  const otherController = owner && controller && controller.id !== owner.id ? controller : undefined;
  const you = (player: Player) => player.id === viewerId ? '（你）' : '';
  const description = [
    owner && `${owner.name}${you(owner)} 拥有${otherController ? `，${otherController.name}${you(otherController)} 操控` : ''}`,
    sameNameTag && `同名对象 #${sameNameTag}`,
    targeted && '合法目标，可点击选择，然后确认行动。',
  ].filter(Boolean).join('；');
  return <button type="button" data-card-instance={card.instanceId} data-card-owner={card.owner} data-card-controller={card.controller} data-control-differs={otherController ? true : undefined} data-card-actionable={actionable} data-card-targeted={targeted} data-card-exhausted={card.exhausted} data-card-color={archive && !card.faceDown ? color.replace(/色$/, '') : undefined} data-card-face-down={archive ? card.faceDown : undefined} data-attachment-count={attachmentCount || undefined} className={`hg-card hg-card-${palette}${compact ? ' hg-card-compact' : ''}${selected ? ' hg-selected' : ''}${card.exhausted ? ' hg-exhausted' : ''}${actionable ? ' hg-card-actionable' : ''}${targeted ? ' hg-card-targeted' : ''}`} onClick={onSelect} onMouseEnter={onPreview} onMouseLeave={onPreviewEnd} onFocus={onPreview} onBlur={onPreviewEnd} aria-pressed={selected} aria-description={description || undefined} aria-label={`查看${card.name}${card.exhausted ? '，已横置' : ''}${attachmentCount ? `，附属 ${attachmentCount} 张` : ''}`}>
    {owner && <span className={`hg-card-owner hg-team-${owner.team}`}><span className="hg-piece-avatar" aria-hidden="true">{owner.seat + 1}</span><span>{owner.name}{owner.id === viewerId ? ' · 你' : ''}</span>{sameNameTag && <span className="hg-instance-tag">#{sameNameTag}</span>}<em>{actionable ? '可行动' : card.faceDown && card.controller === viewerId ? '查看自牌' : '浏览'}</em>
      {otherController && <span className={`hg-card-controller hg-team-${otherController.team}`}>操控 · {otherController.name}{otherController.id === viewerId ? ' · 你' : ''}</span>}</span>}
    {!owner && sameNameTag && <span className="hg-instance-tag">#{sameNameTag}</span>}
    <CardContent card={card} definition={definition} compact={compact} attachmentCount={attachmentCount} />
  </button>;
}
