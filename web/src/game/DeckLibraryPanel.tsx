import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { factions, neutral } from './factions';
import { createDeckDraft, isDeckDraftDeleted, isEditableDeckCard, localDeckStorage, publicDeckDraft, readDeckLibrary, removeDeckDraft, revalidateDraftVersions, saveDeckDraft, selectableSocieties, setDraftCardCount, societyColorCount, validateDeckDraft } from './deckLibrary';
import type { DeckCardDefinition, DeckCatalog, DeckDraft, DeckStorage } from './deckLibrary';
import { DeckCardFace } from './DeckCardFace';
import { DeckCardPreview } from './DeckCardPreview';
import { useDialogFocus } from './useDialogFocus';
import './deck-library.css';

const kinds: Record<string, string> = { character: '角色', spell: '事务', event: '事务（旧目录）', attachment: '附属', region: '地区', society: '秘社' };
function factionId(card: DeckCardDefinition) {
  const color = card.color?.trim().replace(/色$/, '');
  if (color === '中立' || color === '褐') return 'neutral';
  return factions.find(faction => faction.color === color)?.id || 'unknown';
}
function factionName(card: DeckCardDefinition) {
  const id = factionId(card);
  return id === 'neutral' ? neutral.name : factions.find(faction => faction.id === id)?.name || '颜色未标';
}
function copyLabel(card?: DeckCardDefinition) {
  return !card || card.deckCopyLimit === undefined ? '副本限制待确认' : card.deckCopyLimit === null ? '同名数量不限' : `同名最多 ${card.deckCopyLimit} 张`;
}

