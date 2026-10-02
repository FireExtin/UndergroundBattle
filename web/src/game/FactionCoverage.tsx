import { factionCoverage } from './factions';
import type { Catalog } from './types';

export function FactionCoverage({ catalog }: { catalog: Catalog | null }) {
  const coverage = factionCoverage(catalog?.cards || []);
  const unavailable = coverage.factions.filter(({ count }) => !count).map(({ faction }) => faction.color).join('、');
  return <details className="hg-faction-coverage">
    <summary><span>官方八色派系</span><small>{catalog ? `${coverage.playerCount} 种玩家牌${unavailable ? ` · ${unavailable}未实现` : ''} · 查看覆盖` : '开放数量加载中'}</small></summary>
    <p>派系是卡牌颜色；现有自组预组可含多个派系与中立牌。秘社牌和自由构筑尚未开放。</p>
    <ul aria-label="官方八色派系覆盖">{coverage.factions.map(({ faction, count }) => <li key={faction.id} className={`hg-faction-${faction.id}`} data-faction-id={faction.id} data-open-card-count={catalog ? count : undefined}>
      <span className="hg-faction-name"><i className="hg-faction-swatch" aria-hidden="true" />{faction.color} · {faction.name}</span>
      <span>{catalog ? `${count} 种玩家牌` : '加载中'}</span><small>{!catalog ? '等待目录' : count ? '部分开放' : '未实现'}</small>
    </li>)}</ul>
    <p className="hg-neutral-note"><i className="hg-faction-swatch hg-faction-neutral" aria-hidden="true" />褐 · 中立（无派系）：{catalog ? `${coverage.neutralCount} 种通用玩家牌` : '加载中'}，另列，不是第九个派系。</p>
    {!!coverage.unknownCount && <p>{coverage.unknownCount} 种玩家牌暂未标示颜色。</p>}
  </details>;
}
