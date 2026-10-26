// Additive static source/catalog WIP check only. Existing historical gates
// remain unchanged. This never evaluates abilities or authorizes gameplay.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
export const BQ001_IMAGE_SHA256 = 'c2c6de0b1d71e37a913c9b2e2c358a0e28702037d6983430bc7bc10a9a1b78a1';
// Original pre-three-admission 277-record digest, preserved verbatim.
const PRIOR_277_SHA256 = '0876590922afda7f522d4594a5e0d689266b0e325acdf191ef1ce915307e6c91';
const ADMITTED = ['BQ034', 'CQ25', 'XG25'];
const canonical = v => Array.isArray(v) ? v.map(canonical) : v && typeof v === 'object'
  ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canonical(v[k])])) : v;
export function assertBq001SourceWip(pool) {
  assert.equal(pool.rulesVersion, 'hegemony-pdf-v1');
  assert.equal(pool.cardPoolVersion, 'limited-v2.104-bq001-anchor-source-wip');
  assert.equal(pool.engineVersion, 'rust-v0.2.113-bq001-anchor-source-wip');
  assert.equal(pool.cards.length, 278);
  assert.equal(new Set(pool.cards.map(c => c.id)).size, 278);
  assert.equal(pool.cards.at(-1).id, 'BQ001');
  for (const id of ['BQ001', 'BQ009', 'BQ033']) assert.equal(pool.cards.find(c => c.id === id)?.supported, false, `${id} remains closed`);
  assert.equal(pool.cards.filter(c => c.supported !== false).length, 236);
  const previous = structuredClone(pool.cards.slice(0, 277));
  for (const id of ADMITTED) { const c = previous.find(c => c.id === id); assert.equal(c?.supported, true); c.supported = false; }
  assert.equal(createHash('sha256').update(JSON.stringify(canonical(previous))).digest('hex'), PRIOR_277_SHA256);
  const card = pool.cards.at(-1);
  const keys = ['id','name','subtitle','kind','cost','loyalty','color','magic','subtypes','text','icons','defense','keywords','unique','supported'];
  assert.deepEqual(Object.fromEntries(keys.map(k => [k, card[k]])), {
    id: 'BQ001', name: '脑叶专家', subtitle: null, kind: 'character', cost: 1,
    loyalty: ['黄色'], color: '黄', magic: '心灵', subtypes: ['人类','法师','学者'],
    text: '持续 在进行隐秘对抗时，你所展示的牌获得隐秘+X，X等于封印在本方秘社上的名为精神锚点之卡牌的数量。\n现身触发 从你手中选择一张星辰领域的牌，将其封印在本方秘社上，然后抓一张牌。',
    icons: {permanent: {investigation: 0, combat: 0, influence: 1}, temporary: {investigation: 0, combat: 0, influence: 0}},
    defense: 1, keywords: [], unique: false, supported: false,
  });
  assert.equal(card.source.imagePath, 'resource/ymsj-fun.github.io/cards/BQ001 脑叶专家.jpg');
  assert.equal(card.source.imageSha256, BQ001_IMAGE_SHA256);
  assert.equal(card.source.printedCollectorCode, '01/48');
}
