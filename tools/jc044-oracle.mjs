// Fresh ABI93 Native observations; original raw bytes and opaque states stay unchanged.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {currentIdentity,currentWasmSha256,currentSourceManifestSha256,assertCurrentCatalog,assertCurrentSourceBindings, currentFixturePath} from './current-native-source.mjs';
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity=currentIdentity;
export const scenarioNames=[...['deploy','reveal'].flatMap(k=>['accept','skip'].map(a=>`entry-${k}-${a}`)),...[0,1,2,3].map(a=>'actor-'+a),'target-self','target-partner','target-empty','physical-attachment-cost','responsive-exhaust'].sort();
export function assertSources(binding) {
  assertCurrentSourceBindings(binding);
}
export function validateNative(raw){
 assert.equal(raw.format,'jc044-native85-v1');assertSources(raw.sourceBindings);
 assertCurrentCatalog(raw.catalog);
 assert.equal(raw.catalog.cards.length,278);assert.equal(raw.catalog.cards.filter(card=>card.supported).length,236);for(const id of ['BQ033','BQ009','BQ001'])assert.equal(raw.catalog.cards.find(card=>card.id===id).supported,false);assert.equal(raw.catalog.societies.length,12);
 assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
 let commands=0,rejected=0,projections=0;
 for(const s of raw.scenarios){
  assert.equal(s.initial.views.length,4);projections+=4;let state=s.initial.state;
  for(const row of s.steps){
   assert.equal(row.state,state,'Continuous opaque Room state '+s.name);assert.equal(row.serverNowMs,'0');
   assert.equal(row.views.length,4);assert.deepEqual(row.expected.view,row.views[row.seat]);projections+=4;
   if(row.expected.errorCode){assert.equal(row.expected.outcome,'rejected');assert.equal(row.expected.changed,false);assert.equal(row.expected.state,state);assert.deepEqual(row.expected.journal,[]);rejected++;}
   else assert.equal(row.expected.outcome,'accepted');
   state=row.expected.state;commands++;
  }
 }
 assert.deepEqual(raw.invalid.map(r=>r.name).sort(),[['declare',17],['repression',18]].flatMap(([k,length])=>Array.from({length},(_,i)=>`${k}-${String(i).padStart(2,'0')}`)).concat(['stack-deploy-source-identity','stack-reveal-source-identity']).sort());
 for(const r of raw.invalid){assert.equal(typeof r.state,'string');assert(r.gameError.length>0&&r.roomError.length>0);assert.equal(typeof r.roomId,'string');assert(Number.isSafeInteger(r.revision));}
 const counts={scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length};
 assert.deepEqual(counts,{scenarios:13,commands:118,rejected:17,projections:524,invalidRestores:37},'Complete JC044 Native acceptance contract');
 return counts;
}
export function loadFixture() {
  const bytes=readFileSync(currentFixturePath('rust-game-wasm/tests/jc044Native112.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.format,'jc044-native93-raw-v1');
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
  for(const r of s.steps){assert.deepEqual(JSON.parse(abi.applyRoom(r.state,r.seat,JSON.stringify(r.command),r.serverNowMs)),r.expected,s.name+' command');
   r.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(r.expected.state,seat)),v,s.name+' view'));}
 }
 for(const r of f.invalid){const expected=error=>{assert.equal(String(error),r.roomError,r.name+' Native/WASM error');return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(r.state,seat),expected,r.name+' restore view');
  const command={commandId:'invalid-'+r.name,expectedVersion:r.revision,action:{kind:'pauseRoom'}};
  assert.throws(()=>abi.applyRoom(r.state,0,JSON.stringify(command),'0'),expected,r.name+' command');
  assert.throws(()=>abi.pollRoom(r.state,0,'0'),expected,r.name+' poll');
 }
 return {...f.counts,nativeInputs:Object.keys(f.sourceBindings.files).length,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: jc044-oracle.mjs fresh-native.json current93.wasm NEW-Native93.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  assert.equal(sha256(bytes),currentWasmSha256);
  const bundle={format:'jc044-native93-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes),currentSourceManifestSha256};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
