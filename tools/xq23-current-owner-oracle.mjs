// Fresh ABI93 execution of the reviewed owner-elimination behavior. No old state is re-signed.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {assertCurrentSourceBindings as assertSources,assertCurrentCatalog,currentWasmSha256,currentSourceManifestSha256} from './current-native-source.mjs';
import {sha256} from './xq23-oracle.mjs';
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export function validateOwnerNative(raw){
 assert.equal(raw.format,'xq23-current-owner-elimination-native-v1');assertSources(raw.sourceBindings);
 assertCurrentCatalog(raw.catalog);assert.equal(raw.initial.views.length,4);
 let state=raw.initial.state,rejected=0;
 for(const row of raw.steps){
  assert.equal(row.state,state);assert.equal(row.serverNowMs,'0');assert.equal(row.views.length,4);assert.deepEqual(row.expected.view,row.views[row.seat]);
  if(row.expected.errorCode){rejected++;assert.equal(row.expected.changed,false);assert.equal(row.expected.state,state);assert.deepEqual(row.expected.journal,[]);}
  else{assert.equal(row.expected.changed,true);assert.equal(row.expected.outcome,'accepted');}
  state=row.expected.state;
 }
 const before=JSON.parse(raw.beforeElimination).game,after=JSON.parse(raw.afterElimination).game,final=JSON.parse(state).game;
 assert.equal(before.players[0].assets.length,11);assert.equal(after.players[0].assets.length,10);
 assert.equal(after.players[2].eliminated,true);assert.equal(after.players[3].eliminated,false);assert.equal(after.status,'playing');
 assert(after.players.flatMap(p=>p.assets).every(c=>c.owner!==2));assert(after.players[0].assets.some(c=>c.definition==='XQ23'&&c.owner===3&&c.controller===0));
 assert.equal(after.players[0].assets.filter(c=>!c.exhausted).length,10);
 assert.equal(final.players[2].hand.length,0);assert(final.players[3].hand.some(c=>c.definition==='XQ23'));
 assert.equal(raw.steps.at(-1).views[0].players[2].score,4);assert.equal(raw.steps.at(-1).views[0].players[3].score,2);
 assert.equal(raw.summary.commands,raw.steps.length);assert.equal(raw.steps.length,166);assert.equal(rejected,1);
 return{commands:raw.steps.length,rejected,projections:4*(raw.steps.length+1),nativeInputs:Object.keys(raw.sourceBindings.files).length};
}
export function loadOwnerFixture(){
 const b=JSON.parse(gunzipSync(readFileSync(path.join(root,'rust-game-wasm/tests/xq23OwnerNative111.fixture.json.gz'))));
 assert.equal(b.format,'xq23-owner-native93-raw-v1');assert.equal(Buffer.byteLength(b.raw.json),b.raw.bytes);assert.equal(sha256(b.raw.json),b.raw.sha256);
 assert.equal(b.currentSourceManifestSha256,currentSourceManifestSha256);assert.equal(b.wasmSha256,currentWasmSha256);
 const raw=JSON.parse(b.raw.json);assert.deepEqual(validateOwnerNative(raw),b.counts);return{...b,...raw};
}
export function compareOwner(f,abi,bytes){
 assert.equal(sha256(bytes),f.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),f.catalog);
 f.initial.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(f.initial.state,seat)),v));
 for(const[index,row]of f.steps.entries()){
  assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,'fresh owner command '+index);
  row.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),v));
 }
 return{...f.counts,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,completeOpaqueStatesJournalsAndFourSeatViewsEqual:true};
}
export async function packOwnerFixture(nativeFile,wasmFile,destination,verify=compareOwner){
 if(!destination)throw Error('Usage: xq23-current-owner-oracle.mjs NEW.Native.json current93.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateOwnerNative(raw);
 const b={format:'xq23-owner-native93-raw-v1',currentSourceManifestSha256,raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const{kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=verify({...b,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(b),{level:9}),{flag:'wx'});console.log(JSON.stringify({destination,...result}));
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(moduleUrl))await packOwnerFixture(...process.argv.slice(2));
