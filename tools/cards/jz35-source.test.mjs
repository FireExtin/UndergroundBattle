import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=n=>readFileSync(new URL(n,root)),json=n=>JSON.parse(read(n));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const spec=json('docs/factions/card-specifications.json').cards.JZ35,pool=json('rust-game/data/cards.json');
test('JZ35 preserves all 137 definitions and metadata from d303545',()=>{
  const {cards,cardPoolVersion,engineVersion,...metadata}=pool;
  assert.equal(cards.length,147);assert.equal(new Set(cards.map(c=>c.id)).size,147);
  assert.equal(digest(cards.filter(bq035 => !["BQ035", "JZ46", "WM062", "JC054"].includes(bq035.id)).filter(c=>!['JZ35','JC052','JC037','JC060','JC095','JC046'].includes(c.id))),'1740110194f78c5a71f166a6efec4f0ffa5a9f6c32a4d2f59eb16f88152d9787');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(engineVersion,'rust-v0.2.83-msjc04-msjc05-societies-candidate');assert.equal(cardPoolVersion,'limited-v2.75-msjc04-msjc05-societies-candidate');
});
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
  assert.deepEqual(read('web/public/cards/JZ35.jpg'),original);const r=validateAdmittedScans();assert.equal(r.checkedCards,147);assert.ok(r.checkedIds.includes('JZ35'));
});
