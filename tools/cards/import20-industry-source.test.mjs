import { beforeBatch20Admission } from './import20-spec-history.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { industrySlice as intake, currentImport20, beforeIndustrySpecifications, priorSpecifications } from './import20-spec-history.mjs';
const read=n=>readFileSync(new URL('../../'+n,import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool=JSON.parse(read('rust-game/data/cards.json')),specs=JSON.parse(read('docs/factions/card-specifications.json'));
const priorCards=beforeBatch20Admission(pool.cards);

test('industry appends exactly2 and preserves all earlier263 and public213',()=>{
  const digest='7f9ec9a7c352d0ce5114a15eae9d0cff6056a0d74a00e1c404d48714944f25a6';
  assert.equal(intake.priorCatalogCount,263);assert.equal(intake.priorCardsCanonicalSha256,digest);assert.equal(sha(JSON.stringify(canonical(priorCards.slice(0,263)))),digest);
  assert.equal(priorCards.length,272);assert.equal(new Set(priorCards.map(c=>c.id)).size,272);assert.deepEqual(priorCards.slice(263,265).map(c=>c.id),['BQ092','BQ093']);assert.deepEqual(priorCards.slice(265,267).map(c=>c.id),['CQ18','CQ19']);assert.deepEqual(priorCards.slice(267,269).map(c=>c.id),['XG20','BQ057']);assert(priorCards.slice(263,265).every(c=>c.supported===false));assert.equal(priorCards.filter(c=>c.supported!==false).length,213);
});
test('exact BQ092 self-exhaust correction reconstructs the entire previous source and all older identities',()=>{
  const old='快速行动，横置一个副名为圣甲虫产业的本方附属：你获得①直到本步骤结束。';
  const corrected='快速行动，横置本牌，横置一个副名为圣甲虫产业的本方附属：你获得①直到本步骤结束。';
  assert.deepEqual(intake.sourceCorrections,[{id:'BQ092',path:['fields','printedRuleTextLines',1],old,new:corrected},{id:'BQ092',path:['abilities',1,'printedText'],old,new:corrected}]);
  assert.equal(sha(JSON.stringify(beforeIndustrySpecifications(specs),null,2)+'\n'),'a3d393a179ac17e46e8d213279bea1a3f92f0a2adc2d79c3f73973e93c2e13e9');assert.equal(sha(JSON.stringify(priorSpecifications(specs),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');assert.equal(pool.sourceSha256,sha(read('docs/factions/card-specifications.json')));
});
test('both black facilities retain original pixels costs subtitle and nonunique status',()=>{
  for(const[id,cost,hash]of[['BQ092',1,'5b69295d844278f651226f007b2669b7ec2a8cc56e8bc89ab4bfa6e3714036fe'],['BQ093',2,'d182614e9928ab4cd2f5159b107fb93d2e487762bf202ade0215c06437916f0c']]){
    const d=priorCards.find(c=>c.id===id),s=specs.cards[id];assert.equal(d.color,'黑');assert.equal(d.cost,cost);assert.equal(d.kind,'attachment');assert.equal(d.unique,false);assert.equal(d.subtitle,'圣甲虫产业');assert.deepEqual(d.loyalty,['黑色']);assert.equal(d.defense,null);assert.equal(d.source.imageSha256,hash);assert.equal(d.text,s.fields.printedRuleTextLines.join('\n'));assert.equal(sha(read(s.sourceVerification.imagePath)),hash);assert.equal(sha(read(`web/public/cards/${id}.jpg`)),hash);
  }
});
test('single current kernel remains WIP with no new public or independent completion claim',()=>{
  assert.equal(intake.engineVersion,'rust-v0.2.106-import20-industry');assert.equal(intake.cardPoolVersion,'limited-v2.97-import20-industry');assert.equal(currentImport20.engineVersion,pool.engineVersion);assert.equal(currentImport20.cardPoolVersion,pool.cardPoolVersion);assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert.equal(intake.priorReference20AuthorNativeWipComplete,10);assert.equal(intake.potentialReference20AuthorNativeWipComplete,12);assert.equal(intake.current50ReviewedNativeClosed,38);assert.equal(intake.independentReview,'notRun');assert.equal(intake.WASM,'notBuiltOrRun');assert.equal(intake.Worker,'notBuiltOrRun');assert.equal(intake.published,false);assert.equal(intake.references.SuezCodeOrArtCopied,false);assert(!priorCards.slice(0,271).some(c=>c.id==='XQ19'));
});
