import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=p=>readFileSync(new URL(p,root)),json=p=>JSON.parse(read(p));
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const digest=v=>createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
const pool=json('rust-game/data/cards.json'),specs=json('docs/factions/card-specifications.json').cards,spec=specs.BQ035;
test('BQ035 adds exactly one definition and preserves every prior 143 card and all deck/world metadata',()=>{
  const {cards,engineVersion,cardPoolVersion,...metadata}=pool;
  assert.equal(cards.length,144);assert.equal(new Set(cards.map(c=>c.id)).size,144);
  assert.equal(digest(cards.filter(c=>c.id!=='BQ035')),'927fff6b3556ca40c0bacadfb9ebc83aa6e4b01f18540f7c6f678d4731ef43e9');
  assert.equal(digest(metadata),'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(engineVersion,'rust-v0.2.78-bq035-vampire-mobility-candidate');assert.equal(cardPoolVersion,'limited-v2.71-bq035-vampire-mobility-candidate');
});
test('BQ035 corrects only its pinned blue loyalty and keeps source verification distinct from game acceptance',()=>{
  const {BQ035,...others}=specs;
  assert.equal(digest(others),'8ab029b3a76d90de5f39dee63b769b7bcc05fe91c62bb93e8f3273b3e6c66b7e');
  assert.deepEqual(spec.fields.loyalty,[{kind:'color',value:'蓝',count:1}]);
  assert.deepEqual(spec.sourceVerification.fieldCorrections[0].previous,[]);
  assert.equal(spec.sourceVerification.imageSha256,'52500e1a2eac4bb3ab1e9dc56c971669cea058b48f328ff09c42ebd24b7761b5');
  assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.deepEqual(spec.openQuestionIds,[]);
});
test('BQ035 runtime fields include the entire print rather than only its two keywords',()=>{
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='BQ035'),{id:'BQ035',name:f.name,kind:'character',subtypes:['吸血鬼','猎手'],cost:5,
    loyalty:['蓝色'],loyaltyText:'蓝色',color:'蓝',magic:'鲜血',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},defense:3,
    text:f.printedRuleTextLines.join('\n'),keywords:['机动','威名'],source:{file:'docs/factions/card-specifications.json',record:'BQ035',image:s.imagePath,sha256:s.imageSha256}});
});
test('BQ035 original art and all existing public scans remain exact pinned bytes',()=>{
  const original=read(spec.sourceVerification.imagePath);
  assert.equal(createHash('sha256').update(original).digest('hex'),spec.sourceVerification.imageSha256);
  assert.deepEqual(read('web/public/cards/BQ035.jpg'),original);
  const r=validateAdmittedScans();assert.equal(r.checkedCards,144);assert.equal(r.manifestEntries,155);
});
