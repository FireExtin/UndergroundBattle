import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JZ35,pool=json('rust-game/data/cards.json');

test('JZ35 complete fields match its original without unresolved source questions',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'93579e39a80ad3515f9e5b82f3770eb683de5a7ad1fed579811d99f1ebe2e296');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='JZ35'),{id:'JZ35',name:f.name,kind:'spell',subtypes:f.subtypes,cost:f.printedCost,
    loyalty:['红色'],loyaltyText:'红色',color:f.colorKey,magic:'心灵',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},
    text:f.printedRuleTextLines.join('\n'),keywords:f.printedKeywords,unique:false,
    source:{file:'docs/factions/card-specifications.json',record:'JZ35',image:s.imagePath,sha256:s.imageSha256}});
});
test('JZ35 public face is the exact pinned original and all admitted scans validate',()=>{
  const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/JZ35.jpg'),original);const r=validateAdmittedScans();assert.ok(r.checkedIds.includes('JZ35'));
});
