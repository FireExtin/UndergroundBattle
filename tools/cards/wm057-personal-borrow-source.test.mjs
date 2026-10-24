import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { wm057BorrowSlice as intake, currentImport20 } from './import20-spec-history.mjs';
const read=n=>readFileSync(new URL('../../'+n,import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool=JSON.parse(read('rust-game/data/cards.json')),specs=JSON.parse(read('docs/factions/card-specifications.json'));
test('broker ruling changes no269 card records or public213 and retains the full primary document identity',()=>{
  assert.equal(intake.priorCatalogCount,269);assert.equal(pool.cards.length,271);assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,269)))),intake.priorCardsCanonicalSha256);assert.equal(sha(read('docs/factions/card-specifications.json')),intake.priorSpecificationsSha256);assert.equal(pool.sourceSha256,intake.priorSpecificationsSha256);assert.deepEqual(intake.newCards,[]);assert.deepEqual(intake.sourceCorrections,[]);assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);
});
test('original broker printed source and image are unchanged; personal ruling is separate from printed text',()=>{
  const d=pool.cards.find(c=>c.id==='WM057'),s=specs.cards.WM057;
  assert.equal(d.cost,2);assert.equal(d.color,'黑');assert.equal(d.text,s.fields.printedRuleTextLines.join('\n'));assert.equal(d.source.imageSha256,s.sourceVerification.imageSha256);assert.equal(sha(read(s.sourceVerification.imagePath)),d.source.imageSha256);assert.equal(sha(read('web/public/cards/WM057.jpg')),d.source.imageSha256);
  assert.equal(intake.ruling.userReply,'OK');assert.equal(intake.ruling.userReplyMessage,'Sentinel_0e77b75db9dc81918586912ac13d8f34');assert(intake.ruling.scope.includes('actual enemy activator personal secrecy'));
});
test('single109 and17 author WIP do not assert independent or public acceptance',()=>{
  assert.equal(intake.engineVersion,'rust-v0.2.109-wm057-personal-borrow');assert.equal(intake.cardPoolVersion,'limited-v2.100-wm057-personal-borrow');assert.equal(pool.engineVersion,currentImport20.engineVersion);assert.equal(pool.cardPoolVersion,currentImport20.cardPoolVersion);assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));assert.equal(intake.priorReference20AuthorNativeWipComplete,16);assert.equal(intake.potentialReference20AuthorNativeWipComplete,17);assert.equal(intake.current50ReviewedNativeClosed,38);assert.equal(intake.independentReview,'notRun');assert.equal(intake.WASM,'notBuiltOrRun');assert.equal(intake.Worker,'notBuiltOrRun');assert.equal(intake.published,false);
});
test('XQ19 remains unregistered and this ruling carries no cost or broad copying permission',()=>{
  assert(!pool.cards.some(c=>c.id==='XQ19'));assert.equal(intake.references.SuezCodeOrArtCopied,false);assert(intake.ruling.scope.includes('No team-wide authority'));assert(intake.ruling.scope.includes('XQ19 cost inference'));
});
