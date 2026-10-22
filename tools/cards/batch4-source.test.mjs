import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const read = name => readFileSync(new URL('../../' + name, import.meta.url));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const digest = value => hash(JSON.stringify(canonical(value)));
const pool = JSON.parse(read('rust-game/data/cards.json'));
const specs = JSON.parse(read('docs/factions/card-specifications.json')).cards;
const plan = JSON.parse(read('docs/factions/batch20-fourth-2026-10-22.json'));

test('fourth roster preserves frozen213 and keeps native pilots separate from public admission', () => {
  assert.equal(plan.baseCommit, 'b80f12202799a632ee9d75f40da8352f2ca274f1');
  assert.equal(digest(pool.cards.slice(0, 213)), '5ddc50185c92d6ba6b25c811c054c20aea1e22b72fc8fd1c93932fab8e576324');
  assert.equal(plan.cards.length, 20);
  assert.equal(new Set(plan.cards.map(c => c.id)).size, 20);
  const old = new Set(pool.cards.slice(0, 213).map(c => c.id));
  const colors = {};
  for (const c of plan.cards) {
    assert(!old.has(c.id), c.id);
    assert(!['CQ11', 'TK008'].includes(c.id));
    colors[c.color] = (colors[c.color] || 0) + 1;
    assert.equal(hash(read(c.sourceImage)), c.sourceImageSha256, c.id);
    assert.equal(specs[c.id].sourceVerification.imageSha256, c.sourceImageSha256, c.id);
    assert.equal(c.imageReviewedHere, true, c.id);
    assert.equal(c.selectionEnabled, false, c.id);
    assert.equal(c.published, false, c.id);
    assert.equal(c.accepted, false, c.id);
  }
  assert.deepEqual(colors, { 红: 5, 黑: 5, 蓝: 5, 灰: 5 });
  const pilots = plan.cards.filter(c => c.runtime === 'nativePilotPassedPendingBatchIntegration');
  assert.equal(pilots.length, plan.scope.nativePilots);
  assert.deepEqual(pool.cards.slice(213).map(c => c.id).sort(), pilots.map(c => c.id).sort());
  assert(pool.cards.slice(213).every(c => c.supported === false));
  for (const key of ['selectionEnabled', 'published', 'accepted', 'fullyIntegratedNewCards']) assert.equal(plan.scope[key], 0);
  assert.equal(plan.candidateComparison[0].nativeCards, 20);
  assert.equal(plan.candidateComparison[0].wasmCards, 20);
  assert.equal(plan.candidateComparison[1].wasmCards, 0);
  assert.equal(hash(read(plan.ruleEvidence.chaos.pdf)), plan.ruleEvidence.chaos.sha256);
});

test('fourth original corrections preserve conditional public, conditional secret plans and exact threshold', () => {
  assert.deepEqual(specs.WM060.fields.printedKeywords, ['暴露条件：公开']);
  assert.match(specs.WM060.fields.printedRuleTextLines[0], /暴露〔日形〕公开/);
  for (const id of ['WM045', 'WM046', 'WM047']) {
    assert.deepEqual(specs[id].fields.printedKeywords, ['隐秘条件：秘密计划', '可推进']);
    assert.match(specs[id].fields.printedRuleTextLines[0], /隐秘〔月形〕秘密计划/);
  }
  assert.match(specs.WM046.fields.printedRuleTextLines[1], /达成推进2时/);
  assert(!specs.WM046.fields.printedRuleTextLines.join('').includes('达成推进3'));
  assert.equal(plan.sourceCorrections.length, 4);
  const corrected = new Set(plan.sourceCorrections.map(c => c.id));
  const untouched = Object.fromEntries(Object.entries(specs).filter(([id]) => !corrected.has(id)));
  assert.equal(digest(untouched), plan.baseSourceRecordsExceptFourCorrectionsSha256);
});
