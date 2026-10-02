import { useEffect, useRef, useState } from 'react';
import { CardContent, visibleCard } from './CardTile';
import { cardScanUrl } from './cardScans';
import type { Card, CardDefinition } from './types';

export function ReadModal({ card, definition, viewerId, onClose, context }: { card: Card; definition?: CardDefinition; viewerId: string; onClose: () => void; context?: string }) {
  const close = useRef<HTMLButtonElement>(null);
  const modal = useRef<HTMLElement>(null);
  const dismiss = useRef(onClose);
  dismiss.current = onClose;
  const safe = visibleCard(card, viewerId);
  const scan = cardScanUrl(safe.cardId);
  const [showScan, setShowScan] = useState(false);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    close.current?.focus();
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') dismiss.current();
      if (event.key === 'Tab') {
        const controls = Array.from(modal.current?.querySelectorAll<HTMLElement>('button:not(:disabled), a[href]') || []);
        const current = controls.indexOf(document.activeElement as HTMLElement);
        event.preventDefault();
        controls[(current + (event.shiftKey ? -1 : 1) + controls.length) % controls.length]?.focus();
      }
    };
    document.addEventListener('keydown', escape);
    return () => { document.removeEventListener('keydown', escape); previous?.focus(); };
  }, []);
  return <div className="hg-modal-backdrop" onClick={onClose}><section ref={modal} className="hg-modal hg-reading" data-reading-card={safe.instanceId} role="dialog" aria-modal="true" aria-label={`放大阅读${safe.name}`} onClick={event => event.stopPropagation()}>
    <button ref={close} className="hg-close" onClick={onClose} aria-label="关闭放大阅读">×</button><span className="hg-eyebrow">卡牌档案 · 放大阅读</span>
    {context && <p className="hg-attachment-context">{context}</p>}
    {scan && <div className="hg-reading-tabs" role="group" aria-label="阅读内容"><button type="button" className="hg-button hg-button-quiet" aria-pressed={!showScan} onClick={() => setShowScan(false)}>当前状态与文字</button><button type="button" className="hg-button hg-button-quiet" aria-pressed={showScan} onClick={() => setShowScan(true)}>原始牌面</button></div>}
    {showScan && scan ? <div className="hg-source-reading"><a href={scan} target="_blank" rel="noreferrer" aria-label={`打开${safe.name}原始牌面全图`}><img src={scan} alt={`${safe.name}原始牌面`} /></a><p>点击牌面打开原图。印刷值请结合当前状态中的修正查看。</p></div> : <><div className="hg-reading-card"><CardContent card={safe} definition={safe.cardId ? definition : undefined} /></div><p>◈ 调查 · ⚔ 战斗 · ⚑ 势力。白底图标仅在操控者团队持先手时生效。</p></>}
  </section></div>;
}
