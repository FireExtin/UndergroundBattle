// Native compiler observations; opaque states and u64 counters stay in Rust.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const root=fileURLToPath(new URL('../',import.meta.url));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity={rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.77-jc044-time-repress-candidate',engineVersion:'rust-v0.2.85-jc044-time-repress-candidate'};
export const scenarioNames=[...['deploy','reveal'].flatMap(k=>['accept','skip'].map(a=>`entry-${k}-${a}`)),...[0,1,2,3].map(a=>'actor-'+a),'target-self','target-partner','target-empty','physical-attachment-cost','responsive-exhaust'].sort();
export function assertSources(binding){
 const expected=Object.keys(JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi84/native-source-binding.json'),'utf8')).files).concat(['rust-game/src/jc044.rs','rust-game/src/jc044_tests.rs']).sort();
 assert.equal(binding.algorithm,'sha256');assert.equal(expected.length,133);assert.deepEqual(Object.keys(binding.files).sort(),expected);
 for(const[name,hash]of Object.entries(binding.files))assert.equal(sha256(readFileSync(path.join(root,name))),hash,'Compiler input '+name);
}
export function validateNative(raw){
 assert.equal(raw.format,'jc044-native85-v1');assertSources(raw.sourceBindings);
 for(const[key,value]of Object.entries(identity))assert.equal(raw.catalog[key],value);
 assert.equal(raw.catalog.cards.length,149);assert.equal(raw.catalog.societies.length,10);
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
 assert.deepEqual(raw.invalid.map(r=>r.name).sort(),['declare','repression'].flatMap(k=>Array.from({length:16},(_,i)=>`${k}-${String(i).padStart(2,'0')}`)).sort());
 for(const r of raw.invalid){assert.equal(typeof r.state,'string');assert(r.gameError.length>0&&r.roomError.length>0);assert.equal(typeof r.roomId,'string');assert(Number.isSafeInteger(r.revision));}
 return {scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length};
}
export function loadFixture(){
 const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/jc044Native85.fixture.json.gz')),bundle=JSON.parse(gunzipSync(bytes));
 assert.equal(bundle.format,'jc044-native85-raw-v1');assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
 const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);return {...bundle,...raw};
}
export function compare(f,abi,wasm){
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
 return {...f.counts,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const[nativeFile,wasmFile,destination]=process.argv.slice(2);if(!destination)throw Error('Usage: jc044-oracle.mjs Native85.json fresh85.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
 const bundle={format:'jc044-native85-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current85-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=compare({...bundle,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
 console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
