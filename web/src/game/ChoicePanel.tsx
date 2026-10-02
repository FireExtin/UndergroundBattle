import { useState } from 'react';
import { CardContent } from './CardTile';
import type { Action, CardDefinition, Choice, LegalAction } from './types';

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
export function ChoicePanel({ choice, action, definitions, busy, onSubmit }: {
  choice: Choice; action?: LegalAction; definitions: Map<string, CardDefinition>; busy: boolean; onSubmit: (action: Action) => void;
}) {
  const [selected, setSelected] = useState<string[]>(choice.kind === 'region_return' ? choice.options.map(option => option.id) : []);
  const [top, setTop] = useState(choice.options.map(option => option.id));
  const [bottom, setBottom] = useState<string[]>([]);
  const [allocations, setAllocations] = useState<Record<string, number>>({});
  const ordering = choice.kind === 'order' || choice.kind === 'investigation';
  const damage = choice.kind === 'damage';
  const amount = choice.amount ?? 0;
  const canDecline = !!choice.allowDecline && !(damage && amount > 0);
  const assigned = Object.values(allocations).reduce((total, n) => total + n, 0);
  const min = choice.min ?? 1;
  const max = choice.max ?? choice.options.length;
  const valid = damage ? assigned === amount : ordering ? top.length + bottom.length === choice.options.length
    : selected.length >= min && selected.length <= max && (selected.length > 0 || canDecline);
  const choose = (id: string) => setSelected(ids => ids.includes(id) ? ids.filter(value => value !== id) : ids.length < max ? [...ids, id] : ids);
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

  return <section className="hg-choice" data-choice-id={choice.id} aria-label="待完成的选择">
    <div className="hg-section-title"><span className="hg-eyebrow">轮到你选择</span><span className="hg-choice-tag">{damage ? `尚余 ${amount - assigned} 点` : ordering ? '自上而下排列' : `已选 ${selected.length} / ${max}`}</span></div>
    <h2>{choice.title}</h2><p>{choice.description}</p>
    {ordering ? <div className="hg-order-columns"><div><h3>牌库顶 · 最上面先抓</h3>{reorderList(top, 'top')}</div><div><h3>牌库底 · 自上而下</h3>{reorderList(bottom, 'bottom')}</div></div>
      : damage ? <div className="hg-damage-options">{choice.options.map(option => <label key={option.id}><span><strong>{option.label}</strong>{option.card && <small>防御 {option.card.defense ?? '—'} · 已受伤 {option.card.damage ?? 0}</small>}</span>
        <input type="number" min={0} max={amount - assigned + (allocations[option.id] || 0)} value={allocations[option.id] || 0} disabled={busy} aria-label={`给${option.label}分配伤害`} onChange={e => {
          const available = amount - assigned + (allocations[option.id] || 0);
          const value = Math.max(0, Math.min(available, Math.floor(Number(e.target.value) || 0)));
          setAllocations({ ...allocations, [option.id]: value });
        }} />
      </label>)}</div>
      : <><div className="hg-choice-options">{choice.options.map(option => <button type="button" key={option.id} data-choice-option={option.id} className={`hg-choice-option ${selected.includes(option.id) ? 'hg-selected' : ''}`} disabled={busy || (!selected.includes(option.id) && selected.length >= max)} onClick={() => choose(option.id)} aria-pressed={selected.includes(option.id)}>
        <span className="hg-choice-check">{selected.includes(option.id) ? selected.indexOf(option.id) + 1 : '+'}</span>
        {option.card ? <CardContent card={option.card} definition={definitions.get(option.card.cardId || '')} /> : <strong>{option.label}</strong>}
      </button>)}</div>
      {choice.kind === 'region_return' && selected.length > 1 && <div className="hg-return-order"><h3>置于牌库底的顺序</h3>{selected.map((id, index) => <div key={id}><span>{index + 1}. {label(id)}</span><button disabled={index === 0 || busy} onClick={() => setSelected(move(selected, id, -1))} aria-label={`${label(id)}上移`}>↑</button><button disabled={index === selected.length - 1 || busy} onClick={() => setSelected(move(selected, id, 1))} aria-label={`${label(id)}下移`}>↓</button></div>)}</div>}
    </>}
    <div className="hg-choice-footer"><span>{damage ? `共需分配 ${amount} 点，已分配 ${assigned} 点` : ordering ? '确认后按此顺序放回牌库。' : `请选择 ${min === max ? min : `${min}–${max}`} 项${choice.allowDecline ? '，或跳过' : ''}。`}</span>
      <div>{canDecline && <button className="hg-button hg-button-quiet" disabled={busy || !action} onClick={() => submit(true)}>{choice.kind === 'mulligan' ? '保留全部手牌' : '跳过此选择'}</button>}
      <button className="hg-button hg-button-primary" disabled={busy || !valid || !action} onClick={() => submit()}>{busy ? '正在提交…' : '确认选择'}</button></div>
    </div>
  </section>;
}
