import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { IconStrip } from './CardTile';
import { cardScanUrl } from './cardScans';
import { DeckCardFace } from './DeckCardFace';
import type { DeckCardDefinition } from './deckLibrary';
import { isEditableDeckCard } from './deckLibrary';
import { useDialogFocus } from './useDialogFocus';

export function DeckCardPreview({ card, count, disabled, copyLabel, onChangeCount, onClose }: {
  card: DeckCardDefinition; count: number; disabled: boolean; copyLabel: string;
  onChangeCount: (delta: number) => void; onClose: () => void;
}) {
  const dialog = useRef<HTMLElement>(null);
  useDialogFocus(dialog, true, onClose);
  useEffect(() => {
    // An expanded workspace already owns the body scroll lock.
    if (document.body.style.overflow === 'hidden') return;
    const previous = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = previous; };
  }, []);
  const source = cardScanUrl(card.id);
  return createPortal(<div className="hg-library-preview-backdrop" onClick={onClose}>
    <section className="hg-library-preview" ref={dialog} tabIndex={-1} role="dialog" aria-modal="true" aria-label={`卡面预览 ${card.name}（${card.id}）`} onClick={event => event.stopPropagation()}>
      <div className="hg-library-preview-heading"><span className="hg-eyebrow">原始牌面 · 卡牌档案</span><button type="button" className="hg-button hg-button-quiet" onClick={onClose} aria-label="关闭卡面预览">关闭 ×</button></div>
      <div className="hg-library-preview-content">
        <div><DeckCardFace key={card.id} card={card} large />{source && <a className="hg-library-original-link" href={source} target="_blank" rel="noreferrer">打开原图 ↗</a>}</div>
        <div className="hg-library-preview-details">
          <h2>{card.name}</h2>{card.subtitle && <p>{card.subtitle}</p>}
          <p>{card.id} · {({ character: '角色', spell: '事务', event: '事务（旧目录）', attachment: '附属', region: '地区', society: '秘社' }[card.kind] || card.kind)}{card.subtypes?.length ? ` · ${card.subtypes.join(' / ')}` : card.type ? ` · ${card.type}` : ''}</p>
          {card.kind !== 'society' && card.kind !== 'region' && <p>印刷费用 {card.cost} · {card.color || '颜色未标'}{card.magic ? ` · ${card.magic}` : ''}</p>}
          {(card.icons?.permanent || card.permanentIcons) && <p>永久图标 <IconStrip icons={card.icons?.permanent || card.permanentIcons} /></p>}
          {(card.icons?.temporary || card.temporaryIcons) && <p>先手图标 <IconStrip icons={card.icons?.temporary || card.temporaryIcons} temporary /></p>}
          {typeof card.defense === 'number' && <p>印刷防御 {card.defense}</p>}
          {card.kind !== 'society' && card.kind !== 'region' && <p>{card.loyaltyText || (card.loyalty?.length ? `忠诚 ${card.loyalty.join(' / ')}` : '无忠诚要求')}</p>}
          <p className="hg-library-card-text">{card.text || '目录未提供额外规则文字。'}</p>
          <p>{copyLabel}</p>
          <div className="hg-library-preview-actions"><button type="button" className="hg-button hg-button-quiet" aria-label={`预览减少 ${card.name}（${card.id}）`} disabled={disabled || count === 0} onClick={() => onChangeCount(-1)}>− 减少</button><strong role="status">已加入 {count} 张</strong><button type="button" className="hg-button hg-button-primary" aria-label={`预览添加 ${card.name}（${card.id}）`} disabled={disabled || !isEditableDeckCard(card)} onClick={() => onChangeCount(1)}>+ 添加</button></div>
          {!isEditableDeckCard(card) && <p className="hg-library-warning">此卡不可加入玩家牌组。请在目录指定的入口选择秘社或查看地区。</p>}
          <small>构筑校验以当前目录为准；对局中的费用与效果由服务端裁定。</small>
        </div>
      </div>
    </section>
  </div>, document.body);
}
