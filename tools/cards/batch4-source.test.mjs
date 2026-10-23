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
const packageB = JSON.parse(read('docs/factions/batch4-package-b-20261022.json'));
const partials = new Set(packageB.nativePartialCards);

test('fourth roster preserves frozen213 and keeps native pilots separate from public admission', () => {
  assert.equal(plan.baseCommit, 'b80f12202799a632ee9d75f40da8352f2ca274f1');
  assert.equal(digest(pool.cards.slice(0, 213)), '5ddc50185c92d6ba6b25c811c054c20aea1e22b72fc8fd1c93932fab8e576324');
  assert.equal(plan.scope.target, 50);
  assert.equal(plan.cards.length, plan.scope.selected);
  assert(plan.scope.selected <= plan.scope.target);
  assert.equal(plan.scope.unassignedSlots, plan.scope.target - plan.cards.length);
  assert.equal(new Set(plan.cards.map(c => c.id)).size, plan.cards.length);
  const old = new Set(pool.cards.slice(0, 213).map(c => c.id));
  const colors = {};
  for (const c of plan.cards) {
    assert(!old.has(c.id), c.id);
    assert(['红', '黑', '蓝', '灰'].includes(c.color), c.id);
    assert(!['CQ11', 'TK008'].includes(c.id));
    colors[c.color] = (colors[c.color] || 0) + 1;
    assert.equal(hash(read(c.sourceImage)), c.sourceImageSha256, c.id);
    assert.equal(specs[c.id].sourceVerification.imageSha256, c.sourceImageSha256, c.id);
    assert.equal(c.imageReviewedHere, true, c.id);
    assert.equal(c.selectionEnabled, false, c.id);
    assert.equal(c.published, false, c.id);
    assert.equal(c.accepted, false, c.id);
  }
  assert.deepEqual(colors, plan.colorCounts);
  assert.equal(Object.values(colors).reduce((a,b) => a+b, 0), plan.scope.selected);
  assert.equal(plan.mechanismClusters.flatMap(c => c.cards).length, plan.scope.selected);
  assert.equal(new Set(plan.mechanismClusters.flatMap(c => c.cards)).size, plan.scope.selected);
  assert.equal(plan.expansionSlotPlan.reduce((n,c) => n+c.unassignedSlots, 0), plan.scope.unassignedSlots);
  const pilots = plan.cards.filter(c => c.runtime === 'nativePilotPassedPendingBatchIntegration');
  assert.equal(pilots.length, plan.scope.nativePilots);
  const drafts = plan.cards.filter(c => c.runtime === 'nativeWipDirectedPassedPendingClosure');
  assert.equal(drafts.length, plan.scope.nativeWipCards ?? 0);
  const reviewed = ['WM021', 'WM022', 'WM032', 'WM061', 'BQ082', 'BQ027', 'BQ031', 'BQ032', 'BQ038'];
  assert.equal(plan.packAValidation.completionCountAdded, 5);
  assert.equal(plan.scope.nativePilots, 35);
  assert.equal(plan.lc16ClosureReview.status, 'closed');
  assert.equal(hash(read(plan.lc16ClosureReview.report)), plan.lc16ClosureReview.reportSha256);
  assert(pilots.some(c => c.id === 'LC16' && c.fullNativeCardComplete));
  assert.equal(plan.xq20ClosureReview.status, 'closed');
  assert.equal(hash(read(plan.xq20ClosureReview.report)), plan.xq20ClosureReview.reportSha256);
  assert(pilots.some(c => c.id === 'XQ20' && c.fullNativeCardComplete));
  assert.equal(plan.jz56ClosureReview.productSource, '292d3f75aaddae742ad04ca0e1833b68944c6c2d');
  assert.equal(plan.jz56ClosureReview.status, 'closed');
  assert.equal(hash(read(plan.jz56ClosureReview.report)), plan.jz56ClosureReview.reportSha256);
  assert(pilots.some(c => c.id === 'JZ56' && c.fullNativeCardComplete));
  assert.equal(plan.scope.remainingNativePrograms, plan.scope.target - pilots.length);
  assert.equal(plan.scope.nativePartialCards, partials.size);
  assert.deepEqual(plan.nativeClosureReview.closedCards, reviewed);
  assert.deepEqual(plan.nativeClosureReview.notClosedCards, ['BQ036']);
  assert.equal(plan.nativeClosureReview.candidate, 'e5ccb050f91222838b79d33001f00ae97c1fb69e');
  assert.equal(hash(read(plan.nativeClosureReview.report)), plan.nativeClosureReview.reportSha256);
  for (const id of reviewed) assert(pilots.some(c => c.id === id), id);
  assert(pilots.some(c => c.id === 'BQ036'));
  assert(!pilots.some(c => c.id === 'XG21'));
  assert.equal(plan.bq036ClosureReview.candidate, '8a5cf1f760c143b14e7e3d4cc78cd55bcaa2f022');
  assert.equal(plan.bq036ClosureReview.status, 'closed');
  assert.equal(plan.bq036ClosureReview.nativeWholeCardComplete, true);
  assert.equal(hash(read(plan.bq036ClosureReview.report)), plan.bq036ClosureReview.reportSha256);
  assert.deepEqual(pool.cards.slice(213).map(c => c.id).sort(), [...pilots.map(c => c.id), ...drafts.map(c => c.id), ...partials].sort());
  assert(pool.cards.slice(213).every(c => c.supported === false));
  for (const c of pool.cards.slice(213)) {
    const spec = specs[c.id];
    assert.equal(c.cost, spec.fields.printedCost, c.id);
    assert.equal(c.defense, spec.fields.defense, c.id);
    assert.deepEqual(c.icons.permanent, spec.fields.permanentIcons, c.id);
    assert.deepEqual(c.icons.temporary, spec.fields.temporaryIcons, c.id);
    assert.equal(c.source.imageSha256, spec.sourceVerification.imageSha256, c.id);
    assert.equal(hash(read(`web/public/cards/${c.id}.jpg`)), spec.sourceVerification.imageSha256, c.id);
  }
  for (const key of ['selectionEnabled', 'published', 'accepted', 'fullyIntegratedNewCards']) assert.equal(plan.scope[key], 0);
  assert.equal(plan.candidateComparison[0].nativeCards, 20);
  assert.equal(plan.candidateComparison[0].wasmCards, 20);
  assert.equal(plan.candidateComparison[1].wasmCards, 1);
  assert.deepEqual(plan.platformValidation.directedCards, ['JZ56']);
  assert.equal(plan.platformValidation.nativeWasmCommands, 200);
  assert.equal(plan.platformValidation.workerCommands, 200);
  assert.equal(plan.platformValidation.batch50Complete, false);
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
  const corrected = new Set([...plan.sourceCorrections.map(c => c.id), ...packageB.sourceCorrections]);
  const untouched = Object.fromEntries(Object.entries(specs).filter(([id]) => !corrected.has(id)));
  assert.equal(digest(untouched), packageB.baseSourceRecordsExceptPriorFourAndPackageBFiveSha256);
});


