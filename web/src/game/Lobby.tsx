import { useEffect, useRef, useState } from 'react';
import { FactionCoverage } from './FactionCoverage';
import { DeckLibrary } from './DeckLibraryPanel';
import { createDeckDraft, publicDeckDraft, readDeckLibrary, selectableSocieties, validateDeckDraft } from './deckLibrary';
import type { DeckDraft } from './deckLibrary';
import { deckColors } from './factions';
import type { CardDefinition, Catalog, Deck } from './types';
import { navigateLobby, useLobbyPage } from './lobbyNavigation';
import { readLobbySelection, saveLobbySelection } from './deckWorkspace';
import { readSavedSeats } from './api';
import { readPlayerName, rememberPlayerName } from './playerNameStorage';
import './lobby-pages.css';

export const deckThemes: Record<string, { mark: string; role: string }> = {
  watchers: { mark: '◈', role: '调查 · 控制' },
  hunters: { mark: '⚔', role: '机动 · 战斗' },
  keepers: { mark: '⌖', role: '守备 · 势力' },
  reclaimers: { mark: '✧', role: '墓地 · 回收' },
  responders: { mark: '⌁', role: '牺牲 · 响应' },
};

export function DeckPicker({ decks, cards, value, onChange, disabled = false, allowedIds }: { decks: Deck[]; cards: CardDefinition[]; value: string; onChange: (id: string) => void; disabled?: boolean; allowedIds?: string[] }) {
  return <div className="hg-decks">{decks.map((deck, index) => {
    const theme = deckThemes[deck.id] || { mark: '◈', role: '自组预组' };
    const composition = deckColors(deck, cards);
    return <button type="button" key={deck.id} data-deck-id={deck.id} data-deck-main-faction={composition.main.id} className={`hg-deck hg-deck-faction-${composition.main.id} ${value === deck.id ? 'hg-selected' : ''}`} onClick={() => onChange(deck.id)} disabled={disabled || (allowedIds !== undefined && !allowedIds.includes(deck.id))} aria-pressed={value === deck.id}>
      <span className="hg-deck-index">0{index + 1}</span><span className="hg-deck-sigil" aria-hidden="true">{theme.mark}</span>
      <span className="hg-deck-role">{theme.role}</span><strong>{deck.name}</strong><span className="hg-deck-description">{deck.description}</span>
      <span className="hg-deck-colors" aria-label={`${deck.name}的颜色组成`}>{composition.colors.map(({ faction, count }) => <span key={faction.id} className={`hg-faction-${faction.id}`} data-deck-faction={faction.id} data-card-count={count}><i className="hg-faction-swatch" aria-hidden="true" />{faction.color} · {faction.name} {count} 张</span>)}</span>
      <span className="hg-deck-footer">{composition.total} 张 · 自组预组 <span>{value === deck.id ? '✓ 已选择' : '选择牌组 ↗'}</span></span>
    </button>;
  })}</div>;
}

