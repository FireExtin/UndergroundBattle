import { useState } from 'react';
import { FactionCoverage } from './FactionCoverage';
import { DeckLibrary } from './DeckLibrary';
import { publicDeckDraft, validateDeckDraft } from './deckLibrary';
import type { DeckDraft } from './deckLibrary';
import { deckColors } from './factions';
import type { CardDefinition, Catalog, Deck } from './types';

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

export function Lobby({ catalog, busy, onCreate, onJoin, retry, onSelectDraft, onCreateDraft, onJoinDraft }: {
  catalog: Catalog | null; busy: boolean;
  onCreate: (name: string, mode: 'duel' | 'teams', deckId: string) => void;
  onJoin: (invite: string, name: string, deckId: string) => void; retry: () => void;
  onSelectDraft?: (draft: DeckDraft) => void;
  onCreateDraft?: (name: string, mode: 'duel' | 'teams', draft: DeckDraft) => void;
  onJoinDraft?: (invite: string, name: string, draft: DeckDraft) => void;
}) {
  const initialInvite = new URL(location.href).searchParams.get('invite') || '';
  const [name, setName] = useState('');
  const [mode, setMode] = useState<'duel' | 'teams'>('duel');
  const [invite, setInvite] = useState(initialInvite);
  const [tab, setTab] = useState<'create' | 'join'>(initialInvite ? 'join' : 'create');
  const [deckId, setDeckId] = useState('');
  const [selectedDraft, setSelectedDraft] = useState<DeckDraft | null>(null);
  const selected = deckId || catalog?.decks[0]?.id || '';
  const deck = catalog?.decks.find(item => item.id === selected);
  const draftValid = !!selectedDraft && !!catalog && validateDeckDraft(selectedDraft, catalog).valid;
  const draftEntryAvailable = tab === 'create' ? !!onCreateDraft : !!onJoinDraft;
  const entryAvailable = selectedDraft ? draftValid && draftEntryAvailable : !!selected;
  return <main className="hg-lobby">
    <section className="hg-hero">
      <div className="hg-hero-orbit" aria-hidden="true"><span>◈</span><i /><i /></div>
      <div className="hg-eyebrow">THE SECRET WORLD · 霸权</div>
      <h1>世界的背面，<br /><em>等你落子。</em></h1>
      <p>派遣角色，争夺城市的隐秘权柄。<br />与一位对手交锋，或和伙伴并肩加入四人牌桌。</p>
      <div className="hg-hero-features"><span>◈ 调查</span><span>⚔ 战斗</span><span>⚑ 势力</span></div>
    </section>
    <section className="hg-onboarding">
      <form className="hg-entry-form" onSubmit={event => {
        event.preventDefault(); if (!catalog || !name.trim() || !entryAvailable || busy) return;
        if (selectedDraft) {
          if (tab === 'create') onCreateDraft?.(name.trim(), mode, publicDeckDraft(selectedDraft));
          else if (invite.trim()) onJoinDraft?.(invite.trim(), name.trim(), publicDeckDraft(selectedDraft));
        } else if (tab === 'create') onCreate(name.trim(), mode, selected); else if (invite.trim()) onJoin(invite.trim(), name.trim(), selected);
      }}>
        <div className="hg-section-title"><span className="hg-eyebrow">01 / 入席</span><div className="hg-tabs"><button type="button" className={tab === 'create' ? 'hg-tab-active' : ''} onClick={() => setTab('create')}>创建牌桌</button><button type="button" className={tab === 'join' ? 'hg-tab-active' : ''} onClick={() => setTab('join')}>邀请码加入</button></div></div>
        <p className="hg-muted">当前牌组：{selectedDraft?.name || deck?.name || '加载中'} · <a href="#hegemony-decks">更换牌组 ↓</a></p>
        {selectedDraft && <p className="hg-muted">入席使用上次选择的牌组内容；编辑后请重新保存并选择。</p>}
        {selectedDraft && !draftEntryAvailable && <p className="hg-library-warning" role="alert">草稿已保存，当前入口尚未支持自定义牌组{tab === 'join' ? '加入' : '创建'}；可以改选原预组。</p>}
        {selectedDraft && !draftValid && <p className="hg-library-warning" role="alert">已选择草稿需要按当前卡池重新校验，暂不可入席。</p>}
        <label className="hg-field">你的称呼<input value={name} onChange={event => setName(event.target.value)} maxLength={24} placeholder="让牌桌上的伙伴认出你" required autoComplete="nickname" disabled={busy} /></label>
        {tab === 'create' ? <div className="hg-mode-select" role="group" aria-label="对战模式">
          <button type="button" className={mode === 'duel' ? 'hg-selected' : ''} onClick={() => setMode('duel')} disabled={busy}><strong>两人对决 <span>1 VS 1</span></strong><small>3 个地区 · 先获 8 分胜利</small></button>
          <button type="button" className={mode === 'teams' ? 'hg-selected' : ''} onClick={() => setMode('teams')} disabled={busy}><strong>四人协作 <span>2 VS 2</span></strong><small>5 个地区 · 团队先获 10 分胜利</small></button>
        </div> : <label className="hg-field">邀请码<input value={invite} onChange={event => setInvite(event.target.value)} placeholder="粘贴伙伴发来的邀请码" required autoComplete="off" disabled={busy} /></label>}
        <div className="hg-entry-bottom"><p>创建后分享邀请，所有玩家准备后由房主开始。<br />座位保存在此浏览器，刷新即可继续。{tab === 'join' && <><br />本机已有此房座位时会恢复原席，保留原称呼、牌组与准备状态。</>}</p><button className="hg-button hg-button-primary" type="submit" disabled={busy || !catalog || !entryAvailable || !name.trim() || (tab === 'join' && !invite.trim())}>{busy ? '正在入席…' : tab === 'create' ? '创建牌桌 →' : '加入牌桌 →'}</button></div>
      </form>
      <div id="hegemony-decks" className="hg-section-title"><span className="hg-eyebrow">02 / 选择自组预组</span><span className="hg-pool-badge">受限真实卡池预组</span></div>
      <h2>{catalog ? `${catalog.decks.length} 套自组预组` : '自组预组'}</h2>
      <p className="hg-muted">每套 50 张，来自已开放的真实卡牌。当前为受限卡池自组预组，并非官方预组。</p>
      {catalog ? <><DeckPicker decks={catalog.decks} cards={catalog.cards} value={selectedDraft ? '' : selected} onChange={id => { setDeckId(id); setSelectedDraft(null); }} disabled={busy} />
        {!selectedDraft && deck && <details className="hg-deck-list"><summary>查看「{deck.name}」的 {deck.cardCount} 张组成</summary><ul>{deck.cards.map(item => <li key={item.cardId}><span>{catalog.cards.find(card => card.id === item.cardId)?.name || item.cardId}</span><b>× {item.count}</b></li>)}</ul></details>}
      </> : <div className="hg-loading"><span>正在连接牌组档案…</span><button className="hg-button hg-button-quiet" onClick={retry}>重新连接</button></div>}
      <FactionCoverage catalog={catalog} />
      {catalog && <DeckLibrary catalog={catalog} disabled={busy} selectedDraftId={selectedDraft?.id} onSelectDraft={draft => { setSelectedDraft(publicDeckDraft(draft)); onSelectDraft?.(publicDeckDraft(draft)); }} />}
    </section>
  </main>;
}
