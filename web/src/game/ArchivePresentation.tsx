import { createContext, useContext, useState } from 'react';
import { cardScanUrl } from './cardScans';
import type { Card } from './types';

// Only the table provides this presentation; the room protocol is unchanged.
export const ArchivePresentation = createContext(false);
export const useArchivePresentation = () => useContext(ArchivePresentation);

export function ArchiveArtwork({ card }: { card: Card }) {
  const [failed, setFailed] = useState(false);
  // Concealed thumbnails always use a back, including the controller's thumbnail.
  // Assets reveal resources, never their underlying printed-card identity.
  const scan = !card.faceDown && card.kind !== 'asset' && cardScanUrl(card.cardId);
  return <span className={`hg-archive-art${card.faceDown ? ' hg-archive-back' : ''}${card.kind === 'region' ? ' hg-archive-world-art' : ''}${card.kind === 'asset' ? ' hg-archive-asset-art' : ''}${card.kind === 'society' ? ' hg-archive-society-art' : ''}`} aria-hidden="true" data-art-card={scan && !failed ? card.cardId : undefined}>
    {scan && !failed ? <img src={scan} alt="" decoding="async" loading="lazy" draggable={false} onError={() => setFailed(true)} />
      : <span className="hg-archive-sigil">{card.faceDown ? '◈' : card.kind === 'asset' ? '资' : '⌖'}</span>}
  </span>;
}
