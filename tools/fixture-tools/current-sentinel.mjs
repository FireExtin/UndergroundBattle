// Compare freshly exported current Native witnesses before packaging them.
// Opaque Room bytes remain strings, including their original u64 values.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {kernel} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {writeCurrentFixture,currentSourceManifestSha256,currentWasmSha256,sha256}
  from '../current-native-source.mjs';
const [input,destination,receipt]=process.argv.slice(2);
assert(input&&destination&&receipt,'fresh-native NEW-gzip-destination NEW-receipt required');
const bytes=readFileSync(input),scenarios=JSON.parse(bytes);
assert(Array.isArray(scenarios)&&scenarios.length>0);
let commands=0,views=0,duplicates=0;
for(const s of scenarios){
 for(let seat=0;seat<s.initialViews.length;seat++){
  assert.deepEqual(JSON.parse(kernel.view(s.initial,seat)),s.initialViews[seat]);views++;
 }
 for(const r of s.steps){
  const result=JSON.parse(kernel.applyRoom(r.before,r.seat,JSON.stringify(r.command),r.now));
  assert.deepEqual(result,r.expected,s.name+' Native command');commands++;
  for(let seat=0;seat<r.views.length;seat++){
   assert.deepEqual(JSON.parse(kernel.view(result.state,seat)),r.views[seat]);views++;
  }
  assert.deepEqual(JSON.parse(kernel.applyRoom(result.state,r.seat,JSON.stringify(r.command),r.now)),r.duplicate,s.name+' exact duplicate');duplicates++;
 }
}
writeCurrentFixture(destination,scenarios);
const result={inputSha256:sha256(bytes),currentSourceManifestSha256,currentWasmSha256,
 scenarios:scenarios.length,commands,views,duplicates,scope:'Author fresh Native/current WASM comparisons; not browser or public admission acceptance'};
writeFileSync(receipt,JSON.stringify(result,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(result));
