import { useEffect, useRef } from 'react';
import { CardContent, visibleCard } from './CardTile';
import type { Card, CardDefinition } from './types';

export function ReadModal({ card, definition, viewerId, onClose }: { card: Card; definition?: CardDefinition; viewerId: string; onClose: () => void }) {
  const close = useRef<HTMLButtonElement>(null);
  const dismiss = useRef(onClose);
  dismiss.current = onClose;
  const safe = visibleCard(card, viewerId);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    close.current?.focus();
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') dismiss.current();
      if (event.key === 'Tab') { event.preventDefault(); close.current?.focus(); }
    };
    document.addEventListener('keydown', escape);
    return () => { document.removeEventListener('keydown', escape); previous?.focus(); };
  }, []);
  return <div className="hg-modal-backdrop" onClick={onClose}><section className="hg-modal hg-reading" data-reading-card={safe.instanceId} role="dialog" aria-modal="true" aria-label={`放大阅读${safe.name}`} onClick={event => event.stopPropagation()}>
    <button ref={close} className="hg-close" onClick={onClose} aria-label="关闭放大阅读">×</button><span className="hg-eyebrow">卡牌档案 · 放大阅读</span>
    <div className="hg-reading-card"><CardContent card={safe} definition={safe.cardId ? definition : undefined} /></div>
    <p>◈ 调查 · ⚔ 战斗 · ⚑ 势力。白底图标仅在操控者团队持先手时生效。</p>
  </section></div>;
}
