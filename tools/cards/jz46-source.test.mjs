import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const read=p=>readFileSync(root+p), json=p=>JSON.parse(read(p));
const original=JSON.parse(execFileSync('git',['show','0c98068f6b1d35ecef6ee512f2a6773ab7b5cdba:rust-game/data/cards.json'],{cwd:root}));
const pool=json('rust-game/data/cards.json'), spec=json('docs/factions/card-specifications.json').cards.JZ46;
test('JZ46 adds one whole definition while every original 144 card and deck/world metadata stays exact',()=>{
  const {cards,engineVersion,cardPoolVersion,...meta}=pool;
  const {cards:old,engineVersion:oldEngine,cardPoolVersion:oldPool,...oldMeta}=original;
  assert.equal(cards.length,147);assert.equal(new Set(cards.map(c=>c.id)).size,147);
  assert.deepEqual(cards.filter(c=>!['JZ46','WM062','JC054'].includes(c.id)),old);assert.deepEqual(meta,oldMeta);
  assert.equal(engineVersion,'rust-v0.2.81-jc054-complete-conceal-candidate');
  assert.equal(cardPoolVersion,'limited-v2.74-jc054-complete-conceal-candidate');
});
test('the entire JZ46 runtime matches the pinned image specification and preserves source acceptance status',()=>{
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='JZ46'),{id:'JZ46',name:'狴犴铜雕',kind:'attachment',subtypes:['雕像'],cost:2,loyalty:['灰色'],loyaltyText:'灰色',color:'灰',magic:'神圣',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},text:f.printedRuleTextLines.join('\n'),keywords:[],source:{file:'docs/factions/card-specifications.json',record:'JZ46',image:s.imagePath,sha256:s.imageSha256}});
  assert.deepEqual(f.permanentIcons,{investigation:0,combat:0,influence:1});
  assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
});
test('JZ46 art is byte-exact and all 145 admitted scans remain pinned',()=>{
  const original=read(spec.sourceVerification.imagePath);
  assert.equal(createHash('sha256').update(original).digest('hex'),'756bdb34307c84a5c5f1e9cc1b7e13f9112ae727e934847f6993169ae1451785');
  assert.deepEqual(read('web/public/cards/JZ46.jpg'),original);
  const checked=validateAdmittedScans();assert.equal(checked.checkedCards,147);assert.equal(checked.manifestEntries,158);
});
