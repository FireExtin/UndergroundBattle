// Fresh Native81 bytes and observations; historical Native80 is never rewritten.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { gzipSync, gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
// Keep the Node directory URL out of Vite's static web-asset rewriting.
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity={rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.74-jc054-complete-conceal-candidate',engineVersion:'rust-v0.2.81-jc054-complete-conceal-candidate'};
export const scenarioNames=['paid-deploy','natural-end','mandatory-end','mandatory-order','controlled-conceal','source-replaced','paid-response-reveal','pause-mandatory-order'].sort();
export const invalidNames=[...Array.from({length:12},(_,i)=>`frame-${String(i).padStart(2,'0')}`),'frame-duplicate','optional-declare','grant-hidden','grant-play-source','instance-owner','instance-controller','instance-empty','instance-duplicate','instance-collision','marker-early','marker-window','marker-nested-missing',...Array.from({length:4},(_,i)=>`order-${String(i).padStart(2,'0')}`)].sort();
export function assertSources(binding) {
  const files=['Cargo.lock','Cargo.toml','rust-game-wasm/Cargo.toml','rust-game-wasm/build.sh','rust-game-wasm/src/lib.rs','rust-game/Cargo.toml','rust-game/data/cards.json',
    ...readdirSync(path.join(root,'rust-game/src'),{recursive:true}).filter(p=>p.endsWith('.rs')).map(p=>'rust-game/src/'+p)].sort();
  assert.equal(binding.algorithm,'sha256');assert.equal(files.length,127,'Review all new Native compiler inputs explicitly');
  assert.deepEqual(Object.keys(binding.files).sort(),files,'Incomplete Native81 input set');
  for(const file of files)assert.equal(sha256(readFileSync(path.join(root,file))),binding.files[file],`Stale Native81 source: ${file}`);
}
export function validateNative(raw) {
  assert.equal(raw.format,'jc054-native81-v1');assertSources(raw.sourceBindings);
  for(const [key,value]of Object.entries(identity))assert.equal(raw.catalog[key],value);
  assert.equal(raw.catalog.cards.length,147);
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
  const bundle=JSON.parse(gunzipSync(readFileSync(path.join(root,'rust-game-wasm/tests/jc054Native81.fixture.json.gz'))));
  assert.equal(bundle.format,'jc054-native81-raw-v1');
  assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
  const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateNative(raw),bundle.counts);
  return {...bundle,...raw};
}
export function compare(fixture,abi,bytes) {
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
  return {...fixture.counts,negativeRestoreCalls:fixture.invalid.length*12,wasmSha256:fixture.wasmSha256,completeOpaqueStateAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: jc054-oracle.mjs Native81.json fresh81.wasm destination.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  const bundle={format:'jc054-native81-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
  const {kernel}=await import('../rust-game-wasm/tests/current81-kernel.mjs');
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}));
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
