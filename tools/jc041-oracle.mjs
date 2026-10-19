// Compiler-bound Native87; JS never parses or rewrites opaque Room/Game strings.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity={rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.79-jc041-water-hidden-candidate',engineVersion:'rust-v0.2.87-jc041-water-hidden-candidate'};
export const scenarioNames=['water-actor-0','water-actor-1','water-actor-2','water-actor-3','ordinary-deploy','ordinary-reveal-land','empty-hands','hidden-return-response','revealed-source-return-response','repeated-hidden-actions'].sort();
export function assertSources(binding){
 const saved=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi87/native-source-binding.json'),'utf8'));
 assert.equal(binding.algorithm,'sha256');assert.equal(Object.keys(saved.files).length,137);
 assert.deepEqual(binding,saved,'Historical Native87 compiler binding must remain unchanged');
}
export function validateNative(raw){
 assert.equal(raw.format,'jc041-native87-v1');assertSources(raw.sourceBindings);
 for(const[k,v]of Object.entries(identity))assert.equal(raw.catalog[k],v);
 assert.equal(raw.catalog.cards.length,151);assert.equal(raw.catalog.societies.length,10);
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
export function loadFixture(){
 const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/jc041Native87.fixture.json.gz')),pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi87/identity.json'),'utf8'));
 assert.equal(sha256(bytes),pinned.nativeFixtureSha256,'Published Native87 fixture bytes must not change');
 const bundle=JSON.parse(gunzipSync(bytes));assert.equal(bundle.raw.sha256,pinned.nativeRawSha256);assert.equal(bundle.raw.bytes,pinned.nativeRawBytes);
 assert.equal(bundle.format,'jc041-native87-raw-v1');assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
 const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);return {...bundle,...raw};
}
export function compare(f,abi,wasm){
 const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi87/identity.json'),'utf8'));assert.equal(sha256(wasm),pinned.wasmSha256,'Only original published ABI87 can check Native87');
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
 return {...f.counts,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const[nativeFile,wasmFile,destination]=process.argv.slice(2);if(!destination)throw Error('Usage: jc041-oracle.mjs Native87.json current87.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
 const bundle={format:'jc041-native87-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current87-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=compare({...bundle,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
 console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
