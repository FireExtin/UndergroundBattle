import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { vampireSlice as intake, currentImport20, beforeVampireSpecifications, priorSpecifications } from './import20-spec-history.mjs';
const read = name => readFileSync(new URL('../../'+name, import.meta.url));
const sha = x => createHash('sha256').update(x).digest('hex');
const canonical = x => Array.isArray(x) ? x.map(canonical) : x && typeof x==='object'
  ? Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])) : x;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const specifications = JSON.parse(read('docs/factions/card-specifications.json'));

test('third pair preserves exact256 earlier records, public213 and the frozen20 roster', () => {
  assert.equal(intake.priorCatalogCount,256);
  const digest='afa26ebb791e5d947a8e8e3496fe890a4b2a136be4a26b558ee7dc9238afe229';
  assert.equal(intake.priorCardsCanonicalSha256,digest);
  assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,256)))),digest);
  assert.equal(new Set(pool.cards.map(c=>c.id)).size,263);
  assert.deepEqual(pool.cards.slice(256,258).map(c=>c.id),['WM025','WM024']);
  assert.deepEqual(pool.cards.slice(258,260).map(c=>c.id), ['BQ081','WM057']);
  assert.deepEqual(pool.cards.slice(260,261).map(c=>c.id), ['XG13']);
  assert.deepEqual(pool.cards.slice(261).map(c=>c.id), ['WM064','CQ24']);
  assert.equal(pool.cards.filter(c=>c.supported!==false).length,213);
  for (const id of intake.allSixUnadmitted) assert.equal(pool.cards.find(c=>c.id===id).supported,false);
  assert.equal(intake.authorTotalReference20Programs,6);assert.equal(intake.current50ReviewedNativeClosed,38);
  const original=JSON.parse(read('docs/evidence/reference-batch20-intake-20261023/candidate20.json'));
  for (const id of intake.newCards) assert.equal(original.cards.find(c=>c.id===id).inCurrentFourth50,false);
});

test('six exact image-backed corrections reconstruct both earlier whole-document identities', () => {
  const oldText='现身触发 袭击X，X等于敌方秘社上暴露标志的数量。若有其它角色因此效果死去，抓一张牌。';
  const newText=oldText.replace('其它角色','声望角色');
  assert.deepEqual(intake.sourceCorrections,[
    {id:'WM025',path:['fields','printedKeywords',1],old:'屏障',new:'隐秘条件：屏障'},
    {id:'WM025',path:['fields','printedRuleTextLines',0],old:'领袖；屏障',new:'领袖；【隐秘】屏障'},
    {id:'WM025',path:['abilities',0,'printedText'],old:'领袖；屏障',new:'领袖；【隐秘】屏障'},
    {id:'WM025',path:['fields','additionalPrintedSymbols'],old:[],new:[{id:'moon-barrier',status:'confirmed',printedText:'〔月牙〕屏障',interpretation:'本方秘社上有隐秘标志时才获得印刷屏障。'}]},
    {id:'WM024',path:['fields','printedRuleTextLines',1],old:oldText,new:newText},
    {id:'WM024',path:['abilities',1,'printedText'],old:oldText,new:newText},
  ]);
  assert.equal(sha(JSON.stringify(beforeVampireSpecifications(specifications),null,2)+'\n'),'e9c928bfa6bb225e6b298eccd19cb3c4ac341c6a36690393a48efbaa2dbf3c6c');
  assert.equal(sha(JSON.stringify(priorSpecifications(specifications),null,2)+'\n'),'809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c');
  assert.equal(pool.sourceSha256,sha(read('docs/factions/card-specifications.json')));
});

test('both blue vampire images and every mechanical printed field match their primary originals', () => {
  for (const [id,hash,cost,loyalty,unique,defense] of [
    ['WM025','419c1c6188dbd2d793a527da238757fbe61de3f1992fcc902e124103e1651622',8,['蓝色','蓝色','蓝色'],true,4],
    ['WM024','78c85bc44106d49f054e1dc090d8161bebfcfb605942d04f2611323d87822120',6,['蓝色'],false,2],
  ]) {
    const c=pool.cards.find(c=>c.id===id),s=specifications.cards[id];
    assert.equal(sha(read(s.sourceVerification.imagePath)),hash);assert.equal(sha(read(`web/public/cards/${id}.jpg`)),hash);
    assert.equal(c.cost,cost);assert.deepEqual(c.loyalty,loyalty);assert.equal(c.unique,unique);assert.equal(c.defense,defense);
    assert.equal(c.kind,'character');assert.equal(c.color,'蓝');assert.equal(c.text,s.fields.printedRuleTextLines.join('\n'));
    assert.deepEqual(c.subtypes,s.fields.subtypes);assert.deepEqual(c.icons,{permanent:s.fields.permanentIcons,temporary:s.fields.temporaryIcons});
    assert.deepEqual(c.keywords,s.fields.printedKeywords);
  }
  assert.equal(pool.cards.find(c=>c.id==='WM024').magic,'血');
  assert.deepEqual(pool.cards.find(c=>c.id==='WM025').keywords,['领袖','隐秘条件：屏障']);
});

test('current native identity is explicit WIP with no cross-platform or independent completion claim', () => {
  assert.equal(pool.engineVersion,currentImport20.engineVersion);
  assert.equal(pool.cardPoolVersion,currentImport20.cardPoolVersion);
  assert(read('rust-game/src/catalog.rs').toString().includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert.equal(intake.independentReview,'notRun');assert.equal(intake.published,false);
  for (const k of ['WASM','Worker']) assert.equal(intake[k],'notBuiltOrRun');assert.equal(intake.browser,'notRun');
  assert.equal(intake.references.SuezCodeOrArtCopied,false);
});
