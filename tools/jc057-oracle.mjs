// Compiler-bound Native84 observations. Creation refuses to overwrite a witness.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity={rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.76-jc057-live-inspection-candidate',engineVersion:'rust-v0.2.84-jc057-live-inspection-candidate'};
export const scenarioNames=['reveal-p0','reveal-p1','reveal-p2','reveal-p3','empty','skip','deploy','live-exhaust','live-hide','live-control'].sort();
export function assertSources(binding) {
  const expected=Object.keys(JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi83/native-source-binding.json'),'utf8')).files)
    .concat(['rust-game/src/jc057.rs','rust-game/src/jc057_tests.rs']).sort();
  assert.equal(binding.algorithm,'sha256');assert.equal(expected.length,131);
  assert.deepEqual(Object.keys(binding.files).sort(),expected);
  for(const [file,hash]of Object.entries(binding.files))assert.equal(sha256(readFileSync(path.join(root,file))),hash,'Compiler input '+file);
}


export function validateNative(raw) {
  assert.equal(raw.format,'jc057-native84-v1');assertSources(raw.sourceBindings);
  for(const [key,value]of Object.entries(identity))assert.equal(raw.catalog[key],value);
  assert.equal(raw.catalog.cards.length,148);assert.equal(raw.catalog.societies.length,10);
  assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
  const secretName=raw.catalog.cards.find(c=>c.id==='JC094').name;
  const assertPrivateLogs=(scenario,views)=>{if(scenario.name.startsWith('reveal-p'))for(const view of views)assert(!view.log.some(e=>e.text.includes(secretName)),'Inspection must not publish the actual inspected JC094 name');};
  let commands=0,rejected=0,projections=0;
  for(const scenario of raw.scenarios) {
    assert.equal(scenario.initial.views.length,4);projections+=4;assertPrivateLogs(scenario,scenario.initial.views);
    let state=scenario.initial.state;assert.equal(typeof state,'string');
    for(const row of scenario.steps) {
      assert.equal(row.state,state,'Discontinuous opaque Room state: '+scenario.name);assert.equal(row.serverNowMs,'0');
      assert.equal(row.views.length,4);assertPrivateLogs(scenario,row.views);assert.deepEqual(row.expected.view,row.views[row.seat]);projections+=4;
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
  const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/jc057Native84.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));assert.equal(bundle.format,'jc057-native84-raw-v1');
  assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
  const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);
  return {...bundle,...raw};
}
export function compare(fixture,abi,bytes) {
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
  if(!destination)throw Error('Usage: jc057-oracle.mjs Native84.json fresh84.wasm NEW.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  const bundle={format:'jc057-native84-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current84-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
