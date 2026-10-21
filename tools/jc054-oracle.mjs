// Fresh ABI93 Native observations; original raw bytes and opaque states stay unchanged.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { gzipSync, gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import {currentIdentity,currentWasmSha256,currentSourceManifestSha256,assertCurrentCatalog,assertCurrentSourceBindings} from './current-native-source.mjs';
// Keep the Node directory URL out of Vite's static web-asset rewriting.
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity=currentIdentity;
export const scenarioNames=['paid-deploy','natural-end','mandatory-end','mandatory-order','controlled-conceal','source-replaced','paid-response-reveal','pause-mandatory-order'].sort();
export const invalidNames=[...Array.from({length:12},(_,i)=>`frame-${String(i).padStart(2,'0')}`),'frame-duplicate','optional-declare','grant-hidden','grant-play-source','instance-owner','instance-controller','instance-empty','instance-duplicate','instance-collision','marker-early','marker-window','marker-nested-missing',...Array.from({length:4},(_,i)=>`order-${String(i).padStart(2,'0')}`),
  ...['deploy','reveal'].flatMap(kind=>['exhausted','damage','wounds','shield','time_markers','lock_markers','wrong-source'].map(field=>`paid-stack-${kind}-${field}`)),
  'successor-face-up','successor-off-board'].sort();
export function assertSources(binding) {
  assertCurrentSourceBindings(binding);
}
export function validateNative(raw) {
  assert.equal(raw.format,'jc054-native81-v1');assertSources(raw.sourceBindings);
  assertCurrentCatalog(raw.catalog);
  assert.equal(raw.catalog.cards.length,193);assert.equal(raw.catalog.societies.length,10);
  assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
  assert.deepEqual(raw.invalid.map(s=>s.name).sort(),invalidNames);
  let commands=0,projections=0,rejected=0;
  for(const scenario of raw.scenarios) {
    assert.equal(scenario.initial.views.length,4);assert.ok(scenario.steps.length);projections+=4;
    let state=scenario.initial.state;assert.equal(typeof state,'string');
    for(const row of scenario.steps) {
      assert.equal(row.state,state,'Discontinuous opaque Room state');assert.equal(row.serverNowMs,'0');
      assert.equal(row.views.length,4);assert.deepEqual(row.expected.view,row.views[row.seat]);
      assert.equal(typeof row.expected.state,'string');
      if(row.expected.errorCode){assert.equal(row.expected.outcome,'rejected');assert.equal(row.expected.state,state);rejected++;}
      else assert.equal(row.expected.outcome,'accepted');
      state=row.expected.state;commands++;projections+=4;
    }
  }
  for(const row of raw.invalid){assert.equal(typeof row.state,'string');assert.equal(typeof row.gameState,'string');assert.ok(row.gameError&&row.roomError);}
  return {scenarios:raw.scenarios.length,commands,projections,rejected,invalid:raw.invalid.length};
}
export function loadFixture() {
  const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/jc054Native93.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.format,'jc054-native93-raw-v1');
  assert.equal(bundle.currentSourceManifestSha256,currentSourceManifestSha256);
  assert.equal(bundle.wasmSha256,currentWasmSha256);
  assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
  const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);
  return {...bundle,...raw};
}
export function compare(fixture,abi,bytes) {
  assert.equal(sha256(bytes),currentWasmSha256);
  assert.equal(fixture.currentSourceManifestSha256,currentSourceManifestSha256);
  assert.equal(sha256(bytes),fixture.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),fixture.catalog);
  for(const scenario of fixture.scenarios) {
    scenario.initial.views.forEach((v,s)=>assert.deepEqual(JSON.parse(abi.view(scenario.initial.state,s)),v,scenario.name));
    for(const row of scenario.steps) {
      assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),'0')),row.expected,scenario.name);
      row.views.forEach((v,s)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,s)),v,scenario.name));
    }
  }
  for(const row of fixture.invalid)for(let seat=0;seat<4;seat++) {
    const rejected=fn=>{let error;try{fn();}catch(e){error=String(e);}assert.equal(error,row.roomError,row.name);};
    rejected(()=>abi.view(row.state,seat));
    rejected(()=>abi.applyRoom(row.state,seat,JSON.stringify({commandId:'invalid',expectedVersion:0,action:{kind:'pauseRoom'}}),'0'));
    rejected(()=>abi.quoteRoom(row.state,seat,'{}'));
  }
  return {...fixture.counts,nativeInputs:Object.keys(fixture.sourceBindings.files).length,negativeRestoreCalls:fixture.invalid.length*12,wasmSha256:fixture.wasmSha256,completeOpaqueStateAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: jc054-oracle.mjs fresh-native.json current93.wasm NEW-Native93.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  assert.equal(sha256(bytes),currentWasmSha256);
  const bundle={format:'jc054-native93-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes),currentSourceManifestSha256};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
