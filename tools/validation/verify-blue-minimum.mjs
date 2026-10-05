// Exact whole native/actual33 transitions, restore views and rejected commands.
// Explicit offline fixtures; no public-room or deployed-site claim.
import {readFileSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
const root=resolve(process.argv[2]),evidence=resolve(process.argv[3]);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
async function load(folder){const k=await import(pathToFileURL(root+'/rust-game-wasm/'+folder+'/hegemony_wasm.js'));const bytes=readFileSync(root+'/rust-game-wasm/'+folder+'/hegemony_wasm_bg.wasm');k.initSync({module:bytes});return{k,bytes,catalog:JSON.parse(k.catalog())};}
const {k,bytes,catalog}=await load('pkg'),{k:old,bytes:frozen,catalog:prior}=await load('legacy-v0.2.32');
assert.equal(catalog.engineVersion,'rust-v0.2.33-blue-minimum-candidate');assert.equal(catalog.cardPoolVersion,'limited-v2.30-blue-minimum-candidate');
assert.equal(catalog.cards.length,96);assert.equal(catalog.societies.length,8);
assert.deepEqual(catalog.cards.filter(c=>!['JC032','JZ24'].includes(c.id)),prior.cards);
assert.deepEqual(catalog.societies.filter(c=>c.id!=='MSJC03'),prior.societies);assert.deepEqual(catalog.decks,prior.decks);
assert.equal(sha(frozen),'fd375f5e05247c4e78b5cbd65f3da2e4393a729d49b4b2203347dd6d0e62fc89');
const savedFrozen=JSON.parse(readFileSync(evidence+'/frozen-before.json','utf8'));
for(const [file,hash] of Object.entries(savedFrozen))assert.equal(sha(readFileSync(root+'/'+file)),hash,file);
const scans=JSON.parse(readFileSync(root+'/web/public/card-scans.json','utf8'));
const oldScans=JSON.parse(execFileSync('git',['show','713be787f47d4188640922a5d373662bd04ffe38:web/public/card-scans.json'],{cwd:root,encoding:'utf8'}));
assert.equal(Object.keys(scans).length,104);
for(const [id,scan]of Object.entries(oldScans)){assert.deepEqual(scans[id],scan);assert.equal(sha(readFileSync(root+'/web/public'+scan.url)),scan.sha256,id);}
for(const [id,hash]of Object.entries({JC032:'ebeb0a13c62cdf10569f4aa9f300998aff53a65788d756f055e9696f35489b4b',JZ24:'e01995d70e06a6b179dcbbf81c3df73ab548269fc0efe9fafc33e2e2f3c8722d',MSJC03:'07463a18a5cc6726f098bedc10304ef6c4032929cff72535d5f9227338e45f29'})){
 assert.equal(scans[id].sha256,hash);assert.equal(sha(readFileSync(root+'/web/public/cards/'+id+'.jpg')),hash);assert.equal(sha(readFileSync(root+'/'+scans[id].source)),hash);
}
let commands=0,checkpoints=0,rejects=0,views=0;
for(const file of readdirSync(evidence+'/native-bodies').sort()){
 const j=JSON.parse(readFileSync(evidence+'/native-bodies/'+file,'utf8'));
 if(file.startsWith('checkpoint-')){for(let s=0;s<4;s++){assert.deepEqual(JSON.parse(k.view(j.state,s)),j.views[s],file+' seat'+s);views++;}checkpoints++;}
 else if(file.startsWith('step-')||file.startsWith('reject-')){
  const actual=JSON.parse(k.applyRoom(j.state,j.seat,JSON.stringify(j.command),'0'));assert.deepEqual(actual,j.expected,file);
  for(let s=0;s<4;s++){assert.deepEqual(JSON.parse(k.view(actual.state,s)),j.views[s],file+' seat'+s);views++;}
  if(file.startsWith('reject-')){assert.equal(actual.outcome,'rejected');assert.equal(actual.state,j.state);rejects++;}else commands++;
 }else throw Error('Unknown body '+file);
}
const fixtures=JSON.parse(readFileSync(root+'/web/src/game/blueMinimumTest.fixture.json','utf8')).fixtures;
for(const row of fixtures)for(let s=0;s<4;s++){assert.deepEqual(JSON.parse(k.view(row.state,s)),row.views[s],row.kind+' seat'+s);views++;}
for(const name of ['jc032','jz24']){
 const row=JSON.parse(readFileSync(evidence+'/frontend-rows/host-receipt-'+name+'.json','utf8'));
 const actual=JSON.parse(k.applyRoom(row.stateBefore,0,JSON.stringify(row.command),'0'));
 assert.deepEqual(actual,row.expectedTransition);assert.equal(actual.state,row.stateAfter);assert.deepEqual(row.originalView,row.duplicateView);assert.equal(row.savedReceiptCount,1);
 const stale=JSON.parse(k.applyRoom(row.stateAfter,0,JSON.stringify(row.command),'0'));
 assert.equal(stale.outcome,'rejected');assert.equal(stale.state,row.stateAfter);
}
const lobby=JSON.parse(old.newGame('old32-offline','LOCAL','teams','P0','watchers','1'));
assert.throws(()=>k.view(lobby.state,0));assert.deepEqual(JSON.parse(old.view(lobby.state,0)),lobby.view);
assert(commands>0&&checkpoints>0&&rejects>30&&fixtures.length>=10);
console.log(JSON.stringify({ok:true,wasmSha256:sha(bytes),wasmBytes:bytes.length,commands,checkpoints,rejects,comparedFullSeatViews:views,frontendWholeFixtures:fixtures.length,allPrior94DefinitionsExact:true,allSevenPriorSocietiesExact:true,allFivePresetsExact:true,allPrior101OriginalScansExact:true,preservedFrozenAbiFiles:Object.keys(savedFrozen).length,realHostReceiptKinds:['jc032','jz24'],frozen32Exact:true,newKernelRejectsOldIdentities:true,scope:'Whole offline native/actual33 bodies; SQLite receipt/restart fixtures; no deployed/public-room claim'},null,2));
