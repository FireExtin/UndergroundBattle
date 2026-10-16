import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { factions, neutral } from './factions';
import { createDeckDraft, deckCardAdmission, isDeckDraftDeleted, isEditableDeckCard, localDeckStorage, publicDeckDraft, readDeckLibrary, removeDeckDraft, revalidateDraftVersions, saveDeckDraft, selectableSocieties, setDraftCardCount, societyColorCount, validateDeckDraft } from './deckLibrary';
import type { DeckCardDefinition, DeckCatalog, DeckDraft, DeckStorage, DeckIssue } from './deckLibrary';
import { DeckCardFace } from './DeckCardFace';
import { DeckCardPreview } from './DeckCardPreview';
import { moveCardGridFocus } from './cardGridNavigation';
import { useDialogFocus } from './useDialogFocus';
import { defaultCollectionFilters, readDeckWorkspace, saveDeckWorkspace } from './deckWorkspace';
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
function issuesForCard(card: DeckCardDefinition, issues: DeckIssue[], definitions: Map<string, DeckCardDefinition>) {
  return issues.filter(issue => issue.cardId === card.id || (
    ['copies', 'copy-metadata', 'copy-conflict'].includes(issue.code)
    && issue.cardId && definitions.get(issue.cardId)?.name.trim() === card.name.trim()
  ));
}
const quickFactions = ['red', 'black', 'blue', 'gray'].map(id => factions.find(item => item.id === id)!);
function cardFeatures(card: DeckCardDefinition) {
  return [...(card.keywords || []).map(value => `keyword:${value}`), ...(card.subtypes || []).map(value => `subtype:${value}`), ...(card.magic ? [`magic:${card.magic}`] : [])];
}

