import type { Card, CardDefinition, Icons } from './types';

const emptyIcons: Icons = { investigation: 0, combat: 0, influence: 0 };
export function IconStrip({ icons, temporary = false }: { icons?: Icons; temporary?: boolean }) {
  const values = icons || emptyIcons;
  return <span className={`hg-icons ${temporary ? 'hg-icons-temporary' : ''}`} aria-label={`${temporary ? '先手图标' : '图标'}：调查${values.investigation}，战斗${values.combat}，势力${values.influence}`}>
    <span title="调查">◈ <b>{values.investigation}</b></span>
    <span title="战斗">⚔ <b>{values.combat}</b></span>
    <span title="势力">⚑ <b>{values.influence}</b></span>
  </span>;
}

export function CardContent({ card, definition, compact = false }: { card: Card; definition?: CardDefinition; compact?: boolean }) {
  const hidden = !card.cardId && card.faceDown;
  const asset = card.kind === 'asset';
  const concealedCompact = compact && card.kind === 'hidden';
  const permanent = definition?.icons?.permanent || definition?.permanentIcons || card.icons;
  const temporary = definition?.icons?.temporary || definition?.temporaryIcons;
  const hasTemporary = temporary && Object.values(temporary).some(n => n > 0);
  const type = hidden ? '身份未公开' : definition?.type || definition?.subtypes?.join(' · ') || ({ character: '角色', event: '事件', spell: '咒术', region: '地区', asset: '资产', hidden: '暗藏角色' }[card.kind] || '卡牌');
  return <>
    <span className="hg-card-top"><span className="hg-card-kind">{type}</span>{!hidden && !asset && <span className="hg-cost" title="费用">{card.cost ?? definition?.cost ?? 0}</span>}</span>
    <strong className="hg-card-name">{hidden ? '暗藏者' : card.name}</strong>
    {!compact && <span className="hg-card-art" aria-hidden="true"><span>{hidden ? '？' : card.kind === 'event' ? '✧' : card.kind === 'region' ? '⌖' : '◈'}</span><i /></span>}
    {hidden ? <span className="hg-card-text">{concealedCompact ? '暗藏者 · 不受角色伤害' : '暗藏身份，等待揭示。'}</span> : <>
      <span className="hg-card-affiliation">{asset ? `${card.color || '无派系'}${card.magic ? ` · ${card.magic}` : ''}` : <>{definition?.loyaltyText || (definition?.loyalty?.length ? `忠诚 ${definition.loyalty.join(' / ')}` : '无忠诚要求')}{(card.magic || definition?.magic) && ` · ${card.magic || definition?.magic}`}</>}</span>
      {!asset && <IconStrip icons={compact ? card.icons || permanent : permanent} />}
      {!compact && hasTemporary && <span className="hg-temporary-label">先手 <IconStrip icons={temporary} temporary /></span>}
      {!compact && <span className="hg-card-text">{asset ? '未横置时可提供 1 费用，并提供所示派系与魔法忠诚。' : card.text || definition?.text || '此牌没有额外能力。'}</span>}
      <span className="hg-card-footer">
        {!concealedCompact && (card.defense ?? definition?.defense) !== undefined && <span>防御 {card.defense ?? definition?.defense}</span>}
        {!concealedCompact && !!card.damage && <span className="hg-hurt">伤害 {card.damage}</span>}
        {!concealedCompact && !!card.wounds && <span className="hg-hurt">创伤 {card.wounds}</span>}
        {!concealedCompact && !!card.shield && <span>护盾 {card.shield}</span>}
        {card.exhausted && <span className="hg-exhausted-label">已横置</span>}
        {card.faceDown && <span>{concealedCompact ? '暗藏者 · 不受角色伤害' : '暗藏'}</span>}
      </span>
    </>}
  </>;
}

export function CardTile({ card, definition, selected, onSelect, compact = false }: {
  card: Card; definition?: CardDefinition; selected?: boolean; onSelect?: () => void; compact?: boolean;
}) {
  const color = card.color || definition?.color || '';
  const palette = color.includes('黄') ? 'gold' : color.includes('红') ? 'red' : color.includes('蓝') ? 'blue' : color.includes('绿') ? 'green' : color.includes('紫') ? 'violet' : 'neutral';
  return <button type="button" data-card-instance={card.instanceId} className={`hg-card hg-card-${palette}${compact ? ' hg-card-compact' : ''}${selected ? ' hg-selected' : ''}${card.exhausted ? ' hg-exhausted' : ''}`} onClick={onSelect} aria-pressed={selected} aria-label={`查看${card.name}${card.exhausted ? '，已横置' : ''}`}>
    <CardContent card={card} definition={definition} compact={compact} />
  </button>;
}
