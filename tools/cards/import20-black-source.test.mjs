import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { blackSlice as intake, currentImport20, beforeBlackSpecifications, priorSpecifications } from './import20-spec-history.mjs';
const read=n=>readFileSync(new URL('../../'+n,import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const pool=JSON.parse(read('rust-game/data/cards.json'));
const specifications=JSON.parse(read('docs/factions/card-specifications.json'));
test('black pair retains exact258 prior records and public213, without changing the frozen20 roster',()=>{
  assert.equal(intake.priorCatalogCount,258);
  const digest='a3d6acd39e625636da9e8c9fae9c14bac2c83b1ae56ff2c71d828cece9e50b17';
  assert.equal(intake.priorCardsCanonicalSha256,digest);assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,258)))),digest);
  assert.equal(new Set(pool.cards.map(c=>c.id)).size,261);assert.deepEqual(pool.cards.slice(258,260).map(c=>c.id),['BQ081','WM057']);
  assert.deepEqual(pool.cards.slice(260).map(c=>c.id), ['XG13']);
  assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);
  for(const id of intake.allEightUnadmitted)assert.equal(pool.cards.find(c=>c.id===id).supported,false);
  assert.equal(intake.current50ReviewedNativeClosed,38);assert.equal(intake.published,false);
});
test('three BQ081 corrections preserve every other byte of both earlier whole specification identities',()=>{
  const old='隐秘+1：在本次隐秘对抗后，将你牌库顶的三张牌置于本方墓地。';const corrected=old.replace('三张牌','一张牌');
  assert.deepEqual(intake.sourceCorrections,[
    {id:'BQ081',path:['fields','printedRuleTextLines',0],old,new:corrected},
    {id:'BQ081',path:['abilities',0,'printedText'],old,new:corrected},
    {id:'BQ081',path:['fields','additionalPrintedSymbols',0,'interpretation'],old:'隐秘地区势力栏，独立于能力图标。',new:'隐秘对抗展示加值，独立于地区势力及能力图标。'},
  ]);
  assert.equal(sha(JSON.stringify(beforeBlackSpecifications(specifications),null,2)+'\n'),'cac18734aa77c5d0aa133bee95f9fe1d37ad3253aaf8ce6f8250f4a48be45b8c');
  assert.equal(sha(JSON.stringify(priorSpecifications(specifications),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');
  assert.equal(pool.sourceSha256,sha(read('docs/factions/card-specifications.json')));
});
test('black pair prints and original pixels are exact and ordinary card registration is distinct from acceptance',()=>{
  for(const[id,hash,cost,subtypes,magic]of[
    ['BQ081','b756871bdbc90a17a24c219ec3380f09e999ab7b64042335bb9f679cea7dac4f',3,['不死生物','食尸鬼','植物'],'死亡'],
    ['WM057','5ec31d37db587a2e8859534ceba7fec5a670845e9e3959d2733aa9cc6026c697',2,['人类','罪犯','工程师'],''],
  ]){
    const c=pool.cards.find(c=>c.id===id),s=specifications.cards[id];assert.equal(c.color,'黑');assert.equal(c.kind,'character');assert.equal(c.cost,cost);assert.equal(c.magic,magic);
    assert.equal(c.defense,1);assert.deepEqual(c.loyalty,['黑色']);assert.deepEqual(c.subtypes,subtypes);assert.equal(c.unique,false);
    assert.equal(c.text,s.fields.printedRuleTextLines.join('\n'));assert.deepEqual(c.icons,{permanent:s.fields.permanentIcons,temporary:s.fields.temporaryIcons});
    assert.equal(sha(read(s.sourceVerification.imagePath)),hash);assert.equal(sha(read(`web/public/cards/${id}.jpg`)),hash);
  }
  assert.match(specifications.cards.BQ081.fields.printedRuleTextLines[0],/一张牌/);
});
test('latest104 is unadmitted native WIP and unresolved enemy permission is not silently granted',()=>{
  assert.equal(pool.engineVersion,currentImport20.engineVersion);assert.equal(pool.cardPoolVersion,currentImport20.cardPoolVersion);
  assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert.equal(intake.WM057EnemyPermission,'pendingUserRuling; no enemy activation opened');assert.equal(intake.ruleQuestion.status,'askedAsyncAwaitingReply');
  assert.equal(intake.priorReference20AuthorNativeWipComplete,6);assert.equal(intake.independentReview,'notRun');
  for(const k of ['WASM','Worker'])assert.equal(intake[k],'notBuiltOrRun');assert.equal(intake.browser,'notRun');assert.equal(intake.references.SuezCodeOrArtCopied,false);
});
