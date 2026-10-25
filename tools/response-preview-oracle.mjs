// Fresh ABI93 Native observations; original raw bytes and opaque states stay unchanged.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { gzipSync, gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import {currentIdentity,currentWasmSha256,currentSourceManifestSha256,assertCurrentCatalog,assertCurrentSourceBindings, currentFixturePath} from './current-native-source.mjs';
const moduleUrl=import.meta.url;
const root=fileURLToPath(new URL('../',moduleUrl));
export const sha256=b=>createHash('sha256').update(b).digest('hex');
export const identity=currentIdentity;
export const scenarioNames=['preview-then-real-reveal','preview-pause-expiry','preview-peer-pass-cancel'].sort();
export function assertSources(binding) {
  assertCurrentSourceBindings(binding);
}
function assertViews(views) {
  assert.equal(views.length,4);
  for(const view of views)for(const source of view.responseWindow?.previewSources||[]) {
    assert.ok(Object.keys(source).every(k=>['kind','cardId','abilityId'].includes(k)),'A preview must not carry a target or submit payload');
    assert.ok(['play','activate','reveal','privilege'].includes(source.kind));
  }
}
export function validateNative(raw) {
  assert.equal(raw.format,'response-preview-native82-v1');assertSources(raw.sourceBindings);
  assertCurrentCatalog(raw.catalog);
  assert.equal(raw.catalog.cards.length,277);assert.equal(raw.catalog.cards.filter(card=>card.supported).length,233);for(const id of ['BQ033','BQ034','CQ25','XG25','BQ009'])assert.equal(raw.catalog.cards.find(card=>card.id===id).supported,false);assert.equal(raw.catalog.societies.length,12);
  assert.deepEqual(raw.scenarios.map(s=>s.name).sort(),scenarioNames);
  let commands=0,polls=0,pureViews=0,projections=0,rejected=0,quotes=0,rejectedQuotes=0;
  for(const scenario of raw.scenarios) {
    assertViews(scenario.initial.views);projections+=4;
    let state=scenario.initial.state;assert.equal(typeof state,'string');
    for(const row of scenario.steps) {
      assert.equal(row.state,state,'Discontinuous opaque Room state: '+scenario.name);
      if(row.kind==='view'){assertViews(row.views);projections+=4;pureViews++;continue;}
      if(row.kind==='quote'){
        assert.equal(typeof row.seat,'number');assert.equal(Boolean(row.expected),!row.error);
        if(row.error)rejectedQuotes++;quotes++;continue;
      }
      assert.ok(row.kind==='command'||row.kind==='poll');
      assert.equal(row.kind==='poll',row.command===null);assert.match(row.serverNowMs,/^\d+$/);
      assert.equal(typeof row.expected.state,'string');assertViews(row.views);projections+=4;
      if(row.expected.errorCode){assert.equal(row.expected.outcome,'rejected');assert.equal(row.expected.state,state);rejected++;}
      if(!row.expected.changed)assert.equal(row.expected.state,state);
      state=row.expected.state;
      if(row.kind==='command')commands++;else polls++;
    }
  }
  return {scenarios:raw.scenarios.length,commands,polls,pureViews,projections,rejected,quotes,rejectedQuotes};
}
export function loadFixture() {
  const bytes=readFileSync(currentFixturePath('rust-game-wasm/tests/responsePreviewNative112.fixture.json.gz'));
  const bundle=JSON.parse(gunzipSync(bytes));
  assert.equal(bundle.format,'response-preview-native93-raw-v1');
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
    scenario.initial.views.forEach((v,s)=>assert.deepEqual(JSON.parse(abi.view(scenario.initial.state,s)),v,scenario.name+' initial'));
    for(const row of scenario.steps) {
      if(row.kind==='quote') {
        const quote=()=>abi.quoteRoom(row.state,row.seat,JSON.stringify(row.request));
        if(row.error){let error;try{quote();}catch(e){error=String(e);}assert.equal(error,row.error,scenario.name+' quote');}
        else assert.deepEqual(JSON.parse(quote()),row.expected,scenario.name+' quote');
      }else if(row.kind==='view')row.views.forEach((v,s)=>assert.deepEqual(JSON.parse(abi.view(row.state,s)),v,scenario.name+' pure view'));
      else {
        const actual=row.kind==='poll'?abi.pollRoom(row.state,row.seat,row.serverNowMs):abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs);
        assert.deepEqual(JSON.parse(actual),row.expected,scenario.name+' '+row.kind);
        row.views.forEach((v,s)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,s)),v,scenario.name+' post view'));
      }
    }
  }
  return {...fixture.counts,nativeInputs:Object.keys(fixture.sourceBindings.files).length,wasmSha256:fixture.wasmSha256,nativeRawSha256:fixture.raw.sha256,completeOpaqueStateAndAllViews:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [nativeFile,wasmFile,destination]=process.argv.slice(2);
  if(!destination)throw Error('Usage: response-preview-oracle.mjs fresh-native.json current93.wasm NEW-Native93.fixture.json.gz');
  const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateNative(raw);
  assert.equal(sha256(bytes),currentWasmSha256);
  const bundle={format:'response-preview-native93-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes),currentSourceManifestSha256};
  const {kernel,moduleBytes}=await import('../rust-game-wasm/tests/current-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
  const result=compare({...bundle,...raw},kernel,bytes);
  writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});
  console.log(JSON.stringify({destination,...result,nativeInputs:Object.keys(raw.sourceBindings.files).length}));
}