export function Lobby({ independent = false, catalog, busy, onCreate, onJoin, retry, onSelectDraft, onCreateDraft, onJoinDraft }: {
  independent?: boolean; catalog: Catalog | null; busy: boolean;
  onCreate: (name: string, mode: 'duel' | 'teams', deckId: string) => void;
  onJoin: (invite: string, name: string, deckId: string) => void; retry: () => void;
  onSelectDraft?: (draft: DeckDraft) => void;
  onCreateDraft?: (name: string, mode: 'duel' | 'teams', draft: DeckDraft) => void;
  onJoinDraft?: (invite: string, name: string, draft: DeckDraft) => void;
}) {
  const invitation = new URL(location.href).searchParams;
  const invitationOnly = invitation.has('invite');
  const initialInvite = invitation.get('invite') || '';
  const requestedPage = useLobbyPage();
  const page = invitationOnly && requestedPage !== 'collection' ? 'play' : requestedPage;
  const [openedCollection, setOpenedCollection] = useState(page === 'collection');
  const heading = useRef<HTMLHeadingElement>(null);
  const previousPage = useRef(page);
  const [initialSelection] = useState(readLobbySelection);
  const [name, setName] = useState(readPlayerName);
  const submittedName = useRef<string | null>(null);
  const [mode, setMode] = useState<'duel' | 'teams'>('duel');
  const [invite, setInvite] = useState(initialInvite);
  const [tab, setTab] = useState<'create' | 'join'>(initialInvite ? 'join' : 'create');
  const [deckId, setDeckId] = useState(initialSelection?.kind === 'preset' ? initialSelection.deckId : '');
  const [selectedDraft, setSelectedDraft] = useState<DeckDraft | null>(initialSelection?.kind === 'draft' ? initialSelection.draft : null);
  const [selectionWarning, setSelectionWarning] = useState(false);
  useEffect(() => {
    if (!selectedDraft && !deckId) return;
    setSelectionWarning(!saveLobbySelection(selectedDraft ? { kind: 'draft', draft: selectedDraft } : { kind: 'preset', deckId }));
  }, [selectedDraft, deckId]);
  useEffect(() => {
    if (page === 'collection') setOpenedCollection(true);
    if (previousPage.current !== page) {
      heading.current?.focus({ preventScroll: true });
      heading.current?.closest('main')?.scrollIntoView?.({ block: 'start' });
      previousPage.current = page;
    }
  }, [page]);
  const selected = deckId || catalog?.decks[0]?.id || '';
  const deck = catalog?.decks.find(item => item.id === selected);
  const joining = invitationOnly || tab === 'join';
  const targetInvite = invitationOnly ? initialInvite : invite;
  const restoringSeat = joining && readSavedSeats().some(seat => seat.inviteCode.trim().toUpperCase() === targetInvite.trim().toUpperCase());
  const savedDecks = readDeckLibrary().drafts;
  const draftValidation = selectedDraft && catalog ? validateDeckDraft(selectedDraft, catalog) : null;
  const draftValid = draftValidation?.valid === true;
  const societies = catalog ? selectableSocieties(catalog) : [];
  const societyId = selectedDraft?.societyId || '';
  const selectedSociety = societies.find(society => society.id === societyId);
  const draftEntryAvailable = joining ? !!onJoinDraft : !!onCreateDraft;
  const entryAvailable = restoringSeat || (selectedDraft ? draftValid && draftEntryAvailable : !!deck);
  const selectedCount = selectedDraft?.cards.reduce((total, item) => total + item.count, 0) ?? deck?.cardCount;
  return <main className={`hg-lobby hg-lobby-${page}`}>
    <nav className="hg-lobby-navigation" aria-label="大厅导航">
      {!invitationOnly && <button type="button" onClick={() => navigateLobby('home')} aria-current={page === 'home' ? 'page' : undefined}>首页</button>}
      <button type="button" onClick={() => navigateLobby('play')} aria-current={page === 'play' ? 'page' : undefined}>入席</button>
      <button type="button" onClick={() => navigateLobby('collection')} aria-current={page === 'collection' ? 'page' : undefined}>卡牌收藏</button>
      {selectedDraft && <span className="hg-lobby-current-deck">已选择：{selectedDraft.name}</span>}
    </nav>
    {selectionWarning && <p className="hg-library-warning" role="status">浏览器无法保留本次牌组选择，刷新后请重新选择。</p>}
    {page === 'home' && <><section className="hg-hero">
      <div className="hg-hero-orbit" aria-hidden="true"><span>◈</span><i /><i /></div>
      <div className="hg-eyebrow">THE SECRET WORLD · 霸权</div>
      <h1 ref={heading} tabIndex={-1}>世界的背面，<br /><em>等你落子。</em></h1>
      <p>派遣角色，争夺城市的隐秘权柄。<br />与一位对手交锋，或和伙伴并肩加入四人牌桌。</p>
      <div className="hg-hero-features"><span>◈ 调查</span><span>⚔ 战斗</span><span>⚑ 势力</span></div>
    </section><section className="hg-lobby-portals" aria-label="开始探索">
      <button type="button" className="hg-lobby-portal" onClick={() => { setTab('create'); navigateLobby('play'); }}><span aria-hidden="true">Ⅰ · ◈</span><strong>创建牌桌</strong><p>选择两人对决或四人协作，邀伙伴入席。</p><small>建立新的邀请牌桌 →</small></button>
      <button type="button" className="hg-lobby-portal" onClick={() => { setTab('join'); navigateLobby('play'); }}><span aria-hidden="true">Ⅱ · ⌖</span><strong>邀请码加入</strong><p>带着伙伴分享的邀请码，前往同一张牌桌。</p><small>前往入席 →</small></button>
      <button type="button" className="hg-lobby-portal hg-lobby-portal-collection" onClick={() => navigateLobby('collection')}><span aria-hidden="true">Ⅲ · ✧</span><strong>卡牌收藏与组牌</strong><p>检索真实卡牌，按派系与特性筛选，构筑你的牌组。</p><small>打开卡牌档案 →</small></button>
    </section></>}
    {page === 'play' && <section className="hg-onboarding hg-play-page">
      <div className="hg-eyebrow">INVITATION · 云端牌桌</div><h1 ref={heading} tabIndex={-1}>准备入席</h1>
      <form className="hg-entry-form" onSubmit={event => {
        event.preventDefault(); if (!catalog || !entryAvailable || busy || joining && !targetInvite.trim()) return;
        const entryName = submittedName.current ?? rememberPlayerName(name);
        submittedName.current = entryName; setName(entryName);
        if (restoringSeat) onJoin(targetInvite.trim(), entryName, selected);
        else if (selectedDraft) {
          if (!joining) onCreateDraft?.(entryName, mode, publicDeckDraft(selectedDraft));
          else onJoinDraft?.(targetInvite.trim(), entryName, publicDeckDraft(selectedDraft));
        } else if (!joining) onCreate(entryName, mode, selected); else onJoin(targetInvite.trim(), entryName, selected);
      }}>
        <div className="hg-section-title"><span className="hg-eyebrow">01 / {invitationOnly ? '受邀加入' : '入席'}</span>{!invitationOnly && <div className="hg-tabs"><button type="button" className={tab === 'create' ? 'hg-tab-active' : ''} onClick={() => setTab('create')}>创建牌桌</button><button type="button" className={tab === 'join' ? 'hg-tab-active' : ''} onClick={() => setTab('join')}>邀请码加入</button></div>}</div>
        {invitationOnly && <p className="hg-muted">此邀请只加入下方牌桌。</p>}
        <p className="hg-muted" aria-label="所选牌组">已选择：{selectedDraft?.name || deck?.name || '加载中'}{selectedCount !== undefined ? ` · ${selectedCount} 张` : ''} · <button type="button" className="hg-text-link" onClick={() => navigateLobby('collection')}>收藏与组牌 →</button></p>
        <label className="hg-field">已保存牌组<select value={selectedDraft ? 'selected' : ''} disabled={busy} onChange={event => {
          const saved = savedDecks.find(item => `saved:${item.id}` === event.target.value);
          if (saved) { setSelectedDraft(publicDeckDraft(saved)); onSelectDraft?.(publicDeckDraft(saved)); }
        }}><option value="">从已保存牌组中选择</option>{selectedDraft && <option value="selected">当前所选快照：{selectedDraft.name} · {selectedCount} 张</option>}{savedDecks.map(item => <option key={item.id} value={`saved:${item.id}`}>{item.name} · {item.cards.reduce((total, card) => total + card.count, 0)} 张</option>)}</select></label>
        {selectedDraft && <p className="hg-muted">入席使用当前所选快照；收藏中编辑后请重新保存并选择。</p>}
        {catalog && <label className="hg-field">入席秘社（可选）<select value={societyId} disabled={busy || restoringSeat || !societies.length} onChange={event => {
          const base = selectedDraft || (deck ? createDeckDraft(catalog, deck) : null);
          if (!base) return;
          const next = { ...publicDeckDraft(base), societyId: event.target.value || null };
          setSelectedDraft(next); onSelectDraft?.(publicDeckDraft(next));
        }}><option value="">不选择秘社 · 起手 6 张</option>{societyId && !selectedSociety && <option value={societyId}>原选秘社当前未开放</option>}{societies.map(society => <option key={society.id} value={society.id}>{society.name} · {society.color} · 起手 {society.startingHand} 张</option>)}</select></label>}
        {selectedSociety && <p className="hg-muted">{selectedSociety.name}：{selectedSociety.deckConstraints.length ? selectedSociety.deckConstraints.map(constraint => constraint.kind === 'minimumColor' ? `至少 ${constraint.count} 张${constraint.color}色牌` : '绿、中立牌或印刷人类且具有永久战斗图标的角色').join('；') : '无额外构筑限制'}。更改秘社仅用于当前入席选择；需要保留时，请在收藏与组牌中保存。</p>}
        {restoringSeat && <p className="hg-muted">恢复原席会保留桌内秘社；入席后可在准备前显式更换牌组。</p>}
        {selectedDraft && !restoringSeat && !draftEntryAvailable && <p className="hg-library-warning" role="alert">草稿已保存，当前入口尚未支持自定义牌组{joining ? '加入' : '创建'}；可以改选原预组。</p>}
        {selectedDraft && !restoringSeat && !draftValid && <div className="hg-library-warning" role="alert"><p>已选择草稿需要按当前卡池重新校验，暂不可入席。</p><ul>{draftValidation?.issues.map((issue, index) => <li key={`${issue.code}:${issue.cardId || index}`}>{issue.message}</li>)}</ul></div>}
        {!selectedDraft && !!deckId && !!catalog && !deck && <p className="hg-library-warning" role="alert">上次选择的预组已不在当前目录，请重新选择。</p>}
        <label className="hg-field">你的称呼<input value={name} onChange={event => { setName(event.target.value); submittedName.current = null; }} maxLength={24} placeholder="留空自动生成称呼" autoComplete="nickname" disabled={busy} /></label>
        {!joining ? <div className="hg-mode-select" role="group" aria-label="对战模式">
          <button type="button" className={mode === 'duel' ? 'hg-selected' : ''} onClick={() => setMode('duel')} disabled={busy}><strong>两人对决 <span>1 VS 1</span></strong><small>3 个地区 · 先获 8 分胜利</small></button>
          <button type="button" className={mode === 'teams' ? 'hg-selected' : ''} onClick={() => setMode('teams')} disabled={busy}><strong>四人协作 <span>2 VS 2</span></strong><small>5 个地区 · 团队先获 10 分胜利</small></button>
        </div> : <label className="hg-field">邀请码<input value={targetInvite} onChange={event => setInvite(event.target.value)} readOnly={invitationOnly} placeholder="粘贴伙伴发来的邀请码" required autoComplete="off" disabled={busy} /></label>}
        <div className="hg-entry-bottom"><p>{joining ? '加入同一牌桌，所有玩家准备后由房主开始。' : '创建后分享邀请，所有玩家准备后由房主开始。'}<br />{independent ? '座位保存在当前标签，刷新即可继续。' : '座位保存在此浏览器，刷新即可继续。'}{joining && <><br />已有此房座位时恢复原席，保留原称呼、实际上桌牌组与准备状态。新选择的牌组须入席后显式应用。</>}</p><button className="hg-button hg-button-primary" type="submit" disabled={busy || !catalog || !entryAvailable || (joining && !targetInvite.trim())}>{busy ? '正在入席…' : !joining ? '创建牌桌 →' : '加入牌桌 →'}</button></div>
      </form>
      <div id="hegemony-decks" className="hg-section-title"><span className="hg-eyebrow">02 / 选择自组预组</span><span className="hg-pool-badge">受限真实卡池预组</span></div>
      <h2>{catalog ? `${catalog.decks.length} 套自组预组` : '自组预组'}</h2>
      <p className="hg-muted">每套 50 张，来自已开放的真实卡牌。当前为受限卡池自组预组，并非官方预组。</p>
      {catalog ? <><DeckPicker decks={catalog.decks} cards={catalog.cards} value={selectedDraft ? '' : selected} onChange={id => { setDeckId(id); setSelectedDraft(null); }} disabled={busy} />
        {!selectedDraft && deck && <details className="hg-deck-list"><summary>查看「{deck.name}」的 {deck.cardCount} 张组成</summary><ul>{deck.cards.map(item => <li key={item.cardId}><span>{catalog.cards.find(card => card.id === item.cardId)?.name || item.cardId}</span><b>× {item.count}</b></li>)}</ul></details>}
      </> : <div className="hg-loading"><span>正在连接牌组档案…</span><button className="hg-button hg-button-quiet" onClick={retry}>重新连接</button></div>}
      <FactionCoverage catalog={catalog} />
    </section>}
    {page === 'collection' && <div className="hg-collection-heading"><div><span className="hg-eyebrow">CARD ARCHIVE · 我的构筑</span><h1 ref={heading} tabIndex={-1}>卡牌收藏与组牌</h1></div><p>按逐卡准入标记组牌；派系标签不代表整派系已开放。</p></div>}
    {catalog && (openedCollection || page === 'collection') && <DeckLibrary catalog={catalog} disabled={busy} active={page === 'collection'} pageMode selectedDraft={selectedDraft} onSelectDraft={draft => { setSelectedDraft(publicDeckDraft(draft)); onSelectDraft?.(publicDeckDraft(draft)); navigateLobby('play'); }} />}
    {page === 'collection' && !catalog && <div className="hg-loading"><span>正在连接卡牌档案…</span><button type="button" className="hg-button hg-button-quiet" onClick={retry}>重新连接</button></div>}
  </main>;
}
