// Actual Native89 state strings and journals; old88 witnesses remain independent.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity={rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.81-jc035-preparation-candidate',engineVersion:'rust-v0.2.89-jc035-preparation-candidate'};
export const scenarioNames=['actor-0','actor-1','actor-2','actor-3','optional-decline','threshold-four','source-destroyed-response','hand-growth-response','late-shrink-no-retroactive-trigger','same-controller-order','won-original-region','ordinary-attach','duel-preparation'].sort();
export const legalRestoreNames=['live-pending','live-queued-undecided','live-accepted','declined-without-container','resolved-without-container',
 'eliminated-actor-undecided','eliminated-actor-accepted','finished-undecided','finished-accepted','departed-source-accepted','changed-controller-accepted'].sort();
export function assertSources(binding){
 const old=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi88/native-source-binding.json')));
 const names=Object.keys(old.files).concat(['rust-game/src/jc035.rs','rust-game/src/jc035_tests.rs']).sort();
 assert.equal(binding.algorithm,'sha256');assert.equal(names.length,141);assert.deepEqual(Object.keys(binding.files).sort(),names);
 for(const[name,digest]of Object.entries(binding.files))assert.equal(sha256(readFileSync(path.join(root,name))),digest,'Native141 input '+name);
}
export function validateNative(raw){
 assert.equal(raw.format,'jc035-native89-v1');assertSources(raw.sourceBindings);
 for(const[k,v]of Object.entries(identity))assert.equal(raw.catalog[k],v);
 assert.equal(raw.catalog.cards.length,153);assert.equal(raw.catalog.societies.length,10);
 assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
 let commands=0,rejected=0,projections=0;
 for(const s of raw.scenarios){
  let state=s.initial.state,n=s.name==='duel-preparation'?2:4;
  assert.equal(s.initial.views.length,n);projections+=n;
  for(const r of s.steps){
   assert.equal(r.state,state,s.name+' continuous opaque state');assert.equal(r.serverNowMs,'0');
   assert.equal(r.views.length,n);assert.deepEqual(r.expected.view,r.views[r.seat]);projections+=n;
   if(r.expected.errorCode){assert.equal(r.expected.changed,false);assert.equal(r.expected.state,state);assert.deepEqual(r.expected.journal,[]);rejected++;}
   else{assert.equal(r.expected.outcome,'accepted');assert.equal(r.expected.changed,true);}
   state=r.expected.state;commands++;
  }
 }
 const invalidNames=[...Array.from({length:24},(_,i)=>'prepare-shape-'+i),...Array.from({length:13},(_,i)=>'frame-shape-'+i),...Array.from({length:7},(_,i)=>'entity-shape-'+i),...Array.from({length:2},(_,i)=>'attach-shape-'+i),'resolved-event-replay',
  'orphan-undecided-pending','orphan-undecided-partial-choice','orphan-accepted-frame'].sort();
 assert.deepEqual(raw.invalid.map(c=>c.name).sort(),invalidNames);
 for(const c of raw.invalid){assert(c.gameError&&c.roomError);assert.equal(typeof c.state,'string');assert.match(c.roomId,/^[0-9a-f]{24}$/);}
 assert.deepEqual(raw.legalRestores.map(c=>c.name).sort(),legalRestoreNames);
 let restoreContinuationCommands=0;
 for(const c of raw.legalRestores){
  assert.equal(typeof c.gameState,'string');assert.equal(typeof c.state,'string');assert.equal(c.views.length,4);
  if(c.continuation){const r=c.continuation;assert.equal(c.name,'live-pending');assert.equal(r.serverNowMs,'0');
   assert.equal(r.expected.outcome,'accepted');assert.equal(r.expected.changed,true);assert.equal(r.expected.errorCode??null,null);
   assert.equal(r.views.length,4);assert.deepEqual(r.expected.view,r.views[r.seat]);restoreContinuationCommands++;}
 }
 const counts={scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length,
  legalRestores:raw.legalRestores.length,restoreContinuationCommands};assert.deepEqual(counts,raw.counts);
 return counts;
}
export function loadFixture(){
 const bundle=JSON.parse(gunzipSync(readFileSync(path.join(root,'rust-game-wasm/tests/jc035Native89.fixture.json.gz'))));
 assert.equal(bundle.format,'jc035-native89-raw-v1');assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
 const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);return {...bundle,...raw};
}
export function compare(f,abi,bytes){
 assert.equal(sha256(bytes),f.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),f.catalog);
 for(const s of f.scenarios){
  s.initial.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(s.initial.state,seat)),v,s.name+' initial view'));
  for(const row of s.steps){
   assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,s.name+' actual command');
   row.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),v,s.name+' resulting view'));
  }
 }
 for(const c of f.invalid){
  const error=e=>{assert.equal(String(e),c.roomError,c.name+' exact Native/WASM error');return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(c.state,seat),error);
  assert.throws(()=>abi.applyRoom(c.state,0,JSON.stringify({commandId:'invalid-'+c.name,expectedVersion:c.revision,action:{kind:'pauseRoom'}}),'0'),error);
  assert.throws(()=>abi.pollRoom(c.state,0,'0'),error);
 }
 for(const c of f.legalRestores){
  c.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(c.state,seat)),view,c.name+' legal restore'));
  const poll=JSON.parse(abi.pollRoom(c.state,0,'0'));assert.equal(poll.changed,false,c.name+' loading must not pump');assert.equal(poll.state,c.state);
  if(c.continuation){const r=c.continuation;
   assert.deepEqual(JSON.parse(abi.applyRoom(c.state,r.seat,JSON.stringify(r.command),r.serverNowMs)),r.expected,c.name+' real pending continuation');
   r.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(r.expected.state,seat)),view,c.name+' continued view'));}
 }
 return {...f.counts,nativeInputs:141,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,allOpaqueStatesJournalsAndActualSeatViewsEqual:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(moduleUrl)){
 const[nativeFile,wasmFile,destination]=process.argv.slice(2);if(!destination)throw Error('Usage: jc035-oracle.mjs Native89.json current89.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
 const bundle={format:'jc035-native89-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const{kernel,moduleBytes}=await import('../rust-game-wasm/tests/current89-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=compare({...bundle,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9}),{flag:'wx'});console.log(JSON.stringify({destination,...result}));
}
