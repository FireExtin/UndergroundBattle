import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const read=p=>readFileSync(root+p),json=p=>JSON.parse(read(p));
const baseline=JSON.parse(execFileSync('git',['show','27026fd864b96da5950c07ca1f58a8315c27a70e:rust-game/data/cards.json'],{cwd:root}));
const pool=json('rust-game/data/cards.json'),spec=json('docs/factions/card-specifications.json').cards.WM062;
test('WM062 adds one complete black card and preserves every former 145 definition, deck and world byte value',()=>{
 const {cards,engineVersion,cardPoolVersion,...meta}=pool;
 const {cards:old,engineVersion:oldEngine,cardPoolVersion:oldPool,...oldMeta}=baseline;
 assert.equal(cards.length,147);assert.equal(new Set(cards.map(c=>c.id)).size,147);
 assert.deepEqual(cards.filter(c=>!['WM062','JC054'].includes(c.id)),old);assert.deepEqual(meta,oldMeta);
 assert.equal(engineVersion,'rust-v0.2.82-response-source-preview-candidate');
 assert.equal(cardPoolVersion,'limited-v2.74-jc054-complete-conceal-candidate');
});
test('WM062 full runtime fields agree with its exact original without promoting archival acceptance',()=>{
 const f=spec.fields,s=spec.sourceVerification;
 assert.deepEqual(pool.cards.find(c=>c.id==='WM062'),{id:'WM062',name:'跑路',kind:'spell',subtypes:['阴谋'],cost:2,loyalty:['黑色'],loyaltyText:'黑色',color:'黑',magic:'',icons:{permanent:f.permanentIcons,temporary:f.temporaryIcons},text:f.printedRuleTextLines[0],keywords:[],unique:false,supported:true,source:{file:'docs/factions/card-specifications.json',record:'WM062',image:s.imagePath,sha256:s.imageSha256}});
 assert.equal(spec.selectionEnabled,false);assert.equal(spec.gameAcceptance,'notAccepted');assert.equal(spec.strategyUiPlaytest,'notRun');
 assert.deepEqual(spec.openQuestionIds,[]);assert.deepEqual(s.blockingFields,[]);
});
test('WM062 art is exact and all 146 admitted scans retain their pinned source identity',()=>{
 const s=spec.sourceVerification,b=read(s.imagePath);
 assert.equal(createHash('sha256').update(b).digest('hex'),'a277463a0dc41c9eba202f7d46d4088779a7d756ca835d5d6a3e57611bf7f335');
 assert.deepEqual(read('web/public/cards/WM062.jpg'),b);
 const checked=validateAdmittedScans();assert.equal(checked.checkedCards,147);assert.equal(checked.manifestEntries,158);
});
