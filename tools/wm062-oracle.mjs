// Fresh ABI93 Native observations. Never relabel or rewrite opaque states.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { gzipSync, gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { currentIdentity, currentWasmSha256, assertCurrentCatalog, assertCurrentSourceBindings, currentSourceManifestSha256, currentFixturePath} from './current-native-source.mjs';
const moduleUrl = import.meta.url;
const root = fileURLToPath(new URL('../', moduleUrl));
export const sha256 = value => createHash('sha256').update(value).digest('hex');
export const identity = currentIdentity;
export const positiveNames = ['move-seat0','move-seat1','move-seat2','move-seat3','decline-cleanup','controlled-foreign-successor','target-concealed-response','target-bounced-response','pause-successor-resume','fast-end-window','no-adjacent-destination','invalid-target-and-payload','duel-adjacent-move','last-criminal-defense-loss'].sort();
export const negativeNames = [...Array.from({length:36},(_,i)=>`pending-${String(i).padStart(2,'0')}`),...Array.from({length:9},(_,i)=>`response-${String(i).padStart(2,'0')}`)].sort();
export const assertSources = assertCurrentSourceBindings;
function decode(rows,names,binding) {
  assert.deepEqual(rows.map(r=>r.file).sort(),names.map(n=>n+'.json'),'Incomplete Native input set');
  return rows.map(row=>{
    assert.equal(Buffer.byteLength(row.json),row.bytes);assert.equal(sha256(row.json),row.sha256,'Native raw byte hash mismatch');
    const value=JSON.parse(row.json);assert.deepEqual(value.sourceBindings,binding,'Mixed Native compiler inputs');return value;
  });
}
export function validateBundle(bundle) {
  assert.equal(bundle.format,'wm062-native93-raw-v1'); assertSources(bundle.sourceBindings);
  assert.equal(bundle.currentSourceManifestSha256,currentSourceManifestSha256);
  assert.equal(bundle.wasmSha256,currentWasmSha256);
  assertCurrentCatalog(bundle.catalog);
  const scenarios=decode(bundle.positive,positiveNames,bundle.sourceBindings),invalid=decode(bundle.negative,negativeNames,bundle.sourceBindings);
  let commands=0,projections=0,rejected=0;
  for(const scenario of scenarios){
    assert.deepEqual(scenario.catalog,bundle.catalog);assert([2,4].includes(scenario.initial.views.length));
    let state=scenario.initial.state;assert.equal(typeof state,'string');projections+=scenario.initial.views.length;assert(scenario.steps.length>0);
    for(const step of scenario.steps){
      assert.equal(step.state,state,'Discontinuous opaque state');assert.equal(step.serverNowMs,'0');assert.equal(step.views.length,scenario.initial.views.length);
      assert.deepEqual(step.expected.view,step.views[step.seat]);assert.equal(typeof step.expected.state,'string');
      if(step.expected.errorCode){assert.equal(step.expected.outcome,'rejected');assert.equal(step.expected.state,state);rejected++;}
      else assert.equal(step.expected.outcome,'accepted');
      state=step.expected.state;commands++;projections+=scenario.initial.views.length;
    }
  }
  for(const row of invalid){assert.equal(typeof row.state,'string');assert.equal(typeof row.gameState,'string');assert(row.gameError&&row.roomError);}
  const counts={scenarios:scenarios.length,commands,projections,rejected,invalid:invalid.length};
  assert.deepEqual(bundle.counts,counts,'Incomplete continuous Native command set');
  return {...bundle,scenarios,invalid};
}
export function loadFixture(){return validateBundle(JSON.parse(gunzipSync(readFileSync(currentFixturePath('rust-game-wasm/tests/wm062Native112.fixture.json.gz')))));}
export function compare(fixture,abi,bytes){
  assert.equal(sha256(bytes),fixture.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),fixture.catalog);
  for(const s of fixture.scenarios){
    s.initial.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(s.initial.state,seat)),view));
    for(const row of s.steps){
      assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),'0')),row.expected,s.name);
      row.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),view));
    }
  }
  for(const row of fixture.invalid)for(let seat=0;seat<4;seat++){
    const check=fn=>{let error;try{fn();}catch(e){error=String(e);}assert.equal(error,row.roomError);};
    check(()=>abi.view(row.state,seat));
    check(()=>abi.applyRoom(row.state,seat,JSON.stringify({commandId:'invalid',expectedVersion:0,action:{kind:'pauseRoom'}}),'0'));
    check(()=>abi.quoteRoom(row.state,seat,'{}'));
  }
  return {...fixture.counts,negativeRestoreCalls:fixture.invalid.length*12,wasmSha256:fixture.wasmSha256,completeOpaqueStateAndAllViews:true};
}
function inputs(directory,names){
  assert.deepEqual(readdirSync(directory).sort(),names.map(n=>n+'.json'));
  return names.map(name=>{const json=readFileSync(path.join(directory,name+'.json'),'utf8');return {file:name+'.json',bytes:Buffer.byteLength(json),sha256:sha256(json),json};});
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  const [positiveDir,negativeDir,output]=process.argv.slice(2);assert(positiveDir&&negativeDir&&output,'Provide exact positive/negative directories and a new output');
  const {kernel,moduleBytes,catalog}=await import('../rust-game-wasm/tests/current-kernel.mjs');
  const positive=inputs(positiveDir,positiveNames),negative=inputs(negativeDir,negativeNames);
  const parsed=positive.map(row=>JSON.parse(row.json));
  const bundle={format:'wm062-native93-raw-v1',currentSourceManifestSha256,sourceBindings:parsed[0].sourceBindings,catalog,wasmSha256:sha256(moduleBytes),
    counts:{scenarios:parsed.length,commands:parsed.reduce((n,s)=>n+s.steps.length,0),projections:parsed.reduce((n,s)=>n+s.initial.views.length+s.steps.length*s.initial.views.length,0),rejected:parsed.reduce((n,s)=>n+s.steps.filter(r=>r.expected.errorCode).length,0),invalid:negative.length},positive,negative};
  const result=compare(validateBundle(bundle),kernel,moduleBytes);
  writeFileSync(output,gzipSync(JSON.stringify(bundle),{level:9}),{flag:'wx'});console.log(JSON.stringify(result));
}