export function DeckLibrary({ catalog, disabled = false, onSelectDraft, selectedDraftId, storage, active = true, pageMode = false }: {
  catalog: DeckCatalog;
  disabled?: boolean;
  onSelectDraft?: (draft: DeckDraft) => void;
  selectedDraftId?: string;
  storage?: DeckStorage | null;
  active?: boolean;
  pageMode?: boolean;
}) {
  const componentId = useId();
  const titleId = `${componentId}-library-title`;
  const gridHintId = `${componentId}-grid-hint`;
  const filtersId = `${componentId}-filters`;
  const settingsId = `${componentId}-deck-settings`;
  const [initial] = useState(() => {
    const target = storage === undefined ? localDeckStorage() : storage;
    const library = readDeckLibrary(target);
    const checkpoint = pageMode ? readDeckWorkspace() : null;
    const latest = checkpoint && !checkpoint.dirty ? library.drafts.find(item => item.id === checkpoint.draft.id) : undefined;
    const draft = latest || checkpoint?.draft || library.drafts[0] || createDeckDraft(catalog);
    return { ...library, checkpoint, draft, deleted: isDeckDraftDeleted(draft.id, target) };
  });
  const [saved, setSaved] = useState(initial.drafts);
  const [draft, setDraft] = useState<DeckDraft>(() => publicDeckDraft(initial.draft));
  const [storageWarning, setStorageWarning] = useState(initial.warning);
  const [deletedDraft, setDeletedDraft] = useState(initial.deleted);
  const [notice, setNotice] = useState(initial.deleted ? '此草稿已被删除，当前编辑仍保留；需要时请另存为新草稿。' : initial.checkpoint?.dirty ? '已恢复当前标签尚未保存的编辑。' : '');
  const [dirty, setDirty] = useState(initial.checkpoint?.dirty || false);
  const [workspaceWarning, setWorkspaceWarning] = useState<string | null>(null);
  const [presetId, setPresetId] = useState(catalog.decks[0]?.id || '');
  const filters = initial.checkpoint?.filters || defaultCollectionFilters;
  const [search, setSearch] = useState(filters.search);
  const [faction, setFaction] = useState(filters.faction);
  const [kind, setKind] = useState(filters.kind);
  const [implementation, setImplementation] = useState(filters.implementation);
  const [cost, setCost] = useState(filters.cost);
  const [feature, setFeature] = useState(filters.feature);
  const [selectedOnly, setSelectedOnly] = useState(filters.selectedOnly);
  const [sort, setSort] = useState(filters.sort);
  const [density, setDensity] = useState<'compact' | 'comfortable'>(filters.density);
  const [showFilters, setShowFilters] = useState(false);
  const [showDeckSettings, setShowDeckSettings] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const [previewIds, setPreviewIds] = useState<string[]>([]);
  const catalogGrid = useRef<HTMLUListElement>(null);
  const workspace = useRef<HTMLElement>(null);
  const workspaceButton = useRef<HTMLButtonElement>(null);
  const previewOpener = useRef<HTMLButtonElement>(null);
  const searchBox = useRef<HTMLInputElement>(null);
  const countFocus = useRef<{ source: HTMLElement; targets: HTMLButtonElement[] } | null>(null);
  const openPreview = (id: string, opener: HTMLButtonElement, ids: string[]) => {
    previewOpener.current = opener;
    // Keep the browsing sequence stable when removing a card from “only selected”.
    setPreviewIds(ids); setPreviewId(id);
  };
  const closeWorkspace = () => { setExpanded(false); workspaceButton.current?.focus(); };
  useDialogFocus(workspace, active && expanded && previewId === null, closeWorkspace);
  useEffect(() => {
    if (!active) { setPreviewId(null); setExpanded(false); }
  }, [active]);
  useEffect(() => {
    if (!pageMode) return;
    const saved = saveDeckWorkspace({ draft, dirty, filters: { search, faction, kind, cost, feature, implementation, selectedOnly, sort, density } });
    setWorkspaceWarning(saved ? null : '当前编辑进度无法自动恢复，请使用“保存牌组”保留构筑。');
  }, [pageMode, draft, dirty, search, faction, kind, cost, feature, implementation, selectedOnly, sort, density]);
  useEffect(() => {
    if (!active || !expanded) return;
    const previous = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = previous; };
  }, [active, expanded]);
  useLayoutEffect(() => {
    // Removing the last copy under “only selected” can remove the preview opener.
    if (active && previewId === null && previewOpener.current && !previewOpener.current.isConnected) searchBox.current?.focus();
  }, [active, previewId]);
  useLayoutEffect(() => {
    const pending = countFocus.current;
    countFocus.current = null;
    if (!active || !pending || previewId !== null || (pending.source.isConnected && !pending.source.matches(':disabled'))) return;
    const next = pending.targets.find(target => target.isConnected && !target.disabled);
    (next || searchBox.current)?.focus();
  }, [active, draft, previewId]);
  const validation = validateDeckDraft(draft, catalog);
  const societies = selectableSocieties(catalog);
  const society = societies.find(card => card.id === draft.societyId);
  const definitions = useMemo(() => new Map(catalog.cards.map(card => [card.id, card])), [catalog.cards]);
  const counts = useMemo(() => new Map(draft.cards.map(entry => [entry.cardId, entry.count])), [draft.cards]);
  const costs = [...new Set(catalog.cards.map(card => card.cost))].sort((a, b) => a - b);
  const features = [...new Set(catalog.cards.flatMap(cardFeatures))].sort((a, b) => a.localeCompare(b, 'zh-CN'));
  const featureLabel = (value: string) => {
    const colon = value.indexOf(':');
    return `${({ keyword: '关键词', subtype: '子类型', magic: '领域' }[value.slice(0, colon)] || '特性')} · ${value.slice(colon + 1)}`;
  };
  const preview = previewId ? definitions.get(previewId) : undefined;
  const previewSequence = previewIds.filter(id => definitions.has(id));
  const previewIndex = previewId ? previewSequence.indexOf(previewId) : -1;
  useEffect(() => {
    if (previewId && !definitions.has(previewId)) {
      setPreviewId(null);
      setNotice('目录已更新，刚才预览的卡牌已不在当前目录。草稿内容仍保留。');
    }
  }, [previewId, definitions]);
  const navigatePreview = (delta: number) => setPreviewId(current => {
    const index = current ? previewSequence.indexOf(current) : -1;
    return index < 0 ? current : previewSequence[index + delta] || current;
  });
  const choices = catalog.cards.filter(card => {
    const query = search.trim().toLocaleLowerCase();
    return (!query || `${card.name} ${card.id} ${card.text} ${card.subtypes?.join(' ') || ''} ${card.keywords?.join(' ') || ''} ${card.magic || ''}`.toLocaleLowerCase().includes(query)) && (faction === 'all' || factionId(card) === faction) && (kind === 'all' || kind === card.kind) && (cost === 'all' || String(card.cost) === cost) && (feature === 'all' || cardFeatures(card).includes(feature)) && (!selectedOnly || (counts.get(card.id) || 0) > 0) && (implementation === 'all' || (implementation === 'implemented' ? isEditableDeckCard(card) : !isEditableDeckCard(card)));
  }).sort((a, b) => sort === 'cost-up' ? a.cost - b.cost : sort === 'cost-down' ? b.cost - a.cost
    : sort === 'selected' ? (counts.get(b.id) || 0) - (counts.get(a.id) || 0)
    : sort === 'name' ? a.name.localeCompare(b.name, 'zh-CN') : 0);
  const advancedFilterLabels = [
    faction !== 'all' && !quickFactions.some(item => item.id === faction) ? [...factions, neutral].find(item => item.id === faction)?.name || '颜色未标' : '',
    kind !== 'all' ? kinds[kind] || kind : '',
    cost !== 'all' ? `${cost} 费` : '',
    feature !== 'all' ? featureLabel(feature) : '',
    implementation !== 'implemented' ? implementation === 'all' ? '当前目录全部' : '不可加入牌组' : '',
  ].filter(Boolean);
  useLayoutEffect(() => {
    if (catalogGrid.current) catalogGrid.current.scrollTop = 0;
  }, [search, faction, kind, cost, feature, implementation, selectedOnly, sort, density]);
  useLayoutEffect(() => {
    const grid = catalogGrid.current;
    if (!pageMode || !active || !grid) return;
    // Let the page grow instead of squeezing the catalog below one full card.
    // Actual card height includes badges, controls and wrapped metadata.
    const resize = () => {
      if (window.innerWidth <= 820) {
        grid.style.removeProperty('--hg-library-grid-height');
        return;
      }
      const rowHeight = Math.max(0, ...Array.from(grid.children, card => card.getBoundingClientRect().height));
      if (!rowHeight) return;
      const style = getComputedStyle(grid);
      const gap = Number.parseFloat(style.rowGap) || 0;
      const padding = (Number.parseFloat(style.paddingTop) || 0) + (Number.parseFloat(style.paddingBottom) || 0);
      const height = `${Math.ceil(rowHeight * 2.5 + gap * 1.5 + padding)}px`;
      if (grid.style.getPropertyValue('--hg-library-grid-height') !== height) grid.style.setProperty('--hg-library-grid-height', height);
    };
    resize();
    const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(resize);
    observer?.observe(grid);
    for (const card of grid.children) observer?.observe(card);
    window.addEventListener('resize', resize);
    return () => { observer?.disconnect(); window.removeEventListener('resize', resize); };
  }, [pageMode, active, choices]);
  const versionsChanged = draft.rulesVersion !== catalog.rulesVersion || draft.cardPoolVersion !== catalog.cardPoolVersion || draft.engineVersion !== catalog.engineVersion;
  const storageTarget = storage === undefined ? localDeckStorage() : storage;
  const edit = (next: DeckDraft | ((current: DeckDraft) => DeckDraft)) => { setDraft(next); setDirty(true); setNotice(''); };
  const rememberCountFocus = () => {
    const source = document.activeElement;
    if (!(source instanceof HTMLElement)) return;
    const row = source.closest('[data-catalog-card], [data-draft-card]');
    if (!row?.parentElement) return;
    const rows = Array.from(row.parentElement.children);
    const index = rows.indexOf(row);
    // A retained draft may contain a card that has left the catalog and has no preview.
    const focusTarget = (entry?: Element) => entry?.querySelector<HTMLButtonElement>('.hg-library-card-preview, .hg-library-entry-preview, button:not(:disabled)');
    countFocus.current = { source, targets: [focusTarget(row), focusTarget(rows[index + 1]), focusTarget(rows[index - 1])].filter((button): button is HTMLButtonElement => !!button) };
  };
  const changeCount = (cardId: string, delta: number) => {
    rememberCountFocus();
    edit(current => setDraftCardCount(current, cardId, Math.max(0, (current.cards.find(entry => entry.cardId === cardId)?.count || 0) + delta)));
  };
  const setCount = (cardId: string, count: number) => {
    rememberCountFocus();
    edit(current => setDraftCardCount(current, cardId, count));
  };
  const resetFilters = () => { setSearch(''); setFaction('all'); setKind('all'); setCost('all'); setFeature('all'); setSelectedOnly(false); setImplementation('implemented'); };
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
  if (!active) return null;
  return <section ref={workspace} tabIndex={expanded ? -1 : undefined} className={`hg-deck-library${pageMode ? ' hg-library-page' : ''}${expanded ? ' hg-library-expanded' : ''}`} role={expanded ? 'dialog' : undefined} aria-modal={expanded ? true : undefined} aria-labelledby={titleId}>
    <div className="hg-library-workspace-heading">
    <div className="hg-section-title"><div><span className="hg-eyebrow">我的牌组 · 卡牌档案</span><h2 id={titleId}>卡面组卡工作台</h2></div><div className="hg-library-workspace-actions"><span className="hg-pool-badge">本地浏览器保存</span>{!pageMode && <button ref={workspaceButton} type="button" className="hg-button hg-button-quiet" onClick={() => expanded ? closeWorkspace() : setExpanded(true)}>{expanded ? '返回大厅组卡' : '展开组卡工作台'}</button>}</div></div>
    <details className="hg-library-guide"><summary>构筑与保存说明 · 当前可编辑 {catalog.cards.filter(isEditableDeckCard).length} 种玩家卡</summary>
      <p className="hg-muted">任意派系可以混合；通常至少 50 张，同名最多 3 张，印刷例外以目录标注为准。只可添加当前已实现的玩家卡；可选秘社独立保存，不计入玩家卡张数。</p>
      <p className="hg-muted">这里只展示当前开放目录，并非完整八派系卡池。草稿保存在此浏览器，不会跨设备同步。</p>
    </details>
    {storageWarning && <p className="hg-library-warning" role="alert">{storageWarning}</p>}
    {workspaceWarning && <p className="hg-library-warning" role="status">{workspaceWarning}</p>}
    </div>
    <div className="hg-library-editor">
      <div className="hg-library-edit-panel">
        <div className="hg-library-deck-status">
          {(expanded || pageMode) && <strong className="hg-library-current-name" aria-label="当前草稿">{draft.name.trim() || '未命名牌组'}</strong>}
          <div className="hg-library-total" role="status"><strong>{validation.total} 张</strong><span>{validation.valid ? '构筑校验通过' : '暂不可开局'}{dirty ? ' · 有未保存修改' : ''}</span></div>
          <div className="hg-library-save-actions">
            <button type="button" className="hg-button hg-button-quiet" disabled={disabled || !draft.name.trim()} onClick={() => persist()}>保存牌组</button>
            {deletedDraft && <button type="button" className="hg-button hg-button-quiet" disabled={disabled || !draft.name.trim()} onClick={() => persist({ ...publicDeckDraft(draft), id: createDeckDraft(catalog).id })}>另存为新草稿</button>}
            <button type="button" className="hg-button hg-button-primary" disabled={disabled || !validation.valid || !onSelectDraft} onClick={() => { const next = persist(); if (next) onSelectDraft?.(publicDeckDraft(next)); }}>保存并选择此牌组</button>
          </div>
          {notice && <p className="hg-library-notice" role="status">{notice}</p>}
        </div>
        <h3>我的已保存牌组</h3>
        {saved.length > 0 && <ul className="hg-library-saved" aria-label="已保存牌组">{saved.map(item => <li key={item.id}>
          <button type="button" aria-pressed={draft.id === item.id} disabled={disabled} onClick={() => { if (draft.id !== item.id) replace(publicDeckDraft(item)); }}><strong>{item.name || '未命名牌组'}</strong><span>{item.cards.reduce((sum, entry) => sum + entry.count, 0)} 张 · {validateDeckDraft(item, catalog).valid ? '构筑校验通过' : '待修正'}{selectedDraftId === item.id ? ' · 已选择入席' : ''}{draft.id === item.id && dirty ? ' · 正在编辑未保存修改' : ''}</span></button>
          <button type="button" className="hg-button hg-button-quiet" aria-label={`删除牌组 ${item.name}`} disabled={disabled} onClick={() => remove(item.id)}>删除</button>
        </li>)}</ul>}
        <label className="hg-field">牌组名称<input value={draft.name} onChange={event => edit({ ...draft, name: event.target.value })} maxLength={80} disabled={disabled} /></label>
        {!saved.length && <p className="hg-muted">保存牌组后会列在这里，入席时也可直接选择。</p>}
        {(expanded || pageMode) && <button type="button" className="hg-button hg-button-quiet hg-library-settings-toggle" aria-expanded={showDeckSettings} aria-controls={settingsId} onClick={() => setShowDeckSettings(value => !value)}>{showDeckSettings ? '收起牌组设置' : '牌组设置与模板'}{versionsChanged ? ' · 版本待确认' : ''}</button>}
        <div id={settingsId} hidden={(expanded || pageMode) && !showDeckSettings}>
        <div className="hg-library-toolbar">
          <button type="button" className="hg-button hg-button-quiet" disabled={disabled} onClick={() => replace(createDeckDraft(catalog), true)}>新建空白牌组</button>
          <label className="hg-field">预组模板<select value={presetId} onChange={event => setPresetId(event.target.value)} disabled={disabled || !catalog.decks.length}><option value="">选择预组模板</option>{presetId && !catalog.decks.some(deck => deck.id === presetId) && <option value={presetId}>原模板已不在当前目录</option>}{catalog.decks.map(deck => <option key={deck.id} value={deck.id}>{deck.name}</option>)}</select></label>
          <button type="button" className="hg-button hg-button-quiet" disabled={disabled || !catalog.decks.some(deck => deck.id === presetId)} onClick={() => replace(createDeckDraft(catalog, catalog.decks.find(deck => deck.id === presetId)), true)}>复制为草稿</button>
        </div>
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
        </div>
        {validation.issues.length > 0 && <ul className="hg-library-issues" aria-label="牌组校验问题">{validation.issues.map((issue, index) => <li key={`${issue.code}-${index}`}>{issue.message}</li>)}</ul>}
        {!onSelectDraft && <p className="hg-muted">此入口尚未接入自定义牌组选择，仍可编辑与保存草稿。</p>}
        <h3>牌组组成</h3>
        {draft.cards.length === 0 ? <p className="hg-muted">牌组还是空的。从目录添加卡牌，或复制一套预组开始编辑。</p> : <ul className="hg-library-entries">{draft.cards.map(entry => {
          const card = definitions.get(entry.cardId);
          const label = card ? `${card.name}（${card.id}）` : `未知卡牌（${entry.cardId}）`;
          const editable = !!card && isEditableDeckCard(card);
          return <li key={entry.cardId} data-draft-card={entry.cardId}>
            <div>{card ? <button type="button" className="hg-library-entry-preview" aria-label={`预览已选 ${label}`} onClick={event => openPreview(card.id, event.currentTarget, draft.cards.map(item => item.cardId))}><DeckCardFace key={card.id} card={card} /><strong>{card.name}</strong></button> : <strong>未知卡牌</strong>}<small>{entry.cardId} · {copyLabel(card)}{!editable ? ' · 不可用于开局' : ''}</small></div>
            <div className="hg-library-counter">
              <button type="button" aria-label={`减少 ${label}`} disabled={disabled} onClick={() => changeCount(entry.cardId, -1)}>−</button>
              <input type="number" min={0} step={1} aria-label={`${label}张数`} value={entry.count} disabled={disabled || !editable} onChange={event => { if (event.target.value !== '') setCount(entry.cardId, Number(event.target.value)); }} />
              <button type="button" aria-label={`增加 ${label}`} disabled={disabled || !editable} onClick={() => changeCount(entry.cardId, 1)}>+</button>
              <button type="button" className="hg-library-remove" aria-label={`移除 ${label}`} disabled={disabled} onClick={() => setCount(entry.cardId, 0)}>移除</button>
            </div>
          </li>;
        })}</ul>}
      </div>
      <div className="hg-library-catalog">
        <div className="hg-library-catalog-heading"><h3>当前开放卡牌</h3>{expanded && <button type="button" className="hg-button hg-button-quiet" aria-expanded={showFilters} aria-controls={filtersId} onClick={() => setShowFilters(value => !value)}>{showFilters ? '收起筛选' : '更多筛选'}{advancedFilterLabels.length > 0 ? ` · ${advancedFilterLabels.length} 项` : ''}</button>}</div>
        <div className="hg-library-quick-factions" role="group" aria-label="派系速选">
          <button type="button" aria-pressed={faction === 'all'} disabled={disabled} onClick={() => setFaction('all')}>全部</button>
          {quickFactions.map(item => <button type="button" key={item.id} className={`hg-faction-${item.id}`} aria-pressed={faction === item.id} disabled={disabled} onClick={() => setFaction(current => current === item.id ? 'all' : item.id)}><i className="hg-faction-swatch" aria-hidden="true" />{item.color} · {item.name}</button>)}
        </div>
        <label className="hg-field hg-library-search"><span>检索卡牌</span><input ref={searchBox} type="search" value={search} onChange={event => setSearch(event.target.value)} placeholder="名称、编号或规则文字" disabled={disabled} /></label>
        <div id={filtersId} className="hg-library-filters" hidden={expanded && !showFilters}>
          <label className="hg-field">派系<select value={faction} onChange={event => setFaction(event.target.value)} disabled={disabled}><option value="all">全部派系</option>{[...factions, neutral].map(item => <option key={item.id} value={item.id}>{item.name}</option>)}<option value="unknown">颜色未标</option></select></label>
          <label className="hg-field">卡牌类型<select value={kind} onChange={event => setKind(event.target.value)} disabled={disabled}><option value="all">全部类型</option>{kind !== 'all' && !catalog.cards.some(card => card.kind === kind) && <option value={kind}>{kinds[kind] || kind}（当前目录无此类型）</option>}{Object.entries(kinds).filter(([value]) => catalog.cards.some(card => card.kind === value)).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label>
          <label className="hg-field">印刷费用<select value={cost} onChange={event => setCost(event.target.value)} disabled={disabled}><option value="all">全部费用</option>{cost !== 'all' && !costs.some(value => String(value) === cost) && <option value={cost}>{cost} 费（当前目录无此费用）</option>}{costs.map(value => <option key={value} value={value}>{value} 费</option>)}</select></label>
          <label className="hg-field">特性<select value={feature} onChange={event => setFeature(event.target.value)} disabled={disabled}><option value="all">全部特性</option>{feature !== 'all' && !features.includes(feature) && <option value={feature}>{featureLabel(feature)}（当前目录无此特性）</option>}{features.map(value => <option key={value} value={value}>{featureLabel(value)}</option>)}</select></label>
          <label className="hg-field">组牌准入<select value={implementation} onChange={event => setImplementation(event.target.value)} disabled={disabled}><option value="implemented">可组牌</option><option value="unavailable">仅查看 / 暂不可用</option><option value="all">当前目录全部</option></select></label>
        </div>
        {expanded && !showFilters && advancedFilterLabels.length > 0 && <p className="hg-library-active-filters" aria-label="生效的更多筛选">筛选：{advancedFilterLabels.join(' · ')}</p>}
        <div className="hg-library-filter-footer"><p className="hg-library-match-count" role="status">找到 {choices.length} 种卡牌 · 点击卡面放大</p><label className="hg-library-selected-filter"><input type="checkbox" checked={selectedOnly} onChange={event => setSelectedOnly(event.target.checked)} disabled={disabled} />只看已加入</label><button type="button" className="hg-button hg-button-quiet" onClick={resetFilters} disabled={disabled}>重置筛选</button></div>
        <div className="hg-library-view-controls">
          <label className="hg-field">卡牌排序<select value={sort} onChange={event => setSort(event.target.value)}><option value="catalog">目录顺序</option><option value="cost-up">费用从低到高</option><option value="cost-down">费用从高到低</option><option value="selected">已加入优先</option><option value="name">卡牌名称</option></select></label>
          <div role="group" aria-label="卡面大小"><button type="button" aria-pressed={density === 'compact'} onClick={() => setDensity('compact')}>紧凑</button><button type="button" aria-pressed={density === 'comfortable'} onClick={() => setDensity('comfortable')}>舒展</button></div>
        </div>
        <p className="hg-library-grid-hint" id={gridHintId}>点卡面仅预览；用 + / − 调整张数。聚焦卡面后，可用方向键浏览网格。</p>
        <ul ref={catalogGrid} className="hg-library-card-options" data-density={density} aria-label="卡面目录" aria-describedby={gridHintId}>{choices.map(card => {
          const count = counts.get(card.id) || 0;
          const label = `${card.name}（${card.id}）`;
          const issue = issuesForCard(card, validation.issues, definitions)[0];
          const admission = deckCardAdmission(card);
          return <li key={card.id} data-catalog-card={card.id} data-card-admission={admission.status} data-selected-count={count} data-card-issue={issue?.code} className={count ? 'hg-library-card-selected' : undefined}>
            <button type="button" className="hg-library-card-preview" aria-label={`预览 ${label}`} onKeyDown={event => moveCardGridFocus(event, catalogGrid.current)} onClick={event => openPreview(card.id, event.currentTarget, choices.map(item => item.id))}><DeckCardFace key={card.id} card={card} /><span className={`hg-library-card-badge hg-admission-${admission.status}`}>{admission.label}{count ? ` · 已加入 ×${count}` : ''}</span></button>
            <div className="hg-library-card-heading"><strong>{card.name}</strong></div>
            <div className="hg-library-grid-counter"><button type="button" className="hg-button hg-button-quiet" aria-label={`目录减少 ${label}`} disabled={disabled || count === 0} onClick={() => changeCount(card.id, -1)}>−</button><span aria-label={`${label}已加入张数`}>{count} 张</span><button type="button" className="hg-button hg-button-quiet" aria-label={`添加 ${label}`} disabled={disabled || !isEditableDeckCard(card)} onClick={() => changeCount(card.id, 1)}>{density === 'compact' ? '+' : '+ 添加'}</button></div>
            <small>{card.id} · {factionName(card)} · {kinds[card.kind] || card.kind}{card.kind !== 'society' && ` · 费用 ${card.cost}`}</small>
            <small>{copyLabel(card)}</small>
            {issue && <p className="hg-library-warning">当前牌组：{issue.message}</p>}
            {admission.status !== 'buildable' && <p className="hg-library-warning">{admission.reason}</p>}
            <details><summary>查看规则文字</summary><p className="hg-library-card-text">{card.text}</p>{card.loyaltyText && <p>{card.loyaltyText}</p>}</details>
          </li>;
        })}</ul>
        {choices.length === 0 && <p className="hg-muted">当前开放目录没有符合筛选的卡牌。</p>}
      </div>
    </div>
    {preview && <DeckCardPreview card={preview} count={counts.get(preview.id) || 0} disabled={disabled} copyLabel={copyLabel(preview)}
      sameNameCount={draft.cards.reduce((total, entry) => total + (definitions.get(entry.cardId)?.name.trim() === preview.name.trim() ? entry.count : 0), 0)}
      issues={issuesForCard(preview, validation.issues, definitions)} validation={validation}
      navigation={{ index: previewIndex, total: previewSequence.length, previous: () => navigatePreview(-1), next: () => navigatePreview(1) }}
      onChangeCount={delta => changeCount(preview.id, delta)} onClose={() => setPreviewId(null)} />}
  </section>;
}
