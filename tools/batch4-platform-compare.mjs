// Narrow current-core execution against real Native Room commands. Never patch
// versions or round-trip opaque state through a JavaScript object.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {kernel,moduleBytes,catalog} from '../rust-game-wasm/tests/current-kernel.mjs';
import {currentSourceManifest,currentSourceManifestSha256,currentWasmSha256,writeCurrentFixture} from './current-native-source.mjs';
const [rawFile,destination,receiptFile]=process.argv.slice(2);
if(!rawFile||!destination||!receiptFile)throw Error('Expected raw Native witness, NEW repository-relative gzip destination and receipt');
const native=JSON.parse(readFileSync(rawFile,'utf8'));assert.equal(native.format,'batch4-abi95-native-witness-v1');assert.deepEqual(native.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.95-batch20-fourth');assert.equal(catalog.cardPoolVersion,'limited-v2.86-batch20-fourth');
assert.equal(catalog.cards.length,247);assert.equal(catalog.cards.filter(c=>c.supported).length,213);assert.equal(catalog.cards.find(c=>c.id==='JZ56').supported,false);
let commands=0,views=0,polls=0,duplicates=0,restoreRejections=0;const cases=[];
for(const scenario of native.cases){
 const seed=`"seed":${scenario.seedDecimal}`,random=`"random":${scenario.seedDecimal}`;
 assert(BigInt(scenario.seedDecimal)>BigInt(Number.MAX_SAFE_INTEGER));assert(scenario.initial.state.includes(seed)&&scenario.initial.state.includes(random));
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(scenario.initial.state,seat)),scenario.initial.views[seat]);views++;}
 for(const row of scenario.steps){
  assert(row.state.includes(seed));
  const actual=JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs));
  assert.deepEqual(actual,row.expected,`${scenario.name} command ${commands}`);assert.equal(actual.state,row.expected.state);assert(actual.state.includes(seed));commands++;
  const duplicate=JSON.parse(kernel.applyRoom(actual.state,row.seat,JSON.stringify(row.command),row.serverNowMs));
  assert.equal(duplicate.outcome,'rejected');assert.equal(duplicate.changed,false);assert.equal(duplicate.state,actual.state);assert.deepEqual(duplicate.journal,[]);duplicates++;
  for(let seat=0;seat<4;seat++){
   assert.deepEqual(JSON.parse(kernel.view(actual.state,seat)),row.views[seat]);views++;
   const polled=JSON.parse(kernel.pollRoom(actual.state,seat,row.serverNowMs));assert.equal(polled.state,actual.state);assert.equal(polled.changed,false);assert.deepEqual(polled.view,row.views[seat]);polls++;
  }
 }
 for(const bad of scenario.invalid){
  for(let seat=0;seat<4;seat++){
   for(const operation of [()=>kernel.view(bad.state,seat),()=>kernel.pollRoom(bad.state,seat,'0'),()=>kernel.applyRoom(bad.state,seat,JSON.stringify({commandId:'must-not-execute',expectedVersion:0,action:{kind:'pauseRoom'}}),'0')]){
    assert.throws(operation,e=>!(e instanceof WebAssembly.RuntimeError)&&String(e)===bad.nativeError,`${scenario.name}/${bad.label}`);restoreRejections++;
   }
  }
 }
 cases.push({name:scenario.name,commands:scenario.steps.length,invalidLayouts:scenario.invalid.length,observations:scenario.observations});
}
writeCurrentFixture(destination,native);
const result={scope:'Author current ABI95 Native/WASM directed execution; not full50, Worker, browser or public acceptance',identity:currentSourceManifest.identity,sourceManifestSha256:currentSourceManifestSha256,wasmSha256:currentWasmSha256,wasmBytes:moduleBytes.length,cases,commands,views,polls,duplicates,restoreRejections,opaqueU64Preserved:true,catalogueEqual:true,publicAdmission:false};
writeFileSync(receiptFile,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify(result));
