import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.WM038,pool=json('rust-game/data/cards.json');
test('WM038 preserves all 136 definitions and metadata from cumulative 912f230',()=>{
  const {cards,cardPoolVersion,engineVersion,...metadata}=pool;
  assert.equal(cards.length,143);assert.equal(new Set(cards.map(c=>c.id)).size,143);
  assert.equal(digest(cards.filter(c=>!['WM038','JZ35','JC052','JC037','JC060','JC095','JC046'].includes(c.id))),'9d71a64108f1850060337fda12466c5e588c0f8b3e7d762f898ea678a0bc7b37');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(engineVersion,'rust-v0.2.77-jc046-mandatory-sacrifice-candidate');assert.equal(cardPoolVersion,'limited-v2.70-jc046-mandatory-sacrifice-candidate');
});
test('WM038 complete fields match the pinned original and have no unresolved source question',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'d7ac78da2029dee8ce26aaa645df0261c1ba21dba9db34f276cab404bf0f7143');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);assert.equal(spec.sourceVerification.status,'completeGameplayFieldsForPinnedImage');
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='WM038'),{id:'WM038',name:f.name,kind:'attachment',subtypes:f.subtypes,cost:f.printedCost,
    loyalty:['红色'],loyaltyText:'红色',color:f.colorKey,magic:'鲜血',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},
    text:f.printedRuleTextLines.join('\n'),keywords:f.printedKeywords,unique:false,
    source:{file:'docs/factions/card-specifications.json',record:'WM038',image:s.imagePath,sha256:s.imageSha256}});
});
test('WM038 public bytes preserve the original and all admitted scans validate',()=>{
  const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/WM038.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,143);assert.ok(r.checkedIds.includes('WM038'));
});
