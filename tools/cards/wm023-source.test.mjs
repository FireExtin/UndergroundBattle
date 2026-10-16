import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.WM023,pool=json('rust-game/data/cards.json');
test('WM023 preserves all 134 definitions and metadata from cumulative a967e13',()=>{
  const {cards,cardPoolVersion,engineVersion,...metadata}=pool;
  assert.equal(cards.length,146);assert.equal(new Set(cards.map(c=>c.id)).size,146);
  assert.equal(digest(cards.filter(bq035 => !["BQ035", "JZ46", "WM062"].includes(bq035.id)).filter(c=>!['WM023','XQ25','WM038','JZ35','JC052','JC037','JC060','JC095','JC046'].includes(c.id))),'f96f90e522616cb6c44ac92683567b5725f4389781f03436b82d4079f0603ca6');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(engineVersion,'rust-v0.2.80-wm062-conceal-adjacent-move-candidate');assert.equal(cardPoolVersion,'limited-v2.73-wm062-conceal-adjacent-move-candidate');
});
test('WM023 fields match its complete pinned original with no unresolved source question',()=>{
  assert.equal(digest({fields:spec.fields,sourceVerification:spec.sourceVerification,edition:spec.edition}),'5f5f54a025623911fd500fa6ed44e269e87bbda1e535e6d70aade46315aa7207');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(spec.sourceVerification.blockingFields,[]);assert.equal(spec.sourceVerification.status,'completeGameplayFieldsForPinnedImage');
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='WM023'),{id:'WM023',name:f.name,subtitle:f.subtitle,kind:'character',subtypes:f.subtypes,cost:f.printedCost,
    loyalty:['蓝色'],loyaltyText:'蓝色',color:f.colorKey,magic:'',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:f.defense,
    text:f.printedRuleTextLines.join('\n'),keywords:f.printedKeywords,unique:true,
    source:{file:'docs/factions/card-specifications.json',record:'WM023',image:s.imagePath,sha256:s.imageSha256}});
});
test('WM023 public bytes preserve the original and all admitted scans validate',()=>{
  const original=read(spec.sourceVerification.imagePath);assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/WM023.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,146);assert.ok(r.checkedIds.includes('WM023'));
});
