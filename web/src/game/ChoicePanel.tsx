import { useRef, useState } from 'react';
import { CardContent, visibleCard } from './CardTile';
import { useDialogFocus } from './useDialogFocus';
import type { Action, Card, CardDefinition, Choice, LegalAction } from './types';
import './choice-readability.css';

export function buildChoiceAction(base: LegalAction, choice: Choice, selected: string[], top: string[], bottom: string[], allocations: Record<string, number>): Action {
  if (choice.kind === 'damage') return { ...base, choiceId: choice.id, allocations };
  if (choice.kind === 'order' || choice.kind === 'investigation') return { ...base, choiceId: choice.id, top, bottom };
  return { ...base, choiceId: choice.id, selected };
}
function move(ids: string[], id: string, delta: number) {
  const index = ids.indexOf(id); const target = index + delta;
  if (index < 0 || target < 0 || target >= ids.length) return ids;
  const result = [...ids]; [result[index], result[target]] = [result[target], result[index]]; return result;
}
export function ChoicePanel({ choice, action, definitions, busy, onSubmit, onReadCard, viewerId, playerLabels, modal = false, modalActive = true, onPauseRoom }: {
  choice: Choice; action?: LegalAction; definitions: Map<string, CardDefinition>; busy: boolean; onSubmit: (action: Action) => void;
  onReadCard?: (card: Card) => void;
  viewerId?: string;
  playerLabels?: Record<string, string>;
  modal?: boolean;
  modalActive?: boolean;
  onPauseRoom?: () => void;
}) {
  const dialog = useRef<HTMLElement>(null);
  useDialogFocus(dialog, modal && modalActive);
  const [selected, setSelected] = useState<string[]>(choice.kind === 'region_return' ? choice.options.map(option => option.id) : []);
  const [top, setTop] = useState(choice.options.map(option => option.id));
  const [bottom, setBottom] = useState<string[]>([]);
  const [allocations, setAllocations] = useState<Record<string, number>>({});
  const ordering = choice.kind === 'order' || choice.kind === 'investigation';
  const damage = choice.kind === 'damage';
  const scrollHintId = `${choice.id}-scroll-hint`;
  const readCard = (card: Card) => onReadCard?.(viewerId ? visibleCard(card, viewerId) : card);
  const amount = choice.amount ?? 0;
  const canDecline = !!choice.allowDecline && !(damage && amount > 0);
  const assigned = Object.values(allocations).reduce((total, n) => total + n, 0);
  const min = choice.min ?? 1;
  const max = choice.max ?? choice.options.length;
  const zeroConfirmation = choice.kind === 'jc032_top_six' && min === 0 && max === 0 && choice.options.length === 0;
  const valid = damage ? assigned === amount : ordering ? top.length + bottom.length === choice.options.length
    : selected.length >= min && selected.length <= max && (selected.length > 0 || canDecline || zeroConfirmation);
  const choose = (id: string) => setSelected(ids => ids.includes(id) ? ids.filter(value => value !== id) : ids.length < max ? [...ids, id] : ids);
  const allocate = (id: string, requested: number) => setAllocations(current => {
    const other = Object.entries(current).reduce((total, [key, value]) => total + (key === id ? 0 : value), 0);
    return { ...current, [id]: Math.max(0, Math.min(amount - other, Math.floor(requested) || 0)) };
  });
  const tapDamage = (id: string) => {
    if (amount === 1) setAllocations(Object.fromEntries(choice.options.map(option => [option.id, option.id === id ? 1 : 0])));
    else allocate(id, (allocations[id] || 0) + 1);
  };
  const submit = (decline = false) => {
    if (!action || (decline && !canDecline)) return;
    onSubmit(decline ? { ...action, choiceId: choice.id, selected: [] }
      : buildChoiceAction(action, choice, selected, top, bottom, allocations));
  };
  const label = (id: string) => choice.options.find(option => option.id === id)?.label || id;
  const reorderList = (ids: string[], place: 'top' | 'bottom') => <ol className="hg-order-list">
    {ids.map((id, index) => <li key={id}><span className="hg-order-number">{index + 1}</span><strong>{label(id)}</strong><span className="hg-order-buttons">
      <button type="button" disabled={index === 0 || busy} onClick={() => place === 'top' ? setTop(move(top, id, -1)) : setBottom(move(bottom, id, -1))} aria-label={`${label(id)}上移`}>↑</button>
      <button type="button" disabled={index === ids.length - 1 || busy} onClick={() => place === 'top' ? setTop(move(top, id, 1)) : setBottom(move(bottom, id, 1))} aria-label={`${label(id)}下移`}>↓</button>
      <button type="button" disabled={busy} onClick={() => {
        if (place === 'top') { setTop(top.filter(value => value !== id)); setBottom([...bottom, id]); }
        else { setBottom(bottom.filter(value => value !== id)); setTop([...top, id]); }
      }}>{place === 'top' ? '置于底' : '置于顶'}</button>
    </span></li>)}
    {!ids.length && <li className="hg-empty">这里没有牌</li>}
  </ol>;

  return <section ref={dialog} className="hg-choice" data-choice-id={choice.id} role={modal ? 'dialog' : undefined} aria-modal={modal ? true : undefined} aria-label="待完成的选择" tabIndex={-1}>
    {onPauseRoom && <button type="button" className="hg-button hg-button-quiet" disabled={busy} onClick={onPauseRoom}>暂停并保存此桌</button>}
    <div className="hg-choice-heading">
    <div className="hg-section-title"><span className="hg-eyebrow">轮到你选择</span><span className="hg-choice-tag">{damage ? `尚余 ${amount - assigned} 点` : ordering ? '自上而下排列' : `已选 ${selected.length} / ${max}`}</span></div>
    <h2>{choice.title}</h2><p>{choice.description}</p>
    <p className="hg-choice-scroll-hint" id={scrollHintId}>{choice.kind === 'mulligan' ? '点选要替换的手牌，未选中的保留。' : '先查看选项，再确认你的选择。'} 选项区可上下滚动，确认按钮始终在下方。</p>
    </div>
    <div className="hg-choice-body" tabIndex={0} role="region" aria-label="选择选项，可滚动查看" aria-describedby={scrollHintId}>
    {!!choice.previewCards?.length && <div className="hg-choice-preview"><h3>你查看的牌库顶牌</h3><div className="hg-choice-options">{choice.previewCards.map(card => <div className="hg-choice-option-entry" key={card.instanceId}>
      <CardContent card={card} definition={definitions.get(card.cardId || '')} viewerId={viewerId} />
      {onReadCard && <button type="button" className="hg-choice-read hg-small-read" onClick={() => readCard(card)} aria-label={`放大阅读${card.name}`}>放大文字与图标 ↗</button>}
    </div>)}</div></div>}
    {ordering ? <div className="hg-order-columns"><div><h3>牌库顶 · 最上面先抓</h3>{reorderList(top, 'top')}</div><div><h3>牌库底 · 自上而下</h3>{reorderList(bottom, 'bottom')}</div></div>
      : damage ? <><div className="hg-damage-meter"><span>总计 <b>{amount}</b></span><span>已分配 <b>{assigned}</b></span><span>剩余 <b>{amount - assigned}</b></span><small>{amount === 1 ? '点选角色分配这一点伤害，可切换目标。' : '点选角色或使用加减按钮分配，每次一点。'}</small></div><div className="hg-damage-options">{choice.options.map((option, index) => <div key={option.id} className={`hg-damage-row ${(allocations[option.id] || 0) > 0 ? 'hg-damage-assigned' : ''}`}>
        <button type="button" className="hg-damage-target" data-choice-option={option.id} data-choice-target-number={index + 1} aria-label={`给${option.label}分配1点伤害`} aria-describedby={`${choice.id}-damage-${index}`} aria-pressed={(allocations[option.id] || 0) > 0} disabled={busy || (amount !== 1 && assigned >= amount)} onClick={() => tapDamage(option.id)}><span><strong>{option.label} <em className="hg-target-number">目标 {index + 1}</em></strong>{option.card && <small id={`${choice.id}-damage-${index}`}>{playerLabels?.[option.card.owner] || playerLabels?.[option.card.controller] || '公开角色'} 拥有 · 防御 {option.card.defense ?? '—'} · 已受伤 {option.card.damage ?? 0}{option.card.exhausted ? ' · 已横置' : ''}</small>}</span><b>+1</b></button>
        <div className="hg-damage-adjust"><button type="button" disabled={busy || !allocations[option.id]} aria-label={`给${option.label}减少1点伤害`} onClick={() => allocate(option.id, (allocations[option.id] || 0) - 1)}>−</button>
          <input type="number" inputMode="numeric" min={0} max={amount - assigned + (allocations[option.id] || 0)} value={allocations[option.id] || 0} disabled={busy} aria-label={`给${option.label}分配伤害`} onChange={event => allocate(option.id, Number(event.target.value))} />
          <button type="button" disabled={busy || assigned >= amount} aria-label={`给${option.label}增加1点伤害`} onClick={() => allocate(option.id, (allocations[option.id] || 0) + 1)}>+</button>
        </div>{option.card && onReadCard && <button type="button" className="hg-small-read" onClick={() => readCard(option.card!)}>放大阅读</button>}
      </div>)}</div></>
      : <><div className="hg-choice-options">{choice.options.map((option, index) => <div key={option.id} className="hg-choice-option-entry"><button type="button" data-choice-option={option.id} data-choice-target-number={option.card?.region === undefined ? undefined : index + 1} className={`hg-choice-option ${selected.includes(option.id) ? 'hg-selected' : ''}`} disabled={busy || (!selected.includes(option.id) && selected.length >= max)} onClick={() => choose(option.id)} aria-pressed={selected.includes(option.id)}>
        <span className="hg-choice-check">{selected.includes(option.id) ? selected.indexOf(option.id) + 1 : '+'}</span>
        {option.card?.region !== undefined && <><span className="hg-target-number" style={{ color: 'inherit', marginLeft: 0, alignSelf: 'flex-start' }}>目标 {index + 1}</span><small>{playerLabels?.[option.card.owner] || option.card.owner} 拥有{option.card.controller !== option.card.owner && ` · ${playerLabels?.[option.card.controller] || option.card.controller} 操控`} · 地区 {option.card.region + 1}</small></>}
        {option.card ? <CardContent card={option.card} definition={definitions.get(option.card.cardId || '')} viewerId={viewerId} /> : <strong>{option.label}</strong>}
      </button>{option.card && onReadCard && <button type="button" className="hg-choice-read hg-small-read" aria-label={`放大阅读${viewerId ? visibleCard(option.card, viewerId).name : option.card.name}`} onClick={() => readCard(option.card!)}>放大文字与图标 ↗</button>}</div>)}</div>
      {choice.kind === 'region_return' && selected.length > 1 && <div className="hg-return-order"><h3>置于牌库底的顺序</h3>{selected.map((id, index) => <div key={id}><span>{index + 1}. {label(id)}</span><button disabled={index === 0 || busy} onClick={() => setSelected(move(selected, id, -1))} aria-label={`${label(id)}上移`}>↑</button><button disabled={index === selected.length - 1 || busy} onClick={() => setSelected(move(selected, id, 1))} aria-label={`${label(id)}下移`}>↓</button></div>)}</div>}
    </>}
    </div>
    <div className="hg-choice-footer"><span>{damage ? `共需分配 ${amount} 点，已分配 ${assigned} 点` : ordering ? '确认后按此顺序放回牌库。' : `请选择 ${min === max ? min : `${min}–${max}`} 项${choice.allowDecline ? '，或跳过' : ''}。`}</span>
      <div>{canDecline && <button className="hg-button hg-button-quiet" disabled={busy || !action} onClick={() => submit(true)}>{choice.kind === 'mulligan' ? '保留全部手牌' : '跳过此选择'}</button>}
      <button className="hg-button hg-button-primary" disabled={busy || !valid || !action} onClick={() => submit()}>{busy ? '正在提交…' : '确认选择'}</button></div>
    </div>
  </section>;
}
