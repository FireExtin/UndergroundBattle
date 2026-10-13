import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url);
const read=name=>readFileSync(new URL(name,root));
const json=name=>JSON.parse(read(name));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'
  ?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JC094;
const pool=json('rust-game/data/cards.json');

test('JC094 preserves all 132 definitions and metadata from cumulative 3860acd',()=>{
  const {cards,cardPoolVersion,engineVersion,...metadata}=pool;
  assert.equal(cards.length,135);assert.equal(new Set(cards.map(c=>c.id)).size,135);
  assert.equal(digest(cards.filter(c=>!['JC094','WM056','WM023'].includes(c.id))),'27e4e973b65e71fc9c0194d6d52b72bc6faeb618eec608d2b456f1cbc713f189');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(cardPoolVersion,'limited-v2.62-wm023-unique-hand-entry-candidate');
  assert.equal(engineVersion,'rust-v0.2.69-wm023-unique-hand-entry-candidate');
});
test('JC094 matches the complete pinned original with no unresolved source question',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),
    '4c51f421f43f5a16f3b0c9ce9b8afc1028f3d9a68eeba7187e4a518de181a4d4');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
  assert.equal(spec.sourceVerification.status,'completeGameplayFieldsForPinnedImage');
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='JC094'),{
    id:'JC094',name:f.name,subtitle:f.subtitle,kind:'character',subtypes:f.subtypes,cost:f.printedCost,
    loyalty:['黑色','黑色','黑色'],loyaltyText:'黑色 / 黑色 / 黑色',color:f.colorKey,magic:'死亡',
    icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:f.defense,
    text:f.printedRuleTextLines.join('\n'),keywords:f.printedKeywords,unique:f.nameInk==='gold',
    source:{file:'docs/factions/card-specifications.json',record:'JC094',image:s.imagePath,sha256:s.imageSha256}});
});
test('JC094 original and public bytes agree and all admitted scans remain valid',()=>{
  const original=read(spec.sourceVerification.imagePath);
  assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/JC094.jpg'),original);
  const result=validateAdmittedScans();assert.equal(result.checkedCards,135);assert.ok(result.checkedIds.includes('JC094'));
});
