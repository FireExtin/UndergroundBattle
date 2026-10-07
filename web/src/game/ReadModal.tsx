import { useRef, useState } from 'react';
import { CardContent, visibleCard } from './CardTile';
import { cardScanUrl } from './cardScans';
import type { Card, CardDefinition } from './types';
import { useDialogFocus } from './useDialogFocus';

export function ReadModal({ card, definition, viewerId, onClose, context, active = true }: { card: Card; definition?: CardDefinition; viewerId: string; onClose: () => void; context?: string; active?: boolean }) {
  const modal = useRef<HTMLElement>(null);
  useDialogFocus(modal, active, onClose);
  const safe = visibleCard(card, viewerId);
  const scan = cardScanUrl(safe.cardId);
  const [showScan, setShowScan] = useState(false);
  return <div className="hg-modal-backdrop" onClick={onClose}><section ref={modal} tabIndex={-1} className="hg-modal hg-reading" data-reading-card={safe.instanceId} role="dialog" aria-modal="true" aria-label={`放大阅读${safe.name}`} onClick={event => event.stopPropagation()}>
    <button className="hg-close" onClick={onClose} aria-label="关闭放大阅读">×</button><span className="hg-eyebrow">卡牌档案 · 放大阅读</span>
    {context && <p className="hg-attachment-context">{context}</p>}
    {safe.kind === 'sealed' && <p className="hg-attachment-context">此牌已封印，当前视为空白且在场外。原始牌面仅供阅读，不提供当前能力或数值。</p>}
    {scan && <div className="hg-reading-tabs" role="group" aria-label="阅读内容"><button type="button" className="hg-button hg-button-quiet" aria-pressed={!showScan} onClick={() => setShowScan(false)}>当前状态与文字</button><button type="button" className="hg-button hg-button-quiet" aria-pressed={showScan} onClick={() => setShowScan(true)}>原始牌面</button></div>}
    {showScan && scan ? <div className="hg-source-reading"><a href={scan} target="_blank" rel="noreferrer" aria-label={`打开${safe.name}原始牌面全图`}><img src={scan} alt={`${safe.name}原始牌面`} /></a><p>{safe.kind === 'sealed' ? '原图中的印刷能力和数值在封印期间不生效。' : '点击牌面打开原图。印刷值请结合当前状态中的修正查看。'}</p></div> : <><div className="hg-reading-card"><CardContent card={safe} definition={safe.cardId ? definition : undefined} /></div>{safe.kind !== 'society' && safe.kind !== 'sealed' && <p>◈ 调查 · ⚔ 战斗 · ⚑ 势力。{safe.convertedTemporaryIcons && !safe.faceDown ? '本地区具现化将标示的临时图标转为永久，不受先手限制。' : '白底图标仅在操控者团队持先手时生效。'}</p>}</>}
  </section></div>;
}
