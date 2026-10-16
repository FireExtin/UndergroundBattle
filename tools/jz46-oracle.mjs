// Fresh ABI79 Native observations. Never relabel or rewrite opaque states.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { gzipSync, gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const moduleUrl = import.meta.url;
const root = fileURLToPath(new URL('../', moduleUrl));
export const sha256 = value => createHash('sha256').update(value).digest('hex');
export const identity = { rulesVersion: 'hegemony-pdf-v1', cardPoolVersion: 'limited-v2.72-jz46-private-criminal-inspection-candidate', engineVersion: 'rust-v0.2.79-jz46-private-criminal-inspection-candidate' };
export const positiveNames = ['criminal-seat0','criminal-seat1','criminal-seat2','criminal-seat3','noncriminal-unchanged','all-hidden-no-identity-leak','target-paid-reveal-cancels','pause-inspection-resume','repeat-lock-next-turn'].sort();
export const negativeNames = [...Array.from({length:22},(_,i)=>`pending-${String(i).padStart(2,'0')}`),...Array.from({length:8},(_,i)=>`response-${String(i).padStart(2,'0')}`),'effect-transplant','duplicate-pending-stack'].sort();
export function assertSources(binding) {
  assert.equal(binding.algorithm, 'sha256');
  const files = ['Cargo.lock','Cargo.toml','rust-game-wasm/Cargo.toml','rust-game-wasm/build.sh','rust-game-wasm/src/lib.rs','rust-game/Cargo.toml','rust-game/data/cards.json',
    ...readdirSync(path.join(root,'rust-game/src'),{recursive:true}).filter(n=>n.endsWith('.rs')).map(n=>'rust-game/src/'+n)].sort();
  assert.equal(files.length,123,'Review newly added Native compiler inputs explicitly');
  assert.deepEqual(Object.keys(binding.files).sort(), files, 'Incomplete Native compiler binding');
  for (const file of files) assert.equal(sha256(readFileSync(path.join(root,file))),binding.files[file],`Stale Native source: ${file}`);
}
function decode(rows,names,binding) {
  assert.deepEqual(rows.map(r=>r.file).sort(),names.map(n=>n+'.json'),'Incomplete Native input set');
  return rows.map(row=>{
    assert.equal(Buffer.byteLength(row.json),row.bytes);assert.equal(sha256(row.json),row.sha256,'Native raw byte hash mismatch');
    const value=JSON.parse(row.json);assert.deepEqual(value.sourceBindings,binding,'Mixed Native compiler inputs');return value;
  });
}
export function validateBundle(bundle) {
  assert.equal(bundle.format,'jz46-native79-raw-v1'); assertSources(bundle.sourceBindings);
  for(const [k,v] of Object.entries(identity))assert.equal(bundle.catalog[k],v);
  assert.equal(bundle.catalog.cards.length,145);
  const scenarios=decode(bundle.positive,positiveNames,bundle.sourceBindings),invalid=decode(bundle.negative,negativeNames,bundle.sourceBindings);
  let commands=0,projections=0,rejected=0;
  for(const scenario of scenarios){
    assert.deepEqual(scenario.catalog,bundle.catalog);assert.equal(scenario.initial.views.length,4);
    let state=scenario.initial.state;assert.equal(typeof state,'string');projections+=4;assert(scenario.steps.length>0);
    for(const step of scenario.steps){
      assert.equal(step.state,state,'Discontinuous opaque state');assert.equal(step.serverNowMs,'0');assert.equal(step.views.length,4);
      assert.deepEqual(step.expected.view,step.views[step.seat]);assert.equal(typeof step.expected.state,'string');
      if(step.expected.errorCode){assert.equal(step.expected.outcome,'rejected');assert.equal(step.expected.state,state);rejected++;}
      else assert.equal(step.expected.outcome,'accepted');
      state=step.expected.state;commands++;projections+=4;
    }
  }
  for(const row of invalid){assert.equal(typeof row.state,'string');assert.equal(typeof row.gameState,'string');assert(row.gameError&&row.roomError);}
  const counts={scenarios:scenarios.length,commands,projections,rejected,invalid:invalid.length};
  assert.deepEqual(bundle.counts,counts,'Incomplete continuous Native command set');
  return {...bundle,scenarios,invalid};
}
export function loadFixture(){return validateBundle(JSON.parse(gunzipSync(readFileSync(path.join(root,'rust-game-wasm/tests/jz46Native79.fixture.json.gz')))));}
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
    check(()=>abi.quoteRoom(row.state,seat,'{}','0'));
  }
  return {...fixture.counts,negativeRestoreCalls:fixture.invalid.length*12,wasmSha256:fixture.wasmSha256,completeOpaqueStateAndAllViews:true};
}
function inputs(directory,names){
  assert.deepEqual(readdirSync(directory).sort(),names.map(n=>n+'.json'));
  return names.map(name=>{const json=readFileSync(path.join(directory,name+'.json'),'utf8');return {file:name+'.json',bytes:Buffer.byteLength(json),sha256:sha256(json),json};});
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  const [positiveDir,negativeDir,output]=process.argv.slice(2);assert(positiveDir&&negativeDir&&output,'Provide exact positive/negative directories and a new output');
  const {kernel,moduleBytes,catalog}=await import('../rust-game-wasm/tests/current79-kernel.mjs');
  const positive=inputs(positiveDir,positiveNames),negative=inputs(negativeDir,negativeNames);
  const parsed=positive.map(row=>JSON.parse(row.json));
  const bundle={format:'jz46-native79-raw-v1',sourceBindings:parsed[0].sourceBindings,catalog,wasmSha256:sha256(moduleBytes),
    counts:{scenarios:parsed.length,commands:parsed.reduce((n,s)=>n+s.steps.length,0),projections:parsed.reduce((n,s)=>n+4+s.steps.length*4,0),rejected:parsed.reduce((n,s)=>n+s.steps.filter(r=>r.expected.errorCode).length,0),invalid:negative.length},positive,negative};
  const result=compare(validateBundle(bundle),kernel,moduleBytes);
  writeFileSync(output,gzipSync(JSON.stringify(bundle),{level:9}),{flag:'wx'});console.log(JSON.stringify(result));
}
