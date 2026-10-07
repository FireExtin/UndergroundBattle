import { instanceTag } from './CardTile';
import type { Card, View } from './types';
import './sealed-cards.css';

/** Public reading only. Sealed objects do not join the board's action/target controls. */
export function SealedCards({ view, onRead }: { view: View; onRead: (card: Card) => void }) {
  if (!view.sealedCards?.length) return null;
  const name = (id: string) => view.players.find(player => player.id === id)?.name || '玩家';
  return <section className="hg-sealed-cards" aria-label="场外封印牌">
    <strong>封印牌 · {view.sealedCards.length} 张</strong>
    <p>公开、场外、空白；点击阅读。载体离场或翻暗时回到各自拥有者手牌。</p>
    <div>{view.sealedCards.map(card => {
      const region = view.regions.find(item => item.characters.some(host => host.instanceId === card.hostId));
      const host = region?.characters.find(item => item.instanceId === card.hostId);
      return <button type="button" key={card.instanceId} data-sealed-instance={card.instanceId} data-sealed-host={card.hostId} onClick={() => onRead(card)} aria-label={`阅读封印牌${card.name}`}>
        <strong>{card.name} <small>#{instanceTag(card.instanceId)}</small></strong>
        <span>{name(card.owner)} 拥有 · 已封印</span>
        <small>{host ? `载体：${host.name} #${instanceTag(host.instanceId)} · ${name(host.controller)} 操控 · 地区 ${(region?.index ?? 0) + 1}` : '原载体未在当前视图中显示'}</small>
      </button>;
    })}</div>
  </section>;
}
