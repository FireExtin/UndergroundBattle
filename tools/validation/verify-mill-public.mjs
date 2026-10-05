// Exact offline native/actual34 commands, whole seat views and frozen routing.
import {readFileSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root=resolve(process.argv[2]),evidence=resolve(process.argv[3]);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
async function load(folder){
 const k=await import(pathToFileURL(root+'/rust-game-wasm/'+folder+'/hegemony_wasm.js'));
 const bytes=readFileSync(root+'/rust-game-wasm/'+folder+'/hegemony_wasm_bg.wasm');
 k.initSync({module:bytes});return {k,bytes,catalog:JSON.parse(k.catalog())};
}
const {k,bytes,catalog}=await load('pkg');const previous=await load('legacy-v0.2.33');
assert.equal(catalog.engineVersion,'rust-v0.2.34-mill-public-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.31-mill-public-candidate');
assert.equal(catalog.cards.length,98);assert.equal(catalog.societies.length,8);
assert.deepEqual(catalog.cards.filter(c=>!['XQ36','XQ46'].includes(c.id)),previous.catalog.cards);
assert.deepEqual(catalog.societies,previous.catalog.societies);assert.deepEqual(catalog.decks,previous.catalog.decks);
assert.equal(sha(previous.bytes),'2bdf206c232747045c481c47d3493be18bec9fea263293866ca4d95bf824c2bb');
const frozen=JSON.parse(readFileSync(evidence+'/frozen-before.json','utf8'));
for(const [file,hash]of Object.entries(frozen))assert.equal(sha(readFileSync(root+'/'+file)),hash,file);
const scans=JSON.parse(readFileSync(root+'/web/public/card-scans.json','utf8'));
const oldScans=JSON.parse(readFileSync(evidence+'/base-card-scans.json','utf8'));
assert.equal(Object.keys(scans).length,106);
for(const [id,scan]of Object.entries(oldScans)){assert.deepEqual(scans[id],scan);assert.equal(sha(readFileSync(root+'/web/public'+scan.url)),scan.sha256,id);}
for(const [id,hash]of Object.entries({XQ36:'84b611615ecdbdf7940022160522187a28edb46322525e540fed1935de0526b9',XQ46:'d58e71df94753dde04595f5aa84de4039fcbbe594883acc018e2c06ddb103a89'})){
 assert.equal(scans[id].sha256,hash);assert.equal(sha(readFileSync(root+'/web/public/cards/'+id+'.jpg')),hash);
 assert.equal(sha(readFileSync(root+'/'+scans[id].source)),hash);
}
let commands=0,checkpoints=0,rejects=0,views=0;
for(const file of readdirSync(evidence+'/native-associated').sort()){
 const row=JSON.parse(readFileSync(evidence+'/native-associated/'+file,'utf8'));
 let state=row.state;
 if(file.startsWith('checkpoint-'))checkpoints++;
 else if(file.startsWith('step-')||file.startsWith('reject-')){
  const actual=JSON.parse(k.applyRoom(state,row.seat,JSON.stringify(row.command),'0'));
  assert.deepEqual(actual,row.expected,file);state=actual.state;
  if(file.startsWith('reject-')){assert.equal(actual.outcome,'rejected');assert.equal(state,row.state);rejects++;}else commands++;
 }else throw Error('Unknown native body '+file);
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(state,seat)),row.views[seat],file+' seat'+seat);views++;}
}
const fixtures=JSON.parse(readFileSync(root+'/web/src/game/millPublicTest.fixture.json','utf8')).fixtures;
assert.equal(fixtures.length,46);
for(const row of fixtures)for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(row.state,seat)),row.views[seat]);views++;}
const old31=await load('legacy-v0.2.31'),old32=await load('legacy-v0.2.32');
const {routeKernels}=await import(pathToFileURL(root+'/sites/src/kernel-router.mjs'));
const routed=routeKernels(k,[old31.k,old32.k,previous.k]);
for(const historical of [old31,old32,previous]){
 const room=JSON.parse(historical.k.newGame('preserved-offline','LOCAL','teams','P0','watchers','1'));
 assert.deepEqual(JSON.parse(routed.view(room.state,0)),room.view);
 assert.equal(routed.catalog(room.state),historical.k.catalog());
 assert.throws(()=>k.view(room.state,0));
 const command={commandId:'frozen-ready',expectedVersion:room.version,action:{kind:'game',action:{kind:'ready'}}};
 assert.equal(routed.applyRoom(room.state,0,JSON.stringify(command),'0'),historical.k.applyRoom(room.state,0,JSON.stringify(command),'0'));
}
// One forbidden public conceal plus two forbidden non-controlled costs.
assert(commands>0&&checkpoints>0&&rejects>=3);
console.log(JSON.stringify({ok:true,engine:catalog.engineVersion,pool:catalog.cardPoolVersion,wasmSha256:sha(bytes),wasmBytes:bytes.length,commands,checkpoints,rejects,comparedFullSeatViews:views,frontendWholeFixtures:fixtures.length,prior96DefinitionsExact:true,prior8SocietiesExact:true,prior5PresetsExact:true,prior104ScansExact:true,preservedFrozenFiles:Object.keys(frozen).length,routedHistoricalEngines:[old31,old32,previous].map(x=>x.catalog.engineVersion),scope:'Explicit offline native commands and restored four-seat projections; not deployed/public-room play'},null,2));
