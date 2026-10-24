import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { thiefTattooSlice as intake, currentImport20, beforeThiefTattooSpecifications, priorSpecifications } from './import20-spec-history.mjs';
const read=n=>readFileSync(new URL('../../'+n,import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool=JSON.parse(read('rust-game/data/cards.json')), specs=JSON.parse(read('docs/factions/card-specifications.json'));
test('thief and tattoo append exactly2 while preserving every prior261 and public213',()=>{
  const digest='a5c28650d26f011ac54666675cac4ff7a0d0818fa30fefc787f0729c89dd53a3';
  assert.equal(intake.priorCatalogCount,261);assert.equal(intake.priorCardsCanonicalSha256,digest);
  assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,261)))),digest);
  assert.equal(pool.cards.length,271);assert.equal(new Set(pool.cards.map(c=>c.id)).size,271);
  assert.deepEqual(pool.cards.slice(261,263).map(c=>c.id),['WM064','CQ24']);assert.deepEqual(pool.cards.slice(263,265).map(c=>c.id), ['BQ092','BQ093']);assert.deepEqual(pool.cards.slice(265,267).map(c=>c.id),['CQ18','CQ19']);assert.deepEqual(pool.cards.slice(267,269).map(c=>c.id),['XG20','BQ057']);assert(pool.cards.slice(261).every(c=>c.supported===false));
  assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);assert(!pool.cards.some(c=>c.id==='XQ19'));
});
test('exact CQ24 cost3 correction reconstructs the entire prior source document',()=>{
  const old='进场触发 若已支付该牌的额外费用，或你的隐秘等级高于敌方玩家，则你获得目标印刷费用小于或等于⑤的敌方附属之操控权，你可以为该附属重新选择结附目标。';
  const corrected=old.replace('等于⑤','等于③');
  assert.deepEqual(intake.sourceCorrections,[{id:'CQ24',path:['fields','printedRuleTextLines',2],old,new:corrected},{id:'CQ24',path:['abilities',2,'printedText'],old,new:corrected}]);
  assert.equal(sha(JSON.stringify(beforeThiefTattooSpecifications(specs),null,2)+'\n'),'efad0bc7490873890fd231cb69a353051d32b84abd2959ce17e517bb9e850861');
  assert.equal(sha(JSON.stringify(priorSpecifications(specs),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');
  assert.equal(pool.sourceSha256,sha(read('docs/factions/card-specifications.json')));
});
test('both black cards retain exact original pixels and printed fields',()=>{
  for(const[id,hash,cost,unique,kind,loyalty]of[
    ['WM064','143494c4524088faf0bf21e3ae80cb81a24449b4c7bcadca251793920145ff02',6,false,'attachment',['黑色']],
    ['CQ24','6867f1ea9a7da6a1f811a0c342c80e2879a2c82607dcb5167ce4d9c602f8c86d',1,true,'character',['黑色','黑色']],
  ]){const d=pool.cards.find(c=>c.id===id),s=specs.cards[id];assert.equal(d.color,'黑');assert.equal(d.cost,cost);assert.equal(d.unique,unique);assert.equal(d.kind,kind);assert.deepEqual(d.loyalty,loyalty);
    assert.equal(d.text,s.fields.printedRuleTextLines.join('\n'));assert.deepEqual(d.icons,{permanent:s.fields.permanentIcons,temporary:s.fields.temporaryIcons});
    assert.equal(sha(read(s.sourceVerification.imagePath)),hash);assert.equal(sha(read(`web/public/cards/${id}.jpg`)),hash);
  }
});
test('current106 remains author WIP; question evidence does not infer printed zero',()=>{
  assert.equal(pool.engineVersion,currentImport20.engineVersion);assert.equal(pool.cardPoolVersion,currentImport20.cardPoolVersion);
  assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert.equal(intake.priorReference20AuthorNativeWipComplete,8);assert.equal(intake.potentialReference20AuthorNativeWipComplete,10);assert.equal(intake.current50ReviewedNativeClosed,38);
  assert.equal(intake.published,false);assert.equal(intake.independentReview,'notRun');assert.equal(intake.WASM,'notBuiltOrRun');assert.equal(intake.Worker,'notBuiltOrRun');assert.equal(intake.references.SuezCodeOrArtCopied,false);
  assert.equal(sha(read(intake.CQ24SecretPlanPublicCost.source)),intake.CQ24SecretPlanPublicCost.sha256);
  assert.match(intake.CQ24SecretPlanPublicCost.printedText,/其印刷费用为①/);
  const q=JSON.parse(read('docs/factions/XQ19_DASH_COST_QUESTION_2026-10-24.json'));assert.equal(q.name,'熵力化身');assert.equal(q.comparisons.length,2);assert.match(q.comparisons[1].card,/淘汰弱者/);
  assert.equal(specs.cards.XQ19.costAppearance,'printedDash');assert.equal(specs.cards.XQ19.fields.printedCost,null);
  for(const[path,digest]of Object.entries(q.sourceSha256))assert.equal(sha(read(path)),digest);
});
