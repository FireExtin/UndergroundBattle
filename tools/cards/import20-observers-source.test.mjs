import { beforeBatch20Admission } from './import20-spec-history.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { observerSlice as intake, currentImport20 } from './import20-spec-history.mjs';
const read = n => readFileSync(new URL('../../'+n, import.meta.url));
const sha = b => createHash('sha256').update(b).digest('hex');
const canonical = x => Array.isArray(x) ? x.map(canonical) : x && typeof x==='object'
  ? Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])) : x;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const priorCards=beforeBatch20Admission(pool.cards);

const specs = JSON.parse(read('docs/factions/card-specifications.json')).cards;

test('second serial pair preserves exact254 prior records and public213', () => {
  assert.equal(intake.priorCatalogCount,254);
  assert.equal(sha(JSON.stringify(canonical(priorCards.slice(0,254)))),'6d9b57bc51e4fe36012d9455e5c153d8891184662ea082d85e08d30bece4c8b9');
  assert.equal(intake.priorCardsCanonicalSha256,'6d9b57bc51e4fe36012d9455e5c153d8891184662ea082d85e08d30bece4c8b9');
  assert.deepEqual(priorCards.slice(254,256).map(c=>c.id),['BQ039','WM028']);
  assert.equal(new Set(priorCards.map(c=>c.id)).size,272);
  assert.deepEqual(priorCards.slice(256,258).map(c=>c.id), ['WM025','WM024']);
  assert.deepEqual(priorCards.slice(258,260).map(c=>c.id), ['BQ081','WM057']);
  assert.deepEqual(priorCards.slice(260,261).map(c=>c.id), ['XG13']);
  assert.deepEqual(priorCards.slice(261,263).map(c=>c.id), ['WM064','CQ24']);assert.deepEqual(priorCards.slice(263,265).map(c=>c.id), ['BQ092','BQ093']);assert.deepEqual(priorCards.slice(265,267).map(c=>c.id),['CQ18','CQ19']);assert.deepEqual(priorCards.slice(267,269).map(c=>c.id),['XG20','BQ057']);
  assert.equal(priorCards.filter(c=>c.supported!==false).length,213);
  for (const id of intake.allFourUnadmitted) assert.equal(priorCards.find(c=>c.id===id).supported,false);
  assert.equal(intake.current50ReviewedNativeClosed,38); assert.equal(intake.published,false);
});
test('blue attachments retain original pixels, printed costs, loyalty, unique and full text', () => {
  for (const [id,hash,cost,loyalty,unique] of [
    ['BQ039','db30bdf64444534316d80a9d7665f9ae3df42a9ee886b38b7aed6f1cd25fe2de',1,['蓝色','蓝色'],false],
    ['WM028','f0d66f992939844fcdcc1844de1855e3def4a73c0e8af578fbcb595ce98a7aee',2,['蓝色'],true],
  ]) {
    const c=priorCards.find(c=>c.id===id),s=specs[id];
    assert.equal(sha(read(s.sourceVerification.imagePath)),hash); assert.equal(sha(read(`web/public/cards/${id}.jpg`)),hash);
    assert.equal(c.cost,cost);assert.deepEqual(c.loyalty,loyalty);assert.equal(c.unique,unique);
    assert.equal(c.color,'蓝');assert.equal(c.kind,'attachment');assert.equal(c.text,s.fields.printedRuleTextLines.join('\n'));
    assert.deepEqual(c.subtypes,s.fields.subtypes);assert.deepEqual(s.openQuestionIds,[]);
  }
  assert.match(specs.BQ039.fields.printedRuleTextLines[1],/战斗对抗/);
  assert.match(specs.WM028.fields.printedRuleTextLines[1],/准备步骤强制触发/);
});
test('current development kernel is a single unqualified development kernel, not a public release', () => {
  assert.equal(pool.engineVersion,currentImport20.engineVersion);
  assert.equal(pool.cardPoolVersion,currentImport20.cardPoolVersion);
  const rust=read('rust-game/src/catalog.rs').toString();
  assert(rust.includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert(!rust.includes('rust-v0.2.100-import20-first-slice'));
  assert.equal(intake.independentReview,'notRun');
  for (const k of ['WASM','Worker']) assert.equal(intake[k],'notBuiltOrRun');
  assert.equal(intake.browser,'notRun');
});
