import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { currentImport20 } from './import20-spec-history.mjs';
const read = n => readFileSync(new URL('../../' + n, import.meta.url));
const hash = b => createHash('sha256').update(b).digest('hex');
const canonical = x => Array.isArray(x) ? x.map(canonical) : x && typeof x === 'object'
  ? Object.fromEntries(Object.keys(x).sort().map(k => [k, canonical(x[k])])) : x;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const specs = JSON.parse(read('docs/factions/card-specifications.json')).cards;
const plan = JSON.parse(read('docs/factions/batch20-fourth-2026-10-22.json'));

test('WM037 and complete red Society dependency retain their exact pinned originals', () => {
  for (const [id, expected] of [
    ['WM037', 'e5af8b1e0846a91fa44550cd76407cf6579c07d5acc5d24a1bbb2410a8bebf1b'],
    ['MSJC13', '4bbdcc0a008cfc5984f1b812ad0ba3b107535cf1e2ff20e5bb765899edfb91fb'],
  ]) {
    assert.equal(specs[id].fields.colorKey, '红');
    assert.equal(specs[id].sourceVerification.imageSha256, expected);
    assert.equal(hash(read(specs[id].sourceVerification.imagePath)), expected);
    assert.equal(hash(read(`web/public/cards/${id}.jpg`)), expected);
    assert.deepEqual(specs[id].openQuestionIds, []);
  }
  const card = pool.cards.find(c => c.id === 'WM037');
  assert.equal(card.text, specs.WM037.fields.printedRuleTextLines.join('\n'));
  assert.equal(card.cost, 2); assert.deepEqual(card.loyalty, ['红色']); assert.equal(card.magic, '心灵');
  assert.equal(specs.MSJC13.fields.printedCost, null); assert.equal(specs.MSJC13.fields.startingHand, 6);
  assert.deepEqual(specs.MSJC13.abilities[0].restriction, {
    uncappedColors: ['红', '中立'], maximumOtherColorCardsCombined: 12,
  });
});

test('prior249 regular definitions remain byte-semantically unchanged and public213 stay fixed', () => {
  assert.equal(hash(JSON.stringify(canonical(pool.cards.slice(0, 249)))), 'eb240c02012cff99fb2268d11b929a4dfcca029106c4a9fc8326516198e77b50');
  assert.equal(pool.cards.length, 258);
  assert.equal(pool.cards[249].id, 'WM037');
  assert.deepEqual(pool.cards.slice(254,256).map(c => c.id), ['BQ039', 'WM028']);
  assert.deepEqual(pool.cards.slice(256).map(c=>c.id), ['WM025','WM024']);
  assert.equal(pool.cards.filter(c => c.supported !== false).length, 213);
  assert.equal(pool.cards[249].supported, false);
  assert.deepEqual(pool.cards.slice(250, 252).map(c => c.id), ['WM044', 'WM043']);
  assert.deepEqual(pool.cards.slice(252,254).map(c => c.id), ['BQ090', 'BQ064']);
  assert(pool.cards.slice(249).every(c => c.supported === false));
  assert(!pool.cards.some(c => ['MSJC14', 'MSJC13'].includes(c.id)));
  assert.equal(pool.engineVersion, currentImport20.engineVersion);
  assert.equal(pool.cardPoolVersion, currentImport20.cardPoolVersion);
});

test('WM037 candidate and Society dependency do not inflate independent or public completion', () => {
  const wm = plan.cards.find(c => c.id === 'WM037');
  assert.equal(wm.runtime, 'nativePilotPassedPendingBatchIntegration');
  assert.equal(wm.fullNativeCardComplete, true);
  assert.equal(wm.dependencySociety, 'MSJC13');
  assert.equal(wm.selectionEnabled, false); assert.equal(wm.published, false); assert.equal(wm.accepted, false);
  assert(!plan.cards.some(c => c.id === 'MSJC13'));
  assert.equal(plan.scope.nativePilots, 37);
  assert.equal(plan.projectProgressIncludingExternalReviewedBranches.nativeClosed, 37);
  assert.equal(plan.externalNativeClosure.WM044.integratedInThisClone, true);
  assert.equal(plan.scope.accepted, 0);
});
