// Fresh ABI93 Native observations; original raw bytes and opaque states stay unchanged.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {currentIdentity,currentWasmSha256,currentSourceManifestSha256,assertCurrentCatalog,assertCurrentSourceBindings} from './current-native-source.mjs';
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity=currentIdentity;
export const scenarioNames=["actor-0", "actor-1", "actor-2", "actor-3", "entry-deploy-accept", "entry-deploy-skip", "entry-reveal-accept", "entry-reveal-skip", "host-hide", "host-return", "self-seal", "shared-hand-seal-attachment", "shared-hand-seal-character", "society-search-gray-unique", "target-control", "target-departure", "target-hide", "target-move", "target-reveal"];
export function assertSources(binding) {
  assertCurrentSourceBindings(binding);
}
export function validateNative(raw){
 assert.equal(raw.format,'jc066-native86-v1');assertSources(raw.sourceBindings);
 assertCurrentCatalog(raw.catalog);
 assert.equal(raw.catalog.cards.length,272);assert.equal(raw.catalog.societies.length,12);
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
 assert.deepEqual(raw.invalid.map(r=>r.name).sort(),[['stack-deploy',16],['stack-reveal',16],['declare',25],['frame',28]].flatMap(([k,length])=>Array.from({length},(_,i)=>`${k}-${String(i).padStart(2,'0')}`)).concat(['target-face-up-as-hidden','target-hidden-as-face-up']).sort());
 for(const r of raw.invalid){assert.equal(typeof r.state,'string');assert(r.gameError.length>0&&r.roomError.length>0);assert.equal(typeof r.roomId,'string');assert(Number.isSafeInteger(r.revision));}
 const counts={scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length};
 assert.deepEqual(counts,{"scenarios":19,"commands":252,"rejected":16,"projections":1084,"invalidRestores":87},'Complete JC066 Native acceptance contract');
 return counts;
}
export function loadFixture() {
  const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/jc066Native111.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.format,'jc066-native93-raw-v1');
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
  if(!destination)throw Error('Usage: jc066-oracle.mjs fresh-native.json current93.wasm NEW-Native93.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  assert.equal(sha256(bytes),currentWasmSha256);
  const bundle={format:'jc066-native93-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes),currentSourceManifestSha256};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
