import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JC052,pool=json('rust-game/data/cards.json');

test('JC052 full gold-name Leader and entry text match its pinned original',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'45cb9fdb7a38fd4f2f61169b230ecfcc813f0e6b01db7468c942c5e44a223520');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='JC052'),{id:'JC052',name:f.name,subtitle:f.subtitle,kind:'character',subtypes:f.subtypes,cost:7,
    loyalty:['红色','红色','红色','红色'],loyaltyText:'红色 红色 红色 红色',color:'红',magic:'星辰',
    icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:1,text:f.printedRuleTextLines.join('\n'),keywords:['领袖'],unique:true,
    source:{file:'docs/factions/card-specifications.json',record:'JC052',image:s.imagePath,sha256:s.imageSha256}});
});
test('JC052 source image and public copy are identical and all scans validate',()=>{
  const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/JC052.jpg'),original);const r=validateAdmittedScans();
});
