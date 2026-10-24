import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { currentImport20 as intake, priorSpecifications } from './import20-spec-history.mjs';
const read=n=>readFileSync(new URL('../../'+n,import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool=JSON.parse(read('rust-game/data/cards.json')),specs=JSON.parse(read('docs/factions/card-specifications.json'));
test('tax/ban slice preserves all267 and public213, with exact two gray candidate records',()=>{
  assert.equal(intake.priorCatalogCount,267);assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,267)))),intake.priorCardsCanonicalSha256);
  assert.equal(pool.cards.length,269);assert.equal(new Set(pool.cards.map(c=>c.id)).size,269);assert.deepEqual(pool.cards.slice(267).map(c=>c.id),['XG20','BQ057']);
  assert(pool.cards.slice(267).every(c=>c.supported===false&&c.color==='灰'));assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);
  assert.equal(pool.cards[267].unique,false);assert.equal(pool.cards[268].unique,true);
});
test('primary rules and retained historical document digest do not change',()=>{
  assert.deepEqual(intake.sourceCorrections,[]);assert.equal(sha(read('docs/factions/card-specifications.json')),intake.priorSpecificationsSha256);assert.equal(pool.sourceSha256,intake.priorSpecificationsSha256);
  assert.equal(sha(JSON.stringify(priorSpecifications(specs),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');
});
test('original images, loyalty, unique subtitle and printed cost/metadata match the pinned pixels',()=>{
  for(const id of intake.newCards){const d=pool.cards.find(c=>c.id===id),s=specs.cards[id],f=s.fields;
    assert.equal(d.cost,3);assert.equal(d.subtitle,f.subtitle||'');assert.equal(d.text,f.printedRuleTextLines.join('\n'));assert.deepEqual(d.subtypes,f.subtypes);assert.equal(d.defense,f.defense);assert.deepEqual(d.icons,{permanent:f.permanentIcons,temporary:f.temporaryIcons});
    assert.equal(d.source.imageSha256,s.sourceVerification.imageSha256);assert.equal(sha(read(s.sourceVerification.imagePath)),d.source.imageSha256);assert.equal(sha(read(`web/public/cards/${id}.jpg`)),d.source.imageSha256);
  }assert.equal(pool.cards[267].loyalty.length,3);assert.equal(pool.cards[268].loyalty.length,2);assert.deepEqual(pool.cards[267].keywords,['非资产']);assert.deepEqual(pool.cards[268].keywords,['公开','撤回']);
});
test('only latest108 author WIP; honor is a separate printed marker, with no new public acceptance',()=>{
  assert.equal(pool.engineVersion,'rust-v0.2.108-import20-tax-ban');assert.equal(pool.cardPoolVersion,'limited-v2.99-import20-tax-ban');assert.equal(pool.engineVersion,intake.engineVersion);assert.equal(pool.cardPoolVersion,intake.cardPoolVersion);
  assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));assert.equal(intake.priorReference20AuthorNativeWipComplete,14);assert.equal(intake.potentialReference20AuthorNativeWipComplete,16);assert.equal(intake.current50ReviewedNativeClosed,38);
  assert.equal(intake.independentReview,'notRun');assert.equal(intake.WASM,'notBuiltOrRun');assert.equal(intake.Worker,'notBuiltOrRun');assert.equal(intake.published,false);assert.equal(intake.references.SuezCodeOrArtCopied,false);
});
