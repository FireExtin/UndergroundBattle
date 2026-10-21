import {catalogBeforeAbi84} from './historical-catalog.mjs';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JC052,pool=catalogBeforeAbi84(json('rust-game/data/cards.json'));
test('JC052 preserves all 138 prior definitions and pool metadata from 9890163',()=>{
  const {cards,engineVersion,cardPoolVersion,...metadata}=pool;
  assert.equal(cards.length,147);assert.equal(new Set(cards.map(c=>c.id)).size,147);
  assert.equal(digest(cards.filter(bq035 => !["BQ035", "JZ46", "WM062", "JC054"].includes(bq035.id)).filter(c=>!['JC052','JC037','JC060','JC095','JC046'].includes(c.id))),'56c4165cd44e7d80bb222da6619c0ad9faf0c059162997bdefda140db133e4df');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(engineVersion,'rust-v0.2.90-audit-correctness-candidate');assert.equal(cardPoolVersion,'limited-v2.81-jc035-preparation-candidate');
});
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
  assert.deepEqual(read('web/public/cards/JC052.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,153);assert.equal(r.manifestEntries,166);
});
