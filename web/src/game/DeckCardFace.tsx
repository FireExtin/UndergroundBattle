import { useState } from 'react';
import { cardScanUrl } from './cardScans';
import type { CardDefinition } from './types';

/** Original artwork is presentation only; the supplied catalog controls admission. */
export function DeckCardFace({ card, large = false }: { card: CardDefinition; large?: boolean }) {
  const source = cardScanUrl(card.id);
  const [failedSource, setFailedSource] = useState<string>();
  const available = source && failedSource !== source;
  return <span className="hg-library-face" data-deck-face={card.id} data-face-state={available ? 'original' : 'missing'}>
    {available ? <img src={source} alt={`${card.name}原始牌面`} loading={large ? 'eager' : 'lazy'} decoding="async" draggable={false} onError={() => setFailedSource(source)} />
      : <span className="hg-library-face-placeholder"><span aria-hidden="true">⌖</span><strong>{card.name}</strong><small>{card.id}</small><span>暂无原始牌面</span><small>可查看目录规则文字</small></span>}
  </span>;
}