export function DeckLibrary({ catalog, disabled = false, onSelectDraft, selectedDraftId, storage }: {
  catalog: DeckCatalog;
  disabled?: boolean;
  onSelectDraft?: (draft: DeckDraft) => void;
  selectedDraftId?: string;
  storage?: DeckStorage | null;
}) {
  const [initial] = useState(() => readDeckLibrary(storage === undefined ? localDeckStorage() : storage));
  const [saved, setSaved] = useState(initial.drafts);
  const [draft, setDraft] = useState<DeckDraft>(() => initial.drafts[0] ? publicDeckDraft(initial.drafts[0]) : createDeckDraft(catalog));
  const [storageWarning, setStorageWarning] = useState(initial.warning);
  const [deletedDraft, setDeletedDraft] = useState(false);
  const [notice, setNotice] = useState('');
  const [dirty, setDirty] = useState(false);
  const [presetId, setPresetId] = useState(catalog.decks[0]?.id || '');
  const [search, setSearch] = useState('');
  const [faction, setFaction] = useState('all');
  const [kind, setKind] = useState('all');
  const [implementation, setImplementation] = useState('implemented');
  const [cost, setCost] = useState('all');
  const [selectedOnly, setSelectedOnly] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const workspace = useRef<HTMLElement>(null);
  const workspaceButton = useRef<HTMLButtonElement>(null);
  const previewOpener = useRef<HTMLButtonElement>(null);
  const searchBox = useRef<HTMLInputElement>(null);
  const openPreview = (id: string, opener: HTMLButtonElement) => { previewOpener.current = opener; setPreviewId(id); };
  const closeWorkspace = () => { setExpanded(false); workspaceButton.current?.focus(); };
  useDialogFocus(workspace, expanded && previewId === null, closeWorkspace);
  useEffect(() => {
    if (!expanded) return;
    const previous = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = previous; };
  }, [expanded]);
  useLayoutEffect(() => {
    // Removing the last copy under “only selected” can remove the preview opener.
    if (previewId === null && previewOpener.current && !previewOpener.current.isConnected) searchBox.current?.focus();
  }, [previewId]);
  const validation = validateDeckDraft(draft, catalog);
  const societies = selectableSocieties(catalog);
  const society = societies.find(card => card.id === draft.societyId);
  const definitions = useMemo(() => new Map(catalog.cards.map(card => [card.id, card])), [catalog.cards]);
  const counts = useMemo(() => new Map(draft.cards.map(entry => [entry.cardId, entry.count])), [draft.cards]);
  const costs = [...new Set(catalog.cards.filter(isEditableDeckCard).map(card => card.cost))].sort((a, b) => a - b);
  const preview = previewId ? definitions.get(previewId) : undefined;
  const choices = catalog.cards.filter(card => {
    const query = search.trim().toLocaleLowerCase();
    return (!query || `${card.name} ${card.id} ${card.text}`.toLocaleLowerCase().includes(query)) && (faction === 'all' || factionId(card) === faction) && (kind === 'all' || kind === card.kind) && (cost === 'all' || String(card.cost) === cost) && (!selectedOnly || (counts.get(card.id) || 0) > 0) && (implementation === 'all' || (implementation === 'implemented' ? isEditableDeckCard(card) : !isEditableDeckCard(card)));
  });
  const versionsChanged = draft.rulesVersion !== catalog.rulesVersion || draft.cardPoolVersion !== catalog.cardPoolVersion || draft.engineVersion !== catalog.engineVersion;
  const storageTarget = storage === undefined ? localDeckStorage() : storage;
  const edit = (next: DeckDraft | ((current: DeckDraft) => DeckDraft)) => { setDraft(next); setDirty(true); setNotice(''); };
  const changeCount = (cardId: string, delta: number) => edit(current => setDraftCardCount(current, cardId, Math.max(0, (current.cards.find(entry => entry.cardId === cardId)?.count || 0) + delta)));
  const resetFilters = () => { setSearch(''); setFaction('all'); setKind('all'); setCost('all'); setSelectedOnly(false); setImplementation('implemented'); };
  const replace = (next: DeckDraft, unsaved = false) => { setDraft(next); setDirty(unsaved); setDeletedDraft(false); setNotice(''); };
  const persist = (candidate: DeckDraft = draft): DeckDraft | null => {
    const next = { ...publicDeckDraft(candidate), name: candidate.name.trim(), description: candidate.description.trim(), updatedAt: new Date().toISOString() };
    const error = saveDeckDraft(next, storageTarget);
    if (error) { setStorageWarning(error); setDeletedDraft(isDeckDraftDeleted(draft.id, storageTarget)); setNotice(''); return null; }
    const latest = readDeckLibrary(storageTarget);
    setSaved(latest.drafts); setDraft(next); setDirty(false); setDeletedDraft(false); setStorageWarning(latest.warning); setNotice('牌组已保存在此浏览器。');
    return next;
  };
  const remove = (id: string) => {
    const error = removeDeckDraft(id, storageTarget);
    if (error) { setStorageWarning(error); return; }
    const latest = readDeckLibrary(storageTarget); const list = latest.drafts;
    setSaved(list); setStorageWarning(latest.warning);
    if (draft.id === id) replace(list[0] ? publicDeckDraft(list[0]) : createDeckDraft(catalog));
    setNotice('已删除此浏览器中的草稿；已入席牌桌使用的牌组不会因此改变。');
  };
  return <section ref={workspace} tabIndex={expanded ? -1 : undefined} className={`hg-deck-library${expanded ? ' hg-library-expanded' : ''}`} role={expanded ? 'dialog' : undefined} aria-modal={expanded ? true : undefined} aria-labelledby="hg-library-title">
    <div className="hg-section-title"><span className="hg-eyebrow">03 / 我的牌组</span><div className="hg-library-workspace-actions"><span className="hg-pool-badge">本地浏览器保存</span><button ref={workspaceButton} type="button" className="hg-button hg-button-quiet" onClick={() => expanded ? closeWorkspace() : setExpanded(true)}>{expanded ? '返回大厅组卡' : '展开组卡工作台'}</button></div></div>
    <h2 id="hg-library-title">卡面组卡工作台</h2>
    <p className="hg-muted">任意派系可以混合；通常至少 50 张，同名最多 3 张，印刷例外以目录标注为准。只可添加当前已实现的玩家卡；可选秘社独立保存，不计入玩家卡张数。</p>
    <p className="hg-muted">当前可编辑 {catalog.cards.filter(isEditableDeckCard).length} 种玩家卡。这里只展示当前开放目录，并非完整八派系卡池。草稿保存在此浏览器，不会跨设备同步。</p>
    {storageWarning && <p className="hg-library-warning" role="alert">{storageWarning}</p>}
    <div className="hg-library-toolbar">
      <button type="button" className="hg-button hg-button-quiet" disabled={disabled} onClick={() => replace(createDeckDraft(catalog), true)}>新建空白牌组</button>
      <label className="hg-field">预组模板<select value={presetId} onChange={event => setPresetId(event.target.value)} disabled={disabled || !catalog.decks.length}>{catalog.decks.map(deck => <option key={deck.id} value={deck.id}>{deck.name}</option>)}</select></label>
      <button type="button" className="hg-button hg-button-quiet" disabled={disabled || !catalog.decks.some(deck => deck.id === presetId)} onClick={() => replace(createDeckDraft(catalog, catalog.decks.find(deck => deck.id === presetId)), true)}>复制为草稿</button>
    </div>
    {saved.length > 0 && <ul className="hg-library-saved" aria-label="已保存牌组">{saved.map(item => <li key={item.id}>
      <button type="button" aria-pressed={draft.id === item.id} disabled={disabled} onClick={() => replace(publicDeckDraft(item))}><strong>{item.name || '未命名牌组'}</strong><span>{item.cards.reduce((sum, entry) => sum + entry.count, 0)} 张 · {validateDeckDraft(item, catalog).valid ? '构筑校验通过' : '待修正'}{selectedDraftId === item.id ? ' · 已选择入席' : ''}</span></button>
      <button type="button" className="hg-button hg-button-quiet" aria-label={`删除牌组 ${item.name}`} disabled={disabled} onClick={() => remove(item.id)}>删除</button>
    </li>)}</ul>}
    <div className="hg-library-editor">
      <div className="hg-library-edit-panel">
        <div className="hg-library-deck-status">
          <div className="hg-library-total" role="status"><strong>{validation.total} 张</strong><span>{validation.valid ? '构筑校验通过' : '暂不可开局'}{dirty ? ' · 有未保存修改' : ''}</span></div>
          <div className="hg-library-save-actions">
            <button type="button" className="hg-button hg-button-quiet" disabled={disabled || !draft.name.trim()} onClick={() => persist()}>保存牌组</button>
            {deletedDraft && <button type="button" className="hg-button hg-button-quiet" disabled={disabled || !draft.name.trim()} onClick={() => persist({ ...publicDeckDraft(draft), id: createDeckDraft(catalog).id })}>另存为新草稿</button>}
            <button type="button" className="hg-button hg-button-primary" disabled={disabled || !validation.valid || !onSelectDraft} onClick={() => { const next = persist(); if (next) onSelectDraft?.(publicDeckDraft(next)); }}>保存并选择此牌组</button>
          </div>
          {notice && <p className="hg-library-notice" role="status">{notice}</p>}
        </div>
        <label className="hg-field">牌组名称<input value={draft.name} onChange={event => edit({ ...draft, name: event.target.value })} maxLength={80} disabled={disabled} /></label>
        <label className="hg-field">牌组说明<textarea value={draft.description} onChange={event => edit({ ...draft, description: event.target.value })} maxLength={500} rows={2} disabled={disabled} /></label>
        <section className="hg-library-society" aria-label="可选秘社">
          <label className="hg-field">秘社（可选）<select value={draft.societyId || ''} disabled={disabled || !societies.length} onChange={event => edit({ ...draft, societyId: event.target.value || null })}>
            <option value="">不选择秘社</option>
            {draft.societyId && !society && <option value={draft.societyId}>已保存选择 · {draft.societyId}（当前未开放）</option>}
            {societies.map(item => <option key={item.id} value={item.id}>{item.name}</option>)}
          </select></label>
          {draft.societyId !== null && <button type="button" className="hg-button hg-button-quiet" disabled={disabled} onClick={() => edit({ ...draft, societyId: null })}>清除秘社选择</button>}
          <p>{!societies.length && '当前目录尚未开放秘社。'}{draft.societyId === null ? '不选择秘社时，起手 6 张。' : society ? `选定秘社起手 ${Number.isSafeInteger(society.startingHand) && society.startingHand >= 0 ? society.startingHand : '待确认'} 张。` : '已保存的秘社当前不可用于开局。'}</p>
          {society && <>
            {society.subtitle && <strong>{society.subtitle}</strong>}
            <p className="hg-library-card-text">{society.text}</p>
            <strong>构筑要求</strong>
            {!Array.isArray(society.deckConstraints) ? <p>构筑要求待确认。</p> : society.deckConstraints.length ? <ul>{society.deckConstraints.map((constraint, index) => <li key={index}>{constraint.kind === 'minimumColor' && typeof constraint.color === 'string' ? `至少 ${constraint.count} 张${constraint.color.replace(/色$/, '')}色卡 · 当前 ${societyColorCount(draft, catalog, constraint.color)} 张` : constraint.kind === 'greenNeutralOrPrintedHumanCombat' && society.id === 'MSJC11' ? '绿、中立牌；其它派系仅限印刷人类角色且具有永久战斗图标。' : '构筑要求待确认'}</li>)}</ul> : <p>无额外颜色数量要求。</p>}
          </>}
          <small>秘社不加入牌组组成；所有已选秘社在开局时同时公开。</small>
        </section>
        <p className="hg-library-version">草稿来源：{draft.rulesVersion} / {draft.cardPoolVersion} / {draft.engineVersion}</p>
        {versionsChanged && <button type="button" className="hg-button hg-button-quiet" disabled={disabled} onClick={() => edit(revalidateDraftVersions(draft, catalog))}>按当前卡池重新校验</button>}
        {validation.issues.length > 0 && <ul className="hg-library-issues" aria-label="牌组校验问题">{validation.issues.map((issue, index) => <li key={`${issue.code}-${index}`}>{issue.message}</li>)}</ul>}
        {!onSelectDraft && <p className="hg-muted">此入口尚未接入自定义牌组选择，仍可编辑与保存草稿。</p>}
        <h3>牌组组成</h3>
        {draft.cards.length === 0 ? <p className="hg-muted">牌组还是空的。从目录添加卡牌，或复制一套预组开始编辑。</p> : <ul className="hg-library-entries">{draft.cards.map(entry => {
          const card = definitions.get(entry.cardId);
          const label = card ? `${card.name}（${card.id}）` : `未知卡牌（${entry.cardId}）`;
          const editable = !!card && isEditableDeckCard(card);
          return <li key={entry.cardId} data-draft-card={entry.cardId}>
            <div>{card ? <button type="button" className="hg-library-entry-preview" aria-label={`预览已选 ${label}`} onClick={event => openPreview(card.id, event.currentTarget)}><DeckCardFace key={card.id} card={card} /><strong>{card.name}</strong></button> : <strong>未知卡牌</strong>}<small>{entry.cardId} · {copyLabel(card)}{!editable ? ' · 不可用于开局' : ''}</small></div>
            <div className="hg-library-counter">
              <button type="button" aria-label={`减少 ${label}`} disabled={disabled} onClick={() => changeCount(entry.cardId, -1)}>−</button>
              <input type="number" min={0} step={1} aria-label={`${label}张数`} value={entry.count} disabled={disabled || !editable} onChange={event => { if (event.target.value !== '') edit(setDraftCardCount(draft, entry.cardId, Number(event.target.value))); }} />
              <button type="button" aria-label={`增加 ${label}`} disabled={disabled || !editable} onClick={() => changeCount(entry.cardId, 1)}>+</button>
              <button type="button" className="hg-library-remove" aria-label={`移除 ${label}`} disabled={disabled} onClick={() => edit(setDraftCardCount(draft, entry.cardId, 0))}>移除</button>
            </div>
          </li>;
        })}</ul>}
      </div>
      <div className="hg-library-catalog">
        <h3>当前开放卡牌</h3>
        <label className="hg-field">检索卡牌<input ref={searchBox} type="search" value={search} onChange={event => setSearch(event.target.value)} placeholder="名称、编号或规则文字" disabled={disabled} /></label>
        <div className="hg-library-filters">
          <label className="hg-field">派系<select value={faction} onChange={event => setFaction(event.target.value)} disabled={disabled}><option value="all">全部派系</option>{[...factions, neutral].map(item => <option key={item.id} value={item.id}>{item.name}</option>)}<option value="unknown">颜色未标</option></select></label>
          <label className="hg-field">卡牌类型<select value={kind} onChange={event => setKind(event.target.value)} disabled={disabled}><option value="all">全部类型</option>{Object.entries(kinds).filter(([value]) => catalog.cards.some(card => card.kind === value)).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label>
          <label className="hg-field">印刷费用<select value={cost} onChange={event => setCost(event.target.value)} disabled={disabled}><option value="all">全部费用</option>{costs.map(value => <option key={value} value={value}>{value} 费</option>)}</select></label>
          <label className="hg-field">实现状态<select value={implementation} onChange={event => setImplementation(event.target.value)} disabled={disabled}><option value="implemented">已实现玩家卡</option><option value="unavailable">不可加入牌组</option><option value="all">当前目录全部</option></select></label>
        </div>
        <div className="hg-library-filter-footer"><p className="hg-library-match-count" role="status">找到 {choices.length} 种卡牌 · 点击卡面放大</p><label className="hg-library-selected-filter"><input type="checkbox" checked={selectedOnly} onChange={event => setSelectedOnly(event.target.checked)} disabled={disabled} />只看已加入</label><button type="button" className="hg-button hg-button-quiet" onClick={resetFilters} disabled={disabled}>重置筛选</button></div>
        <ul className="hg-library-card-options" aria-label="卡面目录">{choices.map(card => {
          const count = counts.get(card.id) || 0;
          const label = `${card.name}（${card.id}）`;
          const issue = validation.issues.find(issue => issue.cardId === card.id || (issue.cardId && definitions.get(issue.cardId)?.name.trim() === card.name.trim()));
          return <li key={card.id} data-catalog-card={card.id} data-selected-count={count} data-card-issue={issue?.code} className={count ? 'hg-library-card-selected' : undefined}>
            <button type="button" className="hg-library-card-preview" aria-label={`预览 ${label}`} onClick={event => openPreview(card.id, event.currentTarget)}><DeckCardFace key={card.id} card={card} /><span className="hg-library-card-badge">{count ? `已加入 ×${count}` : '点击放大'}</span></button>
            <div className="hg-library-card-heading"><strong>{card.name}</strong></div>
            <small>{card.id} · {factionName(card)} · {kinds[card.kind] || card.kind}{card.kind !== 'society' && ` · 费用 ${card.cost}`}</small>
            <small>{copyLabel(card)}</small>
            <div className="hg-library-grid-counter"><button type="button" className="hg-button hg-button-quiet" aria-label={`目录减少 ${label}`} disabled={disabled || count === 0} onClick={() => changeCount(card.id, -1)}>−</button><span aria-label={`${label}已加入张数`}>{count} 张</span><button type="button" className="hg-button hg-button-quiet" aria-label={`添加 ${label}`} disabled={disabled || !isEditableDeckCard(card)} onClick={() => changeCount(card.id, 1)}>+ 添加</button></div>
            {issue && <p className="hg-library-warning">{issue.message}</p>}
            {!isEditableDeckCard(card) && <p className="hg-library-warning">{card.kind === 'region' ? '地区由世界牌库提供，不可加入玩家牌组。' : card.kind === 'society' ? '秘社须在独立选择中选取，不加入玩家卡张数。' : '该卡尚未确认已实现，不可添加。'}</p>}
            <details><summary>查看规则文字</summary><p className="hg-library-card-text">{card.text}</p>{card.loyaltyText && <p>{card.loyaltyText}</p>}</details>
          </li>;
        })}</ul>
        {choices.length === 0 && <p className="hg-muted">当前开放目录没有符合筛选的卡牌。</p>}
      </div>
    </div>
    {preview && <DeckCardPreview key={preview.id} card={preview} count={counts.get(preview.id) || 0} disabled={disabled} copyLabel={copyLabel(preview)} onChangeCount={delta => changeCount(preview.id, delta)} onClose={() => setPreviewId(null)} />}
  </section>;
}
