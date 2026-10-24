import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { cultistSlice as intake, currentImport20, beforeCultistSpecifications, priorSpecifications } from './import20-spec-history.mjs';
const read = n => readFileSync(new URL('../../'+n,import.meta.url));
const sha = b => createHash('sha256').update(b).digest('hex');
const canonical = x => Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const specs = JSON.parse(read('docs/factions/card-specifications.json'));

test('XG13 appends one original card; all260 previous records and public213 remain identical',()=>{
  const digest = 'b59caeaf73d14629cb7107d8669e57a26bb5825d121fb60e2153f9ded2a864d5';
  assert.equal(intake.priorCatalogCount,260); assert.equal(intake.priorCardsCanonicalSha256,digest);
  assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,260)))),digest);
  assert.equal(pool.cards.length,271); assert.equal(new Set(pool.cards.map(c=>c.id)).size,271);
  assert.deepEqual(pool.cards.slice(260,261).map(c=>c.id),['XG13']);
  assert.deepEqual(pool.cards.slice(261,263).map(c=>c.id), ['WM064','CQ24']);assert.deepEqual(pool.cards.slice(263,265).map(c=>c.id), ['BQ092','BQ093']);assert.deepEqual(pool.cards.slice(265,267).map(c=>c.id),['CQ18','CQ19']);assert.deepEqual(pool.cards.slice(267,269).map(c=>c.id),['XG20','BQ057']);
  assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);
  assert.equal(pool.cards.find(c=>c.id==='XG13').supported,false);
  assert(!pool.cards.some(c=>c.id==='XQ19'));
});
test('exact two XG13 faction-symbol corrections preserve earlier whole source identities',()=>{
  const old = '快速行动 牺牲另一个邪教徒：本回合中你正面打出的下一张灾难牌或具有血脉图标的牌，其所需费用减少②。';
  const corrected = old.replace('灾难牌','鸣钟教派牌');
  assert.deepEqual(intake.sourceCorrections,[
    {id:'XG13',path:['fields','printedRuleTextLines',1],old,new:corrected},
    {id:'XG13',path:['abilities',1,'printedText'],old,new:corrected},
  ]);
  assert.equal(sha(JSON.stringify(beforeCultistSpecifications(specs),null,2)+'\n'),'1f40ee44bf6cc18bd70e8befe4a3a99c91eea1bd6c461da2d36d45f2d488de87');
  assert.equal(sha(JSON.stringify(priorSpecifications(specs),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');
  assert.equal(pool.sourceSha256,sha(read('docs/factions/card-specifications.json')));
});
test('XG13 text, unique title, loyalty and original licensed pixels agree',()=>{
  const d = pool.cards.find(c=>c.id==='XG13'), s = specs.cards.XG13.fields;
  assert.equal(d.name,'玛丽·林德博格'); assert.equal(d.subtitle,'狂信者'); assert.equal(d.cost,2);
  assert.equal(d.color,'红'); assert.equal(d.society,'鸣钟教派'); assert.equal(d.magic,'鲜血');
  assert.deepEqual(d.loyalty,['红色','红色']); assert.deepEqual(d.subtypes,['人类','邪教徒']);
  assert.equal(d.unique,true); assert.equal(d.defense,1);
  assert.deepEqual(d.icons,{permanent:s.permanentIcons,temporary:s.temporaryIcons});
  assert.equal(d.text,s.printedRuleTextLines.join('\n')); assert.equal(d.text,specs.cards.XG13.abilities.map(a=>a.printedText).join('\n'));
  const digest = '3a7d2bf5d20abd2a5da6e5c8d9f777e72668b14aab7a87ccc507ceef9ecf803a';
  assert.equal(sha(read(d.source.image)),digest); assert.equal(sha(read('web/public/cards/XG13.jpg')),digest);
});
test('current native identity native WIP retains unresolved cards and authorization boundaries',()=>{
  assert.equal(pool.engineVersion,currentImport20.engineVersion); assert.equal(pool.cardPoolVersion,currentImport20.cardPoolVersion);
  assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert.equal(intake.priorReference20AuthorNativeWipComplete,7); assert.equal(intake.potentialReference20AuthorNativeWipComplete,8);
  assert.equal(intake.current50ReviewedNativeClosed,38); assert.equal(intake.published,false);
  assert.match(intake.deferred.XQ19,/not a zero/); assert.match(intake.deferred.WM057,/pending user ruling/);
  assert.equal(intake.independentReview,'notRun'); assert.equal(intake.WASM,'notBuiltOrRun'); assert.equal(intake.Worker,'notBuiltOrRun');
  assert.equal(intake.references.SuezCodeOrArtCopied,false);
  const question = JSON.parse(read('docs/factions/WM057_ENEMY_PERMISSION_QUESTION_2026-10-24.json'));
  assert.equal(question.name,'信息掮客'); assert.match(question.readyQuestion,/甲、乙都可以/);
  assert.equal(question.sourceReferences[2].viewedPhysicalPages.length,6);
  for(const [path,digest] of Object.entries(question.sourceSha256)) assert.equal(sha(read(path)),digest);
});
