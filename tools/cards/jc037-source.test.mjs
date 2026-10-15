import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JC037,pool=json('rust-game/data/cards.json');
test('JC037 preserves all 139 accepted0939 definitions, decks, world and metadata',()=>{
 const {cards,engineVersion,cardPoolVersion,...metadata}=pool;
 assert.equal(cards.length,143);assert.equal(new Set(cards.map(c=>c.id)).size,143);
 assert.equal(digest(cards.filter(c=>!['JC037','JC060','JC095','JC046'].includes(c.id))),'8a45c42f19aed17a700793ab46ee89f5eff99b2073bdc85dfc03f1c4318246b0');
 assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
 assert.equal(engineVersion,'rust-v0.2.77-jc046-mandatory-sacrifice-candidate');assert.equal(cardPoolVersion,'limited-v2.70-jc046-mandatory-sacrifice-candidate');
});
test('JC037 complete gold-name Leader and continuous text match the pinned037/135 original',()=>{
 assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'d38386f43e8b9856b416a567def8e0d7f2e53bb741c3b974edb588ea5ab79978');
 assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);
 const f=spec.fields,s=spec.sourceVerification;
 assert.deepEqual(pool.cards.find(c=>c.id==='JC037'),{id:'JC037',name:f.name,subtitle:f.subtitle,kind:'character',subtypes:['吸血鬼'],cost:6,
 loyalty:['蓝色','蓝色','蓝色'],loyaltyText:'蓝色 蓝色 蓝色',color:'蓝',magic:'鲜血',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:3,
 text:f.printedRuleTextLines.join('\n'),keywords:['领袖'],unique:true,source:{file:'docs/factions/card-specifications.json',record:'JC037',image:s.imagePath,sha256:s.imageSha256}});
});
test('JC037 public image retains original bytes and all admitted scans validate',()=>{
 const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
 assert.deepEqual(read('web/public/cards/JC037.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,143);assert.equal(r.manifestEntries,154);
});
