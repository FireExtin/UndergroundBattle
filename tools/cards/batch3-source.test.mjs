import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const read = name => readFileSync(new URL('../../' + name, import.meta.url));
const pool = JSON.parse(read('rust-game/data/cards.json'));
const specs = JSON.parse(read('docs/factions/card-specifications.json')).cards;
const plan = JSON.parse(read('docs/factions/batch20-third-2026-10-21.json'));
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const hash = value => createHash('sha256').update(value).digest('hex');
const digest = value => hash(JSON.stringify(canonical(value)));

test('third batch excludes all 193 preceding definitions and preserves every preceding record', () => {
  // Independently pinned from bb77, before any third-batch data was appended.
  assert.equal(digest(pool.cards.slice(0, 193)), '0e85e0784acef129cea818f4c2fb694912070fbb17bde53b61d52d2fb2def01e');
  assert.equal(plan.cards.length, 20);
  assert.equal(new Set(plan.cards.map(c => c.id)).size, 20);
  const prior = new Set(pool.cards.slice(0, 193).map(c => c.id));
  const candidateAdmitted = plan.scope.selectionEnabled === 20;
  assert([0,20].includes(plan.scope.selectionEnabled),'Admit the entire batch together');
  if(candidateAdmitted) assert(plan.cards.every(c=>c.runtime==='batchIntegratedPendingPublication'),'A whole local candidate needs every card integrated');
  const colors = {};
  for (const card of plan.cards) {
    assert(!prior.has(card.id), card.id);
    colors[card.color] = (colors[card.color] || 0) + 1;
    assert.equal(card.selectionEnabled, candidateAdmitted, card.id);
    assert.equal(hash(read(card.sourceImage)), card.sourceImageSha256, card.id);
    assert.equal(card.sourceImageSha256, specs[card.id].sourceVerification.imageSha256, card.id);
  }
  assert.deepEqual(colors, { 红: 4, 黑: 5, 蓝: 6, 灰: 5 });
  for (const card of pool.cards.slice(193, 213)) {
    assert(plan.cards.some(c => c.id === card.id), card.id);
    assert.equal(card.supported, candidateAdmitted, card.id);
  }
  assert.equal(plan.scope.published, 0);
  assert(!plan.cards.some(c=>c.id==='CQ11'));
  assert(plan.deferredCards.some(c=>c.id==='CQ11'&&c.runtime==='notImplemented'));
  assert(!pool.cards.some(c=>c.id==='CQ11'||c.id==='TK008'));
});

test('new original-image corrections retain sun and moon gates and exact tap cost', () => {
  assert.deepEqual(specs.BQ042.fields.printedKeywords, ['隐秘条件：灵体', '时间标志1']);
  assert.deepEqual(specs.BQ087.fields.printedKeywords, ['暴露条件：领袖', '隐秘条件：传闻']);
  assert.deepEqual(specs.BQ087.fields.domains, [{ id: 'death', count: 1 }]);
  assert.match(specs.BQ087.fields.printedRuleTextLines[1], /^准备步骤强制触发/);
  assert.deepEqual(specs.BQ029.fields.printedKeywords, ['暴露条件：非资产', '袭击1']);
  assert.deepEqual(specs.WM041.fields.printedKeywords, ['领袖', '暴露条件：声望']);
  assert.match(specs.XG26.fields.printedRuleTextLines[2], /横置看场打手/);
  assert(!specs.XG26.fields.printedRuleTextLines.join('').includes('牺牲看场打手'));
  for (const id of ['BQ042', 'BQ087', 'BQ029', 'WM041', 'XG26']) {
    assert.equal(specs[id].sourceVerification.evidenceId, `batch20-third-${id.toLowerCase()}-original-20261021`);
  }
});
