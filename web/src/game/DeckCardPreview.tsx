import { useEffect, useLayoutEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { IconStrip } from './CardTile';
import { cardScanUrl } from './cardScans';
import { DeckCardFace } from './DeckCardFace';
import type { DeckCardDefinition, DeckIssue, DeckValidation } from './deckLibrary';
import { deckCardAdmission, isEditableDeckCard } from './deckLibrary';
import { useDialogFocus } from './useDialogFocus';

export function DeckCardPreview({ card, count, disabled, copyLabel, sameNameCount, issues, validation, navigation, onChangeCount, onClose }: {
  card: DeckCardDefinition; count: number; disabled: boolean; copyLabel: string;
  sameNameCount: number; issues: DeckIssue[]; validation: DeckValidation;
  navigation: { index: number; total: number; previous: () => void; next: () => void };
  onChangeCount: (delta: number) => void; onClose: () => void;
}) {
  const dialog = useRef<HTMLElement>(null);
  const focusedControl = useRef<HTMLElement | null>(null);
  const editable = isEditableDeckCard(card);
  const admission = deckCardAdmission(card);
  useDialogFocus(dialog, true, onClose);
  useLayoutEffect(() => { if (dialog.current) dialog.current.scrollTop = 0; }, [card.id]);
  useLayoutEffect(() => {
    const previous = focusedControl.current;
    if (!previous || (previous.isConnected && !previous.matches(':disabled'))) return;
    // Browsers blur a button when its last decrement or navigation disables it.
    const group = previous.closest('.hg-library-preview-navigation, .hg-library-preview-actions');
    const next = (group?.isConnected ? group.querySelector<HTMLButtonElement>('button:not(:disabled)') : null)
      || dialog.current?.querySelector<HTMLButtonElement>('button:not(:disabled)') || dialog.current;
    next?.focus({ preventScroll: true });
  }, [card.id, count, disabled, editable, navigation.index, navigation.total]);
  useEffect(() => {
    // An expanded workspace already owns the body scroll lock.
    if (document.body.style.overflow === 'hidden') return;
    const previous = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = previous; };
  }, []);
  const source = cardScanUrl(card.id);
  return createPortal(<div className="hg-library-preview-backdrop" onClick={onClose}>
    <section className="hg-library-preview" ref={dialog} tabIndex={-1} role="dialog" aria-modal="true" aria-label={`卡面预览 ${card.name}（${card.id}）`} onClick={event => event.stopPropagation()} onFocusCapture={event => { if (event.target instanceof HTMLElement) focusedControl.current = event.target; }}
      onKeyDown={event => {
        if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
        if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
          event.preventDefault(); event.stopPropagation();
          if (event.key === 'ArrowLeft') navigation.previous(); else navigation.next();
        }
      }}>
      <div className="hg-library-preview-toolbar">
      <div className="hg-library-preview-heading"><span className="hg-eyebrow">原始牌面 · 卡牌档案</span><button type="button" className="hg-button hg-button-quiet" onClick={onClose} aria-label="关闭卡面预览">关闭 ×</button></div>
      {navigation.total > 1 && <nav className="hg-library-preview-navigation" aria-label="卡面浏览">
        <button type="button" className="hg-button hg-button-quiet" aria-label="上一张卡面" disabled={navigation.index <= 0} onClick={navigation.previous}>← 上一张</button>
        <span>{navigation.index + 1} / {navigation.total}<small>← → 浏览 · Esc 返回</small></span>
        <button type="button" className="hg-button hg-button-quiet" aria-label="下一张卡面" disabled={navigation.index >= navigation.total - 1} onClick={navigation.next}>下一张 →</button>
      </nav>}
      </div>
      <div className="hg-library-preview-content">
        <div><DeckCardFace key={card.id} card={card} large />{source && <a className="hg-library-original-link" href={source} target="_blank" rel="noreferrer">打开原图 ↗</a>}</div>
        <div className="hg-library-preview-details">
          <h2>{card.name}</h2>{card.subtitle && <p>{card.subtitle}</p>}
          <p className={`hg-card-admission hg-admission-${admission.status}`} data-card-admission={admission.status}>卡牌准入：{admission.label} · {admission.reason}</p>
          <p>{card.id} · {({ character: '角色', spell: '事务', event: '事务（旧目录）', attachment: '附属', region: '地区', society: '秘社' }[card.kind] || card.kind)}{card.subtypes?.length ? ` · ${card.subtypes.join(' / ')}` : card.type ? ` · ${card.type}` : ''}</p>
          {card.kind !== 'society' && card.kind !== 'region' && <p>印刷费用 {card.cost} · {card.color || '颜色未标'}{card.magic ? ` · ${card.magic}` : ''}</p>}
          {(card.icons?.permanent || card.permanentIcons) && <p>永久图标 <IconStrip icons={card.icons?.permanent || card.permanentIcons} /></p>}
          {(card.icons?.temporary || card.temporaryIcons) && <p>先手图标 <IconStrip icons={card.icons?.temporary || card.temporaryIcons} temporary /></p>}
          {typeof card.defense === 'number' && <p>印刷防御 {card.defense}</p>}
          {card.kind !== 'society' && card.kind !== 'region' && <p>{card.loyaltyText || (card.loyalty?.length ? `忠诚 ${card.loyalty.join(' / ')}` : '无忠诚要求')}</p>}
          <p className="hg-library-card-text">{card.text || '目录未提供额外规则文字。'}</p>
          <p>{copyLabel}</p>
          <div className="hg-library-preview-actions"><button type="button" className="hg-button hg-button-quiet" aria-label={`预览减少 ${card.name}（${card.id}）`} disabled={disabled || count === 0} onClick={() => onChangeCount(-1)}>− 减少</button><strong role="status">已加入 {count} 张</strong><button type="button" className="hg-button hg-button-primary" aria-label={`预览添加 ${card.name}（${card.id}）`} disabled={disabled || !editable} onClick={() => onChangeCount(1)}>+ 添加</button></div>
          <div className="hg-library-preview-validation" aria-live="polite" aria-atomic="true">
            {sameNameCount !== count && <p>其他编号也有此牌：同名合计 {sameNameCount} 张。</p>}
            {issues.length > 0 && <ul className="hg-library-issues" aria-label="此卡构筑提示">{issues.map((issue, index) => <li key={`${issue.code}-${index}`}>{issue.message}</li>)}</ul>}
            <p className={validation.valid ? 'hg-library-notice' : 'hg-muted'}>当前牌组共 {validation.total} 张 · {validation.valid ? '构筑校验通过' : '暂不可开局'}</p>
            {validation.issues.length > 0 && <details><summary>查看牌组的 {validation.issues.length} 项构筑提示</summary><ul className="hg-library-issues">{validation.issues.map((issue, index) => <li key={`${issue.code}-${index}`}>{issue.message}</li>)}</ul></details>}
          </div>
          <small>构筑校验以当前目录为准；对局中的费用与效果由服务端裁定。</small>
        </div>
      </div>
    </section>
  </div>, document.body);
}
