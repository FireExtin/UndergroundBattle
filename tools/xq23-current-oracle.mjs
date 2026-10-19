// Fresh current89 Native states; the exact published88 witnesses stay untouched.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {assertSources,identity,sha256} from './jc035-oracle.mjs';
import {scenarioNames,compare} from './xq23-oracle.mjs';
export {compare};
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export function validateNative(raw){
 assert.equal(raw.format,'xq23-current-native89-v1');assertSources(raw.sourceBindings);
 for(const[k,v]of Object.entries(identity))assert.equal(raw.catalog[k],v);
 assert.equal(raw.catalog.cards.length,153);assert.equal(raw.catalog.societies.length,10);
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
export function loadCurrentFixture(){
 const b=JSON.parse(gunzipSync(readFileSync(path.join(root,'rust-game-wasm/tests/xq23CoreNative89.fixture.json.gz'))));
 assert.equal(b.format,'xq23-current-native89-raw-v1');assert.equal(Buffer.byteLength(b.raw.json),b.raw.bytes);assert.equal(sha256(b.raw.json),b.raw.sha256);
 const raw=JSON.parse(b.raw.json);assert.deepEqual(validateNative(raw),b.counts);return{...b,...raw};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(moduleUrl)){
 const[nativeFile,wasmFile,destination]=process.argv.slice(2);if(!destination)throw Error('Usage: xq23-current-oracle.mjs NEW.NativeCurrent89.json current89.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
 const b={format:'xq23-current-native89-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const{kernel,moduleBytes}=await import('../rust-game-wasm/tests/current89-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=compare({...b,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(b),{level:9}),{flag:'wx'});console.log(JSON.stringify({destination,...result,nativeInputs:141}));
}
