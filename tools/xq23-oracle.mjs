// Current ABI93 compiler-bound Room commands; Native raw schemas and opaque strings are unchanged.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {currentIdentity,assertCurrentSourceBindings,assertCurrentCatalog,currentWasmSha256,currentSourceManifestSha256, currentFixturePath} from './current-native-source.mjs';
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity=currentIdentity;
export const scenarioNames=['actor-0','actor-1','actor-2','actor-3','ordinary-deploy','target-return-response','source-return-response','paid-target','exhausted-target'].sort();
export const assertSources=assertCurrentSourceBindings;
export function validateNative(raw){
 assert.equal(raw.format,'xq23-current-native89-v1');assertSources(raw.sourceBindings);
 assertCurrentCatalog(raw.catalog);assert.equal(raw.catalog.societies.length,23);
 assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
 let commands=0,rejected=0,projections=0;
 for(const s of raw.scenarios){
  let state=s.initial.state;assert.equal(s.initial.views.length,4);projections+=4;
  for(const r of s.steps){
   assert.equal(r.state,state,s.name+' continuous opaque Room state');assert.equal(r.serverNowMs,'0');
   assert.equal(r.views.length,4);assert.deepEqual(r.expected.view,r.views[r.seat]);projections+=4;
   if(r.expected.errorCode){assert.equal(r.expected.outcome,'rejected');assert.equal(r.expected.changed,false);assert.equal(r.expected.state,state);assert.deepEqual(r.expected.journal,[]);rejected++;}
   else assert.equal(r.expected.outcome,'accepted');
   state=r.expected.state;commands++;
  }
 }
 assert.deepEqual(raw.invalid.map(r=>r.name).sort(),[...Array.from({length:23},(_,i)=>'shape-'+i),'bypass-stack','duplicate-frame','asset-owner-before-declaration','asset-owner-paid-frame','target-summary-other-asset-uid','target-summary-empty-uid','target-summary-hidden-kind','target-summary-character-kind'].sort());
 for(const r of raw.invalid){assert.equal(typeof r.state,'string');assert(r.gameError&&r.roomError);assert.match(r.roomId,/^[0-9a-f]{24}$/);assert(Number.isSafeInteger(r.revision));}
 const counts={scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length};
 assert.deepEqual(counts,raw.counts);assert.deepEqual(counts,{scenarios:9,commands:72,rejected:5,projections:324,invalidRestores:31});return counts;
}
export function loadFixture(){
 const bundle=JSON.parse(gunzipSync(readFileSync(currentFixturePath('rust-game-wasm/tests/xq23CoreNative112.fixture.json.gz'))));
 assert.equal(bundle.format,'xq23-current-native93-raw-v1');assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
 assert.equal(bundle.currentSourceManifestSha256,currentSourceManifestSha256);assert.equal(bundle.wasmSha256,currentWasmSha256);
 const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);return {...bundle,...raw};
}
export function compare(f,abi,wasm){
 assert.equal(sha256(wasm),f.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),f.catalog);
 for(const s of f.scenarios){
  s.initial.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(s.initial.state,seat)),v,s.name+' initial view'));
  for(const r of s.steps){
   assert.deepEqual(JSON.parse(abi.applyRoom(r.state,r.seat,JSON.stringify(r.command),r.serverNowMs)),r.expected,s.name+' command');
   r.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(r.expected.state,seat)),v,s.name+' resulting view'));
  }
 }
 for(const r of f.invalid){
  const error=e=>{assert.equal(String(e),r.roomError,r.name+' Native/WASM error');return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(r.state,seat),error);
  assert.throws(()=>abi.applyRoom(r.state,0,JSON.stringify({commandId:'invalid-'+r.name,expectedVersion:r.revision,action:{kind:'pauseRoom'}}),'0'),error);
  assert.throws(()=>abi.pollRoom(r.state,0,'0'),error);
 }
 return {...f.counts,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,completeOpaqueStatesJournalsAndFourViews:true};
}
export async function packCoreFixture(nativeFile,wasmFile,destination){
 if(!destination)throw Error('Usage: xq23-oracle.mjs NativeCurrent89.json current93.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
 const bundle={format:'xq23-current-native93-raw-v1',currentSourceManifestSha256,raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=compare({...bundle,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
 console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(moduleUrl))await packCoreFixture(...process.argv.slice(2));
