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
export const scenarioNames=['msjc04-search','msjc05-search',...['msjc04','msjc05'].flatMap(id=>[id+'-initiative-draw',id+'-rear-paid-no-draw'])].sort();
export function assertSources(binding) {
  assertCurrentSourceBindings(binding);
}

export function validateNative(raw) {
  assert.equal(raw.format,'msjc0405-native83-v1');assertSources(raw.sourceBindings);
  assertCurrentCatalog(raw.catalog);
  assert.equal(raw.catalog.cards.length,193);assert.equal(raw.catalog.societies.length,10);
  assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
  let commands=0,rejected=0,projections=0;
  for(const scenario of raw.scenarios) {
    assert.equal(scenario.initial.views.length,4);projections+=4;
    let state=scenario.initial.state;assert.equal(typeof state,'string');
    for(const row of scenario.steps) {
      assert.equal(row.state,state,'Discontinuous opaque Room state: '+scenario.name);assert.equal(row.serverNowMs,'0');
      assert.equal(row.views.length,4);assert.deepEqual(row.expected.view,row.views[row.seat]);projections+=4;
      assert.equal(typeof row.expected.state,'string');
      if(row.expected.errorCode) {
        assert.equal(row.expected.outcome,'rejected');assert.equal(row.expected.state,state);
        assert.equal(row.expected.changed,false);assert.deepEqual(row.expected.journal,[]);rejected++;
      } else assert.equal(row.expected.outcome,'accepted');
      state=row.expected.state;commands++;
    }
  }
  return {scenarios:raw.scenarios.length,commands,rejected,projections};
}
export function loadFixture() {
  const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/msjc0405Native93.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.format,'msjc0405-native93-raw-v1');
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
    scenario.initial.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(scenario.initial.state,seat)),view,scenario.name+' initial'));
    for(const row of scenario.steps) {
      assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,scenario.name+' command');
      row.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),view,scenario.name+' view'));
    }
  }
  return {...fixture.counts,nativeInputs:Object.keys(fixture.sourceBindings.files).length,wasmSha256:fixture.wasmSha256,nativeRawSha256:fixture.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: msjc0405-oracle.mjs fresh-native.json current93.wasm NEW-Native93.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  assert.equal(sha256(bytes),currentWasmSha256);
  const bundle={format:'msjc0405-native93-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes),currentSourceManifestSha256};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
