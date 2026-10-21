import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.WM056,pool=json('rust-game/data/cards.json');

test('WM056 complete fields and exact printed source have no unresolved question',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'a385a5674cd8f00b935e4e78a487aee46ce6a1d2d94a0bcf0a3a79511be8dafc');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
  assert.equal(spec.sourceVerification.status,'completeGameplayFieldsForPinnedImage');
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='WM056'),{id:'WM056',name:f.name,kind:'character',subtypes:f.subtypes,
    cost:f.printedCost,loyalty:['黑色'],loyaltyText:'黑色',color:f.colorKey,magic:'',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},
    defense:f.defense,text:f.printedRuleTextLines.join('\n'),keywords:f.printedKeywords,unique:false,
    source:{file:'docs/factions/card-specifications.json',record:'WM056',image:s.imagePath,sha256:s.imageSha256}});
});
test('WM056 public image retains original bytes and all admitted scans validate',()=>{
  const original=read(spec.sourceVerification.imagePath);
  assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/WM056.jpg'),original);
  const result=validateAdmittedScans();assert.ok(result.checkedIds.includes('WM056'));
});
