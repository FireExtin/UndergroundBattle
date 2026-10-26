import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {assertBq001SourceWip, BQ001_IMAGE_SHA256} from './bq001-source-contract.mjs';
const read = p => readFileSync(new URL('../../' + p, import.meta.url));
const pool = JSON.parse(read('rust-game/data/cards.json'));
const sha = b => createHash('sha256').update(b).digest('hex');
test('independent BQ001 source WIP contract pins all printed fields and closed admission', () => {
  assertBq001SourceWip(pool);
});
test('BQ001 original and served pixels retain the same image identity without rewriting historical prose', () => {
  const c = pool.cards.at(-1);
  assert.equal(sha(read(c.source.imagePath)), BQ001_IMAGE_SHA256);
  assert.equal(sha(read('web/public/cards/BQ001.jpg')), BQ001_IMAGE_SHA256);
  const spec = JSON.parse(read('docs/factions/card-specifications.json'));
  assert.equal(spec.cards.BQ001.sourceVerification.imageSha256, BQ001_IMAGE_SHA256);
  assert.equal(c.text.split('\n')[0], '持续 在进行隐秘对抗时，你所展示的牌获得隐秘+X，X等于封印在本方秘社上的名为精神锚点之卡牌的数量。');
});
test('the additive contract rejects altered admission, symbols, legacy cards and unapproved expansion', async t => {
  const cases = [
    ['open BQ001', p => { p.cards.at(-1).supported = true; }],
    ['open BQ009', p => { p.cards.find(c => c.id === 'BQ009').supported = true; }],
    ['open BQ033', p => { p.cards.find(c => c.id === 'BQ033').supported = true; }],
    ['reuse ABI112', p => { p.engineVersion = 'rust-v0.2.112-shared-keyword-rewards'; }],
    ['regional instead of mystic bonus', p => { p.cards.at(-1).text = p.cards.at(-1).text.replace('隐秘+X', '地区势力+X'); }],
    ['change printed icon', p => { p.cards.at(-1).icons.permanent.influence = 2; }],
    ['change reveal cost', p => { p.cards.at(-1).cost = 0; }],
    ['change image identity', p => { p.cards.at(-1).source.imageSha256 = '0'.repeat(64); }],
    ['change legacy text', p => { p.cards[0].name += 'changed'; }],
    ['close an admitted card', p => { p.cards.find(c => c.id === 'BQ034').supported = false; }],
    ['append another yellow card', p => { p.cards.push({...p.cards.at(-1), id: 'BQ003'}); }],
    ['reuse a card id', p => { p.cards.at(-1).id = p.cards[0].id; }],
  ];
  for (const [name, mutate] of cases) await t.test(name, () => {
    const candidate = structuredClone(pool); mutate(candidate);
    assert.throws(() => assertBq001SourceWip(candidate));
  });
});
