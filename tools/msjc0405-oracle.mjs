// Compiler-bound Native83 observations. Creation refuses to overwrite a witness.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity={rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.75-msjc04-msjc05-societies-candidate',engineVersion:'rust-v0.2.83-msjc04-msjc05-societies-candidate'};
export const scenarioNames=['msjc04-search','msjc05-search',...['msjc04','msjc05'].flatMap(id=>[id+'-initiative-draw',id+'-rear-paid-no-draw'])].sort();
export function assertSources(binding) {
  const saved=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi83/native-source-binding.json'),'utf8'));
  assert.equal(binding.algorithm,'sha256');assert.equal(Object.keys(saved.files).length,129,'Exact published Native83 input count');
  assert.deepEqual(binding,saved,'Historical Native83 compiler binding must remain unchanged');
}

export function validateNative(raw) {
  assert.equal(raw.format,'msjc0405-native83-v1');assertSources(raw.sourceBindings);
  for(const [key,value]of Object.entries(identity))assert.equal(raw.catalog[key],value);
  assert.equal(raw.catalog.cards.length,147);assert.equal(raw.catalog.societies.length,10);
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
  const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/msjc0405Native83.fixture.json.gz'));
  const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi83/identity.json'),'utf8'));
  assert.equal(sha256(bytes),pinned.nativeFixtureSha256,'Published Native83 fixture bytes must not change');
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.raw.sha256,pinned.nativeRawSha256);assert.equal(bundle.raw.bytes,pinned.nativeRawBytes);assert.equal(bundle.format,'msjc0405-native83-raw-v1');
  assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
  const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);
  return {...bundle,...raw};
}
export function compare(fixture,abi,bytes) {
  const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi83/identity.json'),'utf8'));
  assert.equal(sha256(bytes),pinned.wasmSha256,'Only the exact published v65 WASM may check Native83');
  assert.equal(sha256(bytes),fixture.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),fixture.catalog);
  for(const scenario of fixture.scenarios) {
    scenario.initial.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(scenario.initial.state,seat)),view,scenario.name+' initial'));
    for(const row of scenario.steps) {
      assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,scenario.name+' command');
      row.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),view,scenario.name+' view'));
    }
  }
  return {...fixture.counts,wasmSha256:fixture.wasmSha256,nativeRawSha256:fixture.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: msjc0405-oracle.mjs Native83.json fresh83.wasm NEW.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi83/identity.json'),'utf8'));
  assert.equal(sha256(json),pinned.nativeRawSha256,'Only original published Native83 bytes may be restored');
  assert.equal(sha256(bytes),pinned.wasmSha256,'A newer build cannot re-sign Native83');
  const bundle={format:'msjc0405-native83-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current83-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