test('package B independently closed native cards preserve prior catalogue and public gate', () => {
  assert.equal(packageB.baseCommit, 'adf0a1c080e21b75e971bd650a61f22e47f42e91');
  assert.equal(packageB.baseCatalogCount, 235);
  assert.equal(digest(pool.cards.slice(0, packageB.baseCatalogCount)), packageB.baseCatalogSha256);
  const packageCards = ['BQ027', 'BQ031', 'BQ032', 'BQ036', 'BQ038'];
  assert.deepEqual([...partials], []);
  assert.deepEqual(packageB.historicalNativePartialCards, packageCards);
  assert.deepEqual(packageB.nativeCompleteCards, packageCards);
  assert.deepEqual(packageB.sourceCorrections, packageCards);
  assert.equal(packageB.completeCards, packageB.nativeCompleteCards.length);
  assert.equal(packageB.completeCards, 5);
  assert.equal(packageB.supportedCards, 0);
  assert.equal(packageB.wasmCards, 0);
  for (const id of packageCards) {
    const c = pool.cards.find(c => c.id === id);
    assert.equal(c.supported, false, id);
    assert.equal(plan.cards.find(c => c.id === id).runtime === 'nativePilotPassedPendingBatchIntegration',
      packageB.nativeCompleteCards.includes(id), id);
    assert.equal(hash(read(c.source.image)), packageB.originalCardImageSha256[id], id);
  }
  for (const id of ['BQ031', 'BQ032']) {
    assert.match(specs[id].fields.printedRuleTextLines[0], /〔日形〕非资产/);
    assert(!specs[id].fields.printedRuleTextLines[0].includes('公开'), id);
    const c = pool.cards.find(c => c.id === id);
    assert(c.keywords.includes('暴露条件：非资产'), id);
    assert(!c.text.includes('公开'), id);
  }
  for (const id of ['BQ027', 'BQ036', 'BQ038']) {
    assert(!specs[id].fields.printedRuleTextLines.join('').includes('地区势力'));
  }
  for (const source of packageB.rulesRead) assert.equal(hash(read(source.path)), source.sha256);
});
