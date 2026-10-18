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
export const scenarioNames=['reveal-p0','reveal-p1','reveal-p2','reveal-p3','empty','skip','deploy','live-exhaust','live-hide','live-control','granted-renown','granted-glory','granted-mobility'].sort();
export const invalidNames=['seat-only','seat-and-player-id','player-id-only','stage-accept','stage-mode','stage-attachment-order','kind','title','description','min','max','missing-min','missing-max','allow-decline','amount','preview','options-missing','options-duplicate','options-order','option-id','option-label','option-card','empty-id','eliminated-actor'].sort();
export function assertSources(binding) {
  const saved=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi84/native-source-binding.json'),'utf8'));
  assert.equal(binding.algorithm,'sha256');assert.equal(Object.keys(saved.files).length,131);
  assert.deepEqual(binding,saved,'Historical Native84 compiler binding must remain unchanged');
}


export function validateNative(raw) {
  assert.equal(raw.format,'jc057-native84-v2');assertSources(raw.sourceBindings);
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
  assert.deepEqual(raw.invalid.map(row=>row.name).sort(),invalidNames);
  for(const row of raw.invalid) {
    assert.equal(typeof row.state,'string');assert(row.gameError.length>0);assert(row.roomError.length>0);
    assert.equal(row.revision,13);assert.equal(row.choiceId,'i249');
  }
  return {scenarios:raw.scenarios.length,commands,rejected,projections,invalidRestores:raw.invalid.length};
}
export function loadFixture() {
  const bytes=readFileSync(path.join(root,'rust-game-wasm/tests/jc057Native84.fixture.json.gz'));
  const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi84/identity.json'),'utf8'));
  assert.equal(sha256(bytes),pinned.nativeFixtureSha256,'Published Native84 fixture bytes must not change');
  const bundle=JSON.parse(gunzipSync(bytes));assert.equal(bundle.format,'jc057-native84-raw-v2');
  assert.equal(bundle.raw.sha256,pinned.nativeRawSha256);assert.equal(bundle.raw.bytes,pinned.nativeRawBytes);
  assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
  const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);
  return {...bundle,...raw};
}
export function compare(fixture,abi,bytes) {
  const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi84/identity.json'),'utf8'));
  assert.equal(sha256(bytes),pinned.wasmSha256,'Only the exact published v66 WASM may check Native84');
  assert.equal(sha256(bytes),fixture.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),fixture.catalog);
  for(const scenario of fixture.scenarios) {
    scenario.initial.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(scenario.initial.state,seat)),view,scenario.name+' initial'));
    for(const row of scenario.steps) {
      assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,scenario.name+' command');
      row.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),view,scenario.name+' view'));
    }
  }
  for(const row of fixture.invalid) {
    const nativeError=error=>{assert.equal(String(error),row.roomError,row.name+' Native/WASM restore error');return true;};
    for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(row.state,seat),nativeError,row.name+' view');
    const command={commandId:'invalid-'+row.name,expectedVersion:row.revision,
      action:{kind:'game',action:{kind:'choose',choiceId:row.choiceId,selected:[]}}};
    assert.throws(()=>abi.applyRoom(row.state,row.seat,JSON.stringify(command),'0'),nativeError,row.name+' choose');
    assert.throws(()=>abi.pollRoom(row.state,0,'0'),nativeError,row.name+' poll');
  }
  return {...fixture.counts,wasmSha256:fixture.wasmSha256,nativeRawSha256:fixture.raw.sha256,completeOpaqueStatesJournalsAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: jc057-oracle.mjs Native84.json fresh84.wasm NEW.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  const pinned=JSON.parse(readFileSync(path.join(root,'rust-game-wasm/frozen/abi84/identity.json'),'utf8'));
  assert.equal(sha256(json),pinned.nativeRawSha256,'Only original published Native84 bytes may be restored');
  assert.equal(sha256(bytes),pinned.wasmSha256,'A newer build cannot re-sign Native84');
  const bundle={format:'jc057-native84-raw-v2',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current84-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
