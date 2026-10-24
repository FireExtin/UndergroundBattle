import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { sentinelSlice as intake, priorSpecifications } from './import20-spec-history.mjs';
const read=n=>readFileSync(new URL('../../'+n,import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool=JSON.parse(read('rust-game/data/cards.json')),specs=JSON.parse(read('docs/factions/card-specifications.json'));
test('Sentinel pair preserves all prior265, public213 and exact new gray unique records',()=>{
  assert.equal(intake.priorCatalogCount,265);assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,265)))),intake.priorCardsCanonicalSha256);
  assert.equal(pool.cards.length,269);assert.equal(new Set(pool.cards.map(c=>c.id)).size,269);assert.deepEqual(pool.cards.slice(265,267).map(c=>c.id),['CQ18','CQ19']);
  assert(pool.cards.slice(265,267).every(c=>c.supported===false&&c.color==='灰'&&c.unique));assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);
});
test('no primary rules correction; historical entire-document source identity remains',()=>{
  assert.deepEqual(intake.sourceCorrections,[]);assert.equal(sha(read('docs/factions/card-specifications.json')),intake.priorSpecificationsSha256);assert.equal(pool.sourceSha256,intake.priorSpecificationsSha256);
  assert.equal(sha(JSON.stringify(priorSpecifications(specs),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');
});
test('both original images and complete printed metadata are preserved',()=>{
  for(const id of ['CQ18','CQ19']){const d=pool.cards.find(c=>c.id===id),s=specs.cards[id],f=s.fields;
    assert.equal(d.cost,4);assert.equal(d.subtitle,f.subtitle);assert.equal(d.text,f.printedRuleTextLines.join('\n'));assert.deepEqual(d.subtypes,f.subtypes);assert.equal(d.defense,f.defense);assert.deepEqual(d.icons,{permanent:f.permanentIcons,temporary:f.temporaryIcons});
    assert.equal(d.source.imageSha256,s.sourceVerification.imageSha256);assert.equal(sha(read(s.sourceVerification.imagePath)),d.source.imageSha256);assert.equal(sha(read(`web/public/cards/${id}.jpg`)),d.source.imageSha256);
  }assert.equal(pool.cards[265].loyalty.length,3);assert.equal(pool.cards[266].loyalty.length,1);
});
test('single current107 remains author WIP, with no new public or independent acceptance',()=>{
  assert.equal(pool.engineVersion,'rust-v0.2.108-import20-tax-ban');assert.equal(pool.cardPoolVersion,'limited-v2.99-import20-tax-ban');assert.equal(intake.engineVersion,'rust-v0.2.107-import20-sentinel');assert.equal(intake.cardPoolVersion,'limited-v2.98-import20-sentinel');
  assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));assert.equal(intake.priorReference20AuthorNativeWipComplete,12);assert.equal(intake.potentialReference20AuthorNativeWipComplete,14);assert.equal(intake.current50ReviewedNativeClosed,38);
  assert.equal(intake.independentReview,'notRun');assert.equal(intake.WASM,'notBuiltOrRun');assert.equal(intake.Worker,'notBuiltOrRun');assert.equal(intake.published,false);assert.equal(intake.references.SuezCodeOrArtCopied,false);
});
