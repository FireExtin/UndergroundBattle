import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=new URL('../../',import.meta.url),read=p=>readFileSync(new URL(p,root)),json=p=>JSON.parse(read(p));
const pool=json('rust-game/data/cards.json'),spec=json('docs/factions/card-specifications.json').cards.BQ035;

test('BQ035 retains its pinned blue loyalty and keeps source verification distinct from game acceptance',()=>{
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
  validateAdmittedScans();
});
