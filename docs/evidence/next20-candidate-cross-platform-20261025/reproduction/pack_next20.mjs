import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
const source=process.cwd();
const {kernel,catalog}=await import(pathToFileURL(path.join(source,'rust-game-wasm/tests/current-kernel.mjs')));
const {writeCurrentFixture,sha256,currentSourceManifestSha256,currentWasmSha256}=await import(pathToFileURL(path.join(source,'tools/current-native-source.mjs')));
const bytes=readFileSync(process.argv[2]),native=JSON.parse(bytes);assert.equal(native.format,'next20-candidate-room-native-v1');assert.equal(native.publicAdmission,false);assert.deepEqual(native.catalog,catalog);assert.equal(native.cases.length,4);
let commands=0,views=0,duplicates=0;
for(const s of native.cases){
 let state=s.initial.state;
 s.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),v);views++;});
 for(const r of s.steps){
  assert.equal(r.state,state);const actual=JSON.parse(kernel.applyRoom(state,r.seat,JSON.stringify(r.command),r.serverNowMs));assert.deepEqual(actual,r.expected);state=actual.state;commands++;
  r.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),v);views++;});
  assert.deepEqual(JSON.parse(kernel.applyRoom(state,r.seat,JSON.stringify(r.command),r.serverNowMs)),r.duplicate);duplicates++;
 }
}
for(const id of ['BQ033','BQ034','CQ25','XG25','BQ009'])assert.equal(catalog.cards.find(c=>c.id===id).supported,false);
writeCurrentFixture('rust-game-wasm/tests/current112/next20-candidate-room.json.gz',native);
console.log(JSON.stringify({cases:native.cases.length,commands,views,duplicateReducerProbes:duplicates,rawNativeSha256:sha256(bytes),currentSourceManifestSha256,currentWasmSha256,publicAdmission:false,preparedAnchorNotSupplyReachability:true}));
