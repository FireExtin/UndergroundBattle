// Fresh ABI93 Native observations; original raw bytes and opaque states stay unchanged.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {currentIdentity,currentWasmSha256,currentSourceManifestSha256,assertCurrentCatalog,assertCurrentSourceBindings, currentFixturePath} from './current-native-source.mjs';
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity=currentIdentity;
export const scenarioNames=['water-actor-0','water-actor-1','water-actor-2','water-actor-3','ordinary-deploy','ordinary-reveal-land','empty-hands','hidden-return-response','revealed-source-return-response','repeated-hidden-actions'].sort();
export function assertSources(binding) {
  assertCurrentSourceBindings(binding);
}
export function validateNative(raw){
 assert.equal(raw.format,'jc041-native87-v1');assertSources(raw.sourceBindings);
 assertCurrentCatalog(raw.catalog);
 assert.equal(raw.catalog.cards.length,277);assert.equal(raw.catalog.cards.filter(card=>card.supported).length,236);for(const id of ['BQ033','BQ009'])assert.equal(raw.catalog.cards.find(card=>card.id===id).supported,false);assert.equal(raw.catalog.societies.length,12);
 assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
 let commands=0,rejected=0,projections=0;
 for(const s of raw.scenarios){
  let state=s.initial.state;assert.equal(s.initial.views.length,4);projections+=4;
  for(const row of s.steps){
   assert.equal(row.state,state,s.name+' continuous opaque state');assert.equal(row.serverNowMs,'0');assert.equal(row.views.length,4);
   assert.deepEqual(row.expected.view,row.views[row.seat]);projections+=4;
   if(row.expected.errorCode){assert.equal(row.expected.outcome,'rejected');assert.equal(row.expected.changed,false);assert.equal(row.expected.state,state);assert.deepEqual(row.expected.journal,[]);rejected++;}
   else assert.equal(row.expected.outcome,'accepted');
   state=row.expected.state;commands++;
  }
 }
 const names=[...Array.from({length:16},(_,i)=>'paid-'+i),...Array.from({length:14},(_,i)=>'discard-'+i),'direct-effect','duplicate-frame','missing-hidden-frame','missing-forced-frame','missing-discard-frame'].sort();
 assert.deepEqual(raw.invalid.map(r=>r.name).sort(),names);
 for(const r of raw.invalid){assert.equal(typeof r.state,'string');assert(r.gameError&&r.roomError);assert.match(r.roomId,/^[0-9a-f]{24}$/);assert(Number.isSafeInteger(r.revision));}
 const counts={scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length};assert.deepEqual(counts,raw.counts);assert.deepEqual(counts,{scenarios:10,commands:155,rejected:9,projections:660,invalidRestores:35});return counts;
}
export function loadFixture() {
  const bytes=readFileSync(currentFixturePath('rust-game-wasm/tests/jc041Native112.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.format,'jc041-native93-raw-v1');
  assert.equal(bundle.currentSourceManifestSha256,currentSourceManifestSha256);
  assert.equal(bundle.wasmSha256,currentWasmSha256);
  assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
  const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);
  return {...bundle,...raw};
}
export function compare(f,abi,wasm){
  assert.equal(sha256(wasm),currentWasmSha256);
  assert.equal(f.currentSourceManifestSha256,currentSourceManifestSha256);
  assert.equal(sha256(wasm),f.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),f.catalog);
 for(const s of f.scenarios){
  s.initial.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(s.initial.state,seat)),v,s.name+' initial'));
  for(const r of s.steps){assert.deepEqual(JSON.parse(abi.applyRoom(r.state,r.seat,JSON.stringify(r.command),r.serverNowMs)),r.expected,s.name+' command');r.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(r.expected.state,seat)),v,s.name+' view'));}
 }
 for(const r of f.invalid){const error=e=>{assert.equal(String(e),r.roomError,r.name+' Native/WASM error');return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(r.state,seat),error);
  assert.throws(()=>abi.applyRoom(r.state,0,JSON.stringify({commandId:'invalid-'+r.name,expectedVersion:r.revision,action:{kind:'pauseRoom'}}),'0'),error);
  assert.throws(()=>abi.pollRoom(r.state,0,'0'),error);
 }
 return {...f.counts,nativeInputs:Object.keys(f.sourceBindings.files).length,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
// The current P2 probe is a separate fresh Room trace. The archived ABI87
// input is only checked as an immutable rejected input, never rewritten.
export function validateP2Native(raw) {
 assert.equal(raw.format,'jc041-source-region-current-v1');assertSources(raw.sourceBindings);
 assert.equal(raw.currentIdentity,currentIdentity.engineVersion);assert.equal(raw.expectedAccepted,false);
 assert.equal(raw.archivedStateSha256,'ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c');
 assert.equal(sha256(readFileSync(path.join(root,'rust-game-wasm/tests/jc041P2.original-room.json'))),raw.archivedStateSha256);
 assert.deepEqual(raw.cases.map(c=>c.name).sort(),['source-self','known-hand-nonregion','known-paid-asset-nonregion','known-region-wrong-slot'].sort());
 assert.deepEqual(raw.legal.map(s=>s.name).sort(),['p2-original','p2-source-moved','p2-original-region-retired','p2-vacant-original-slot'].sort());
 assert.equal(typeof raw.originalState,'string');assert.equal(raw.originalViews.length,4);
 for(const c of raw.cases){assert.equal(typeof c.state,'string');assert(c.gameError&&c.roomError);assert.equal(c.views,null);assert.equal(c.expectedPause,null);assert.equal(c.expectedPoll,null);}
 let commands=0,views=raw.originalViews.length;
 for(const scenario of raw.legal){
  let state=scenario.initial.state;assert.equal(scenario.initial.views.length,4);views+=4;
  for(const row of scenario.steps){
   assert.equal(row.state,state);assert.equal(row.serverNowMs,'0');assert.equal(row.views.length,4);
   assert.equal(row.expected.outcome,'accepted');assert.deepEqual(row.expected.view,row.views[row.seat]);
   state=row.expected.state;commands++;views+=4;
  }
 }
 const counts={badChecks:raw.cases.length*6,commands,views};assert.deepEqual(counts,{badChecks:24,commands:56,views:244});return counts;
}
export function loadP2Fixture() {
 const bundle=JSON.parse(gunzipSync(readFileSync(currentFixturePath('rust-game-wasm/tests/jc041P2Native112.fixture.json.gz'))));
 assert.equal(bundle.format,'jc041-p2-native93-raw-v1');assert.equal(bundle.currentSourceManifestSha256,currentSourceManifestSha256);
 assert.equal(bundle.wasmSha256,currentWasmSha256);assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
 const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateP2Native(raw),bundle.counts);return {...bundle,...raw};
}
export function compareP2(f,abi,bytes) {
 assert.equal(sha256(bytes),currentWasmSha256);assert.equal(f.wasmSha256,currentWasmSha256);
 assert.equal(f.currentSourceManifestSha256,currentSourceManifestSha256);assertCurrentCatalog(JSON.parse(abi.catalog()));
 let badChecks=0,commands=0,views=0;
 f.originalViews.forEach((v,seat)=>{assert.deepEqual(JSON.parse(abi.view(f.originalState,seat)),v);views++;});
 for(const c of f.cases){
  const error=e=>{assert.equal(String(e),c.roomError);return true;};
  for(let seat=0;seat<4;seat++){assert.throws(()=>abi.view(c.state,seat),error);badChecks++;}
  assert.throws(()=>abi.applyRoom(c.state,0,JSON.stringify(c.command),'0'),error);badChecks++;
  assert.throws(()=>abi.pollRoom(c.state,0,'0'),error);badChecks++;
 }
 for(const scenario of f.legal){
  let state=scenario.initial.state;
  scenario.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(abi.view(state,seat)),v);views++;});
  for(const row of scenario.steps){
   assert.equal(row.state,state);assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;
   row.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),v);views++;});state=row.expected.state;
  }
 }
 assert.deepEqual({badChecks,commands,views},f.counts);
 return {scope:'Fresh Native93 source-region probe; no old-state version or UID replacement',badChecks,commands,views,nativeInputs:Object.keys(f.sourceBindings.files).length,nativeRawBytes:f.raw.bytes,nativeRawSha256:f.raw.sha256,wasmSha256:f.wasmSha256,allOpaqueStatesJournalsAndViewsEqual:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const args=process.argv.slice(2),p2=args[0]==='--p2';if(p2)args.shift();
  const [nativeFile,wasmFile,destination]=args;
  if(!destination)throw Error('Usage: jc041-oracle.mjs [--p2] fresh-native.json current93.wasm NEW-Native93.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=(p2?validateP2Native:validateNative)(raw);
  assert.equal(sha256(bytes),currentWasmSha256);
  const bundle={format:p2?'jc041-p2-native93-raw-v1':'jc041-native93-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes),currentSourceManifestSha256};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=(p2?compareP2:compare)({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
