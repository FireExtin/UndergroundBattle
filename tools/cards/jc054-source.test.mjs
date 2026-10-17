import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const read=p=>readFileSync(root+p),json=p=>JSON.parse(read(p));
const previous=JSON.parse(execFileSync('git',['show','f89b4371c8f1a2139c7a89edc275c169fd098b10:rust-game/data/cards.json'],{cwd:root}));
const pool=json('rust-game/data/cards.json'),spec=json('docs/factions/card-specifications.json').cards.JC054;
test('JC054 adds only one complete red card while preserving all 146 earlier cards, decks and world',()=>{
  const {cards,cardPoolVersion,engineVersion,...metadata}=pool;
  const {cards:old,cardPoolVersion:oldPool,engineVersion:oldEngine,...oldMetadata}=previous;
  assert.equal(cards.length,147);assert.equal(new Set(cards.map(c=>c.id)).size,147);
  assert.deepEqual(cards.filter(c=>c.id!=='JC054'),old);assert.deepEqual(metadata,oldMetadata);
  assert.equal(cardPoolVersion,'limited-v2.74-jc054-complete-conceal-candidate');
  assert.equal(engineVersion,'rust-v0.2.81-jc054-complete-conceal-candidate');
});
test('JC054 complete printed fields stay pinned without promoting source-only acceptance or admitting neutral/dual cards',()=>{
  const f=spec.fields,s=spec.sourceVerification;
  assert.deepEqual(pool.cards.find(c=>c.id==='JC054'),{id:'JC054',name:'蒙古死亡蠕虫',kind:'character',subtypes:['蠕虫'],cost:4,
    loyalty:[],loyaltyText:'无',color:'红',magic:'',icons:{permanent:{investigation:0,combat:1,influence:0},temporary:{investigation:0,combat:1,influence:0}},
    defense:1,text:f.printedRuleTextLines.join('\n'),keywords:[],unique:false,supported:true,
    source:{file:'docs/factions/card-specifications.json',record:'JC054',image:s.imagePath,sha256:s.imageSha256}});
  assert.equal(f.printedRuleTextLines.length,2);assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
  assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(s.blockingFields,[]);
});
test('JC054 scan is exactly its reviewed original and all 147 ordinary scans retain identity',()=>{
  const bytes=read(spec.sourceVerification.imagePath);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),'3279d70ffe4b766783cc64262165c7d10cd8b08b1c00dc915397ceaadd3650a2');
  assert.deepEqual(bytes,read('web/public/cards/JC054.jpg'));
  const result=validateAdmittedScans();assert.equal(result.checkedCards,147);assert.equal(result.manifestEntries,158);
});
