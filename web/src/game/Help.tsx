import type { Catalog } from './types';
import { FactionCoverage } from './FactionCoverage';

export function Help({ onClose, catalog }: { onClose: () => void; catalog: Catalog | null }) {
  const worldCount = new Set(catalog?.cards.filter(card => card.kind === 'region').map(card => card.id)).size;
  const deckSizes = [...new Set(catalog?.decks.map(deck => deck.cardCount))];
  const poolDescription = catalog
    ? `使用 ${catalog.decks.length} 套${deckSizes.length === 1 ? ` ${deckSizes[0]} 张` : ''}自组预组与 ${worldCount} 种已开放世界牌。`
    : '牌组档案正在加载。';
  return <div className="hg-modal-backdrop" onClick={onClose}><section className="hg-help hg-modal" role="dialog" aria-modal="true" aria-label="上手指南" onClick={event => event.stopPropagation()}>
    <button className="hg-close" onClick={onClose} aria-label="关闭指南">×</button><span className="hg-eyebrow">新手档案 / HOW TO PLAY</span><h2>将世界纳入你的版图</h2>
    <p>争夺地区，把赢得的地区收入计分区。两人对决先得 <b>8 分</b>，四人协作团队先得 <b>10 分</b>。</p>
    <ol className="hg-help-steps"><li><b>建立你的资产</b><span>将手牌置为资产，提供费用与忠诚；派遣角色到地区，参与对抗。点选一张牌即可查看本时点可用的行动。</span></li>
      <li><b>轮流行动，让过推进</b><span>牌桌显示行动团队和当前优先权。四人协作中，全队让过才交出行动；任一队员行动会重置本队让过记录。</span></li>
      <li><b>调查 → 战斗 → 势力</b><span>双方比较参与角色的图标，获胜方获得图标差值 X 的奖励：调查排序牌库顶 X 张后抓 1 张，战斗分配 X 点伤害，势力放置 X 点影响。横置角色不参与对抗。</span></li>
      <li><b>赢得地区</b><span>势力先抵消对方影响，再增加差值。达到地区阈值后还有快速行动窗口；地区上的牌由各拥有者排序置于牌库底，再补充新地区。</span></li></ol>
    <div className="hg-help-notes"><p><b>白底图标</b>只在操控者团队持先手时生效。每团队每回合一次先手特权，使用者必须贡献至少一个图标。</p><p><b>四人位置</b>席位 1 / 3 可正常派遣到左侧三个地区，席位 2 / 4 到右侧三个地区，中间地区共享。所有可执行动作由牌桌实时给出。</p><p><b>初始手牌</b>为 6 张，可再调度一次：暂放任选手牌，抓等量，再将原牌洗回牌库。</p><p><b>当前卡池</b>是逐卡开放的受限真实卡牌池，{poolDescription}</p></div>
    <FactionCoverage catalog={catalog} />
    <button className="hg-button hg-button-primary" onClick={onClose}>回到牌桌</button>
  </section></div>;
}
