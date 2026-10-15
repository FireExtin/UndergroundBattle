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
  const optionsRegion = useRef<HTMLDivElement>(null);
  const searchBox = useRef<HTMLInputElement>(null);
  useDialogFocus(dialog, modal && modalActive);
  const [selected, setSelected] = useState<string[]>(choice.kind === 'region_return' ? choice.options.map(option => option.id) : []);
  const [search, setSearch] = useState('');
  const [selectedOnly, setSelectedOnly] = useState(false);
  const [{ top, bottom }, setOrder] = useState<{ top: string[]; bottom: string[] }>(() => ({ top: choice.options.map(option => option.id), bottom: [] }));
  const [allocations, setAllocations] = useState<Record<string, number>>({});
  const ordering = choice.kind === 'order' || choice.kind === 'investigation';
  const damage = choice.kind === 'damage';
  const filterable = !ordering && !damage && choice.options.length > 6;
  const query = search.trim().toLocaleLowerCase();
  const visibleOptions = choice.options.map((option, index) => ({ option, index })).filter(({ option }) => {
    if (!filterable) return true;
    if (selectedOnly && !selected.includes(option.id)) return false;
    if (!query) return true;
    const card = option.card && (viewerId ? visibleCard(option.card, viewerId) : option.card);
    return (card ? `${card.name} ${card.cardId || ''}` : option.label).toLocaleLowerCase().includes(query);
  });
  const visibleIds = new Set(visibleOptions.map(({ option }) => option.id));
  const hiddenSelected = selected.filter(id => !visibleIds.has(id)).length;
  const scrollHintId = `${choice.id}-scroll-hint`;
  const selectionHintId = `${choice.id}-selection-hint`;
  const readCard = (card: Card) => onReadCard?.(viewerId ? visibleCard(card, viewerId) : card);
  const amount = choice.amount ?? 0;
  const canDecline = !!choice.allowDecline && !(damage && amount > 0);
  const assigned = Object.values(allocations).reduce((total, n) => total + n, 0);
  const min = choice.min ?? 1;
  const max = choice.max ?? choice.options.length;
  const jz50Search = choice.kind === 'jz50_death_search' && min === 0 && max === 1 && !choice.allowDecline;
  const entrySearchZero = !choice.allowDecline && min === 0 && (
    (choice.kind === 'bq104_employee_search' && max === 0 && choice.options.length === 0)
    || (choice.kind === 'xq48_passer_search' && max <= 3));
  const deckSealZero = ['xq44_dream_seal_search', 'jz02_space_seal_search'].includes(choice.kind)
    && !choice.allowDecline && min === 0 && max === 0 && choice.options.length === 0;
  const zeroConfirmation = (choice.kind === 'jc032_top_six' && min === 0 && max === 0 && choice.options.length === 0) || jz50Search || entrySearchZero || deckSealZero;
  const valid = damage ? assigned === amount : ordering ? top.length + bottom.length === choice.options.length
    : selected.length >= min && selected.length <= max && (selected.length > 0 || canDecline || zeroConfirmation);
  const selectionFull = selected.length >= max;
  const selectionHint = max === 1
    ? '再次点击已选项可取消；点击其他项可直接替换。'
    : selectionFull ? '已达选择上限，先取消一项可继续换选。'
      : `还可再选 ${max - selected.length} 项；再次点击已选项可取消。`;
  const choose = (id: string) => {
    setSelected(ids => ids.includes(id) ? ids.filter(value => value !== id)
      : max === 1 ? [id] : ids.length < max ? [...ids, id] : ids);
    if (selectedOnly && selected.includes(id)) searchBox.current?.focus();
  };
  const resetFilters = () => {
    setSearch('');
    setSelectedOnly(false);
    if (optionsRegion.current) optionsRegion.current.scrollTop = 0;
    searchBox.current?.focus();
  };
  const allocate = (id: string, requested: number | ((value: number) => number)) => setAllocations(current => {
    const other = Object.entries(current).reduce((total, [key, value]) => total + (key === id ? 0 : value), 0);
    const value = typeof requested === 'function' ? requested(current[id] || 0) : requested;
    return { ...current, [id]: Math.max(0, Math.min(amount - other, Math.floor(value) || 0)) };
  });
  const tapDamage = (id: string) => {
    if (amount === 1) setAllocations(Object.fromEntries(choice.options.map(option => [option.id, option.id === id ? 1 : 0])));
    else allocate(id, value => value + 1);
  };
  const submit = (decline = false) => {
    if (!action || (decline && !canDecline)) return;
    onSubmit(decline ? { ...action, choiceId: choice.id, selected: [] }
      : buildChoiceAction(action, choice, selected, top, bottom, allocations));
  };
  const label = (id: string) => choice.options.find(option => option.id === id)?.label || id;
  const reorderList = (ids: string[], place: 'top' | 'bottom') => <ol className="hg-order-list">
    {ids.map((id, index) => <li key={id}><span className="hg-order-number">{index + 1}</span><strong>{label(id)}</strong><span className="hg-order-buttons">
      <button type="button" disabled={index === 0 || busy} onClick={() => setOrder(current => ({ ...current, [place]: move(current[place], id, -1) }))} aria-label={`${label(id)}上移`}>↑</button>
      <button type="button" disabled={index === ids.length - 1 || busy} onClick={() => setOrder(current => ({ ...current, [place]: move(current[place], id, 1) }))} aria-label={`${label(id)}下移`}>↓</button>
      <button type="button" disabled={busy} onClick={() => setOrder(current => {
        if (!current[place].includes(id)) return current;
        const destination = place === 'top' ? 'bottom' : 'top';
        return { ...current, [place]: current[place].filter(value => value !== id), [destination]: [...current[destination], id] };
      })}>{place === 'top' ? '置于底' : '置于顶'}</button>
    </span></li>)}
    {!ids.length && <li className="hg-empty">这里没有牌</li>}
  </ol>;

  return <section ref={dialog} className="hg-choice" data-choice-id={choice.id} role={modal ? 'dialog' : undefined} aria-modal={modal ? true : undefined} aria-label="待完成的选择" tabIndex={-1}>
    {onPauseRoom && <button type="button" className="hg-button hg-button-quiet" disabled={busy} onClick={onPauseRoom}>暂停并保存此桌</button>}
    <div className="hg-choice-heading">
    <div className="hg-section-title"><span className="hg-eyebrow">轮到你选择</span><span className="hg-choice-tag" role="status" aria-atomic="true">{damage ? `尚余 ${amount - assigned} 点` : ordering ? '自上而下排列' : `已选 ${selected.length} / ${max}`}</span></div>
    <h2>{choice.title}</h2><p>{choice.description}</p>
    <p className="hg-choice-scroll-hint" id={scrollHintId}>{choice.kind === 'mulligan' ? '点选要替换的手牌，未选中的保留。' : '先查看选项，再确认你的选择。'} 选项区可上下滚动，确认按钮始终在下方。</p>
    </div>
    {filterable && <div className="hg-choice-filters">
      <label className="hg-field">检索选项<input ref={searchBox} type="search" value={search} placeholder="名称或卡牌编号" disabled={busy} onChange={event => {
        setSearch(event.target.value);
        if (optionsRegion.current) optionsRegion.current.scrollTop = 0;
      }} /></label>
      <label className="hg-choice-selected-filter"><input type="checkbox" checked={selectedOnly} disabled={busy} onChange={event => {
        setSelectedOnly(event.target.checked);
        if (optionsRegion.current) optionsRegion.current.scrollTop = 0;
      }} />只看已选</label>
      <button type="button" className="hg-button hg-button-quiet" disabled={busy || (!search && !selectedOnly)} onClick={resetFilters}>重置筛选</button>
      <p className="hg-choice-filter-summary" role="status" aria-label="选项筛选结果" aria-atomic="true">显示 {visibleOptions.length} / {choice.options.length} 项{hiddenSelected > 0 && <strong>另有 {hiddenSelected} 项已选不在当前结果中</strong>}</p>
    </div>}
    <div ref={optionsRegion} className="hg-choice-body" tabIndex={0} role="region" aria-label="选择选项，可滚动查看" aria-describedby={scrollHintId}>
    {!!choice.previewCards?.length && <div className="hg-choice-preview"><h3>{choice.kind === 'bq028-hand-inspect' ? '你检视的手牌' : '你查看的牌库顶牌'}</h3><div className="hg-choice-options">{choice.previewCards.map(card => <div className="hg-choice-option-entry" key={card.instanceId}>
      <CardContent card={card} definition={definitions.get(card.cardId || '')} viewerId={viewerId} />
      {onReadCard && <button type="button" className="hg-choice-read hg-small-read" onClick={() => readCard(card)} aria-label={`放大阅读${viewerId ? visibleCard(card, viewerId).name : card.name}`}>放大文字与图标 ↗</button>}
    </div>)}</div></div>}
    {ordering ? <div className="hg-order-columns"><div><h3>牌库顶 · 最上面先抓</h3>{reorderList(top, 'top')}</div><div><h3>牌库底 · 自上而下</h3>{reorderList(bottom, 'bottom')}</div></div>
      : damage ? <><div className="hg-damage-meter"><span>总计 <b>{amount}</b></span><span>已分配 <b>{assigned}</b></span><span>剩余 <b>{amount - assigned}</b></span><small>{amount === 1 ? '点选角色分配这一点伤害，可切换目标。' : '点选角色或使用加减按钮分配，每次一点。'}</small></div><div className="hg-damage-options">{choice.options.map((option, index) => <div key={option.id} className={`hg-damage-row ${(allocations[option.id] || 0) > 0 ? 'hg-damage-assigned' : ''}`}>
        <button type="button" className="hg-damage-target" data-choice-option={option.id} data-choice-target-number={index + 1} aria-label={`给${option.label}分配1点伤害`} aria-describedby={`${choice.id}-damage-${index}`} aria-pressed={(allocations[option.id] || 0) > 0} disabled={busy || (amount !== 1 && assigned >= amount)} onClick={() => tapDamage(option.id)}><span><strong>{option.label} <em className="hg-target-number">目标 {index + 1}</em></strong>{option.card && <small id={`${choice.id}-damage-${index}`}>{playerLabels?.[option.card.owner] || playerLabels?.[option.card.controller] || '公开角色'} 拥有 · 防御 {option.card.defense ?? '—'} · 已受伤 {option.card.damage ?? 0}{option.card.exhausted ? ' · 已横置' : ''}</small>}</span><b>+1</b></button>
        <div className="hg-damage-adjust"><button type="button" disabled={busy || !allocations[option.id]} aria-label={`给${option.label}减少1点伤害`} onClick={() => allocate(option.id, value => value - 1)}>−</button>
          <input type="number" inputMode="numeric" min={0} max={amount - assigned + (allocations[option.id] || 0)} value={allocations[option.id] || 0} disabled={busy} aria-label={`给${option.label}分配伤害`} onChange={event => allocate(option.id, Number(event.target.value))} />
          <button type="button" disabled={busy || assigned >= amount} aria-label={`给${option.label}增加1点伤害`} onClick={() => allocate(option.id, value => value + 1)}>+</button>
        </div>{option.card && onReadCard && <button type="button" className="hg-small-read" onClick={() => readCard(option.card!)}>放大阅读</button>}
      </div>)}</div></>
      : <><div className="hg-choice-options">{visibleOptions.map(({ option, index }) => {
        const selectedIndex = selected.indexOf(option.id);
        const isSelected = selectedIndex >= 0;
        const atLimit = !isSelected && max !== 1 && selectionFull;
        return <div key={option.id} className="hg-choice-option-entry"><button type="button" data-choice-option={option.id} data-choice-target-number={option.card?.region === undefined ? undefined : index + 1} className={`hg-choice-option ${isSelected ? 'hg-selected' : ''}`} disabled={busy || atLimit} onClick={() => choose(option.id)} aria-pressed={isSelected} aria-describedby={max > 0 ? selectionHintId : undefined}>
        <span className="hg-choice-selection-state" aria-hidden="true"><span className="hg-choice-check">{isSelected ? selectedIndex + 1 : '+'}</span><span>{isSelected ? '已选 · 再点取消' : atLimit ? '已达上限' : max === 1 && selected.length > 0 ? '换选此项' : '点击选择'}</span></span>
        {option.card?.region !== undefined && <><span className="hg-target-number" style={{ color: 'inherit', marginLeft: 0, alignSelf: 'flex-start' }}>目标 {index + 1}</span><small>{playerLabels?.[option.card.owner] || option.card.owner} 拥有{option.card.controller !== option.card.owner && ` · ${playerLabels?.[option.card.controller] || option.card.controller} 操控`} · 地区 {option.card.region + 1}</small></>}
        {option.card ? <CardContent card={option.card} definition={definitions.get(option.card.cardId || '')} viewerId={viewerId} /> : <strong className="hg-choice-option-label">{option.label}</strong>}
      </button>{option.card && onReadCard && <button type="button" className="hg-choice-read hg-small-read" aria-label={`放大阅读${viewerId ? visibleCard(option.card, viewerId).name : option.card.name}`} onClick={() => readCard(option.card!)}>放大文字与图标 ↗</button>}</div>;
      })}</div>
      {filterable && visibleOptions.length === 0 && <p className="hg-choice-empty">没有符合筛选的选项。{selected.length > 0 ? `已选的 ${selected.length} 项仍会保留。` : '可重置筛选后继续选择。'}</p>}
      {choice.kind === 'region_return' && selected.length > 1 && <div className="hg-return-order"><h3>置于牌库底的顺序</h3>{selected.map((id, index) => <div key={id}><span>{index + 1}. {label(id)}</span><button disabled={index === 0 || busy} onClick={() => setSelected(current => move(current, id, -1))} aria-label={`${label(id)}上移`}>↑</button><button disabled={index === selected.length - 1 || busy} onClick={() => setSelected(current => move(current, id, 1))} aria-label={`${label(id)}下移`}>↓</button></div>)}</div>}
    </>}
    </div>
    <div className="hg-choice-footer"><span className="hg-choice-summary">{damage ? `共需分配 ${amount} 点，已分配 ${assigned} 点` : ordering ? '确认后按此顺序放回牌库。' : `请选择 ${min === max ? min : `${min}–${max}`} 项${choice.allowDecline ? '，或跳过' : ''}。`}
      {!damage && !ordering && max > 0 && <small id={selectionHintId}>{selectionHint}</small>}</span>
      <div>{canDecline && <button className="hg-button hg-button-quiet" disabled={busy || !action} onClick={() => submit(true)}>{choice.kind === 'mulligan' ? '保留全部手牌' : choice.kind === 'bq028-hand-inspect' ? '不弃牌并确认' : '跳过此选择'}</button>}
      <button className="hg-button hg-button-primary" disabled={busy || !valid || !action} onClick={() => submit()}>{busy ? '正在提交…' : (jz50Search || entrySearchZero || deckSealZero) && selected.length === 0 ? '不取牌并洗牌' : '确认选择'}</button></div>
    </div>
  </section>;
}
