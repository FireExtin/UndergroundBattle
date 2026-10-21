import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JC095,pool=json('rust-game/data/cards.json');

test('JC095 complete gold-name Leader and Renown face matches pinned095/135 original',()=>{
 assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'9720d1ed8a4aec97e480ebe7ed5384716951c5a46c2e33593b2f9d1059a7dc18');
 assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
 const f=spec.fields,s=spec.sourceVerification;
 assert.deepEqual(pool.cards.find(c=>c.id==='JC095'),{id:'JC095',name:f.name,subtitle:f.subtitle,kind:'character',subtypes:['人类','罪犯'],cost:7,
 loyalty:['黑色','黑色','黑色'],loyaltyText:'黑色 黑色 黑色',color:'黑',magic:'',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:2,
 text:f.printedRuleTextLines.join('\n'),keywords:['领袖','声望'],unique:true,source:{file:'docs/factions/card-specifications.json',record:'JC095',image:s.imagePath,sha256:s.imageSha256}});
});
test('JC095 public scan is the byte-identical original and all admitted scans remain valid',()=>{
 const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
 assert.deepEqual(read('web/public/cards/JC095.jpg'),original);const r=validateAdmittedScans();
});
