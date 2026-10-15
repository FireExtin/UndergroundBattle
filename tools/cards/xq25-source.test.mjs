import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.XQ25,pool=json('rust-game/data/cards.json');
test('XQ25 preserves all 135 definitions and metadata from cumulative c51231c',()=>{
  const {cards,cardPoolVersion,engineVersion,...metadata}=pool;
  assert.equal(cards.length,141);assert.equal(new Set(cards.map(c=>c.id)).size,141);
  assert.equal(digest(cards.filter(c=>!['XQ25','WM038','JZ35','JC052','JC037','JC060'].includes(c.id))),'b570e06865d6ebd4097ebedc72acb70cc5ff5573b20a488c988349146343fc42');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(engineVersion,'rust-v0.2.75-jc060-occult-adviser-candidate');assert.equal(cardPoolVersion,'limited-v2.68-jc060-occult-adviser-candidate');
});
test('XQ25 fields match its complete pinned original with no unresolved source question',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'b1dad08dd37e3869abc1af9f6cd0f8e3bdacd7ae8c3b21dda7a83266be222697');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);assert.equal(spec.sourceVerification.status,'completeGameplayFieldsForPinnedImage');
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='XQ25'),{id:'XQ25',name:f.name,kind:'attachment',subtypes:f.subtypes,cost:f.printedCost,
    loyalty:['灰色','灰色'],loyaltyText:'灰色 / 灰色',color:f.colorKey,magic:'',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},
    text:f.printedRuleTextLines.join('\n'),keywords:f.printedKeywords,unique:false,
    source:{file:'docs/factions/card-specifications.json',record:'XQ25',image:s.imagePath,sha256:s.imageSha256}});
});
test('XQ25 public bytes preserve the original and all admitted scans validate',()=>{
  const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/XQ25.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,141);assert.ok(r.checkedIds.includes('XQ25'));
});
