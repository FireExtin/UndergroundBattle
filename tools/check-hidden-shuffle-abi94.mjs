import assert from 'node:assert/strict';
import {readFileSync,readdirSync,writeFileSync,existsSync} from 'node:fs';
import {join} from 'node:path';
import {kernel,catalog} from '../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog,currentSourceManifest,currentSourceManifestSha256,currentWasmSha256,writeCurrentFixture,loadCurrentFixture,sha256} from './current-native-source.mjs';

const destination='docs/evidence/hidden-shuffle-markers-2026-10-22/abi94-shuffle.current.json.gz';
const [mode,rawDirectory,nativeManifest,reportPath]=process.argv.slice(2);
assert(['wrap','check'].includes(mode));
assert.equal(catalog.engineVersion,'rust-v0.2.94-batch20-third');
assert.equal(catalog.cards.length,213);
assert(!catalog.cards.some(c=>['WM045','WM058','WM029','XG11'].includes(c.id)),'No fourth-batch pilots in ABI94 repair');
let bundle;
if(mode==='wrap'){
  assert(!existsSync(destination));
  assert.deepEqual(JSON.parse(readFileSync(nativeManifest,'utf8')).files,currentSourceManifest.files,'Inputs must still match the actual Native compiler run');
  const cases=readdirSync(rawDirectory).filter(n=>n.endsWith('.json')).sort().map(name=>{
    const json=readFileSync(join(rawDirectory,name),'utf8'),c=JSON.parse(json);
    assert.equal(JSON.stringify(c),json,'Keep exact fresh Native JSON bytes');
    return {name,json,bytes:Buffer.byteLength(json),sha256:sha256(json),case:c};
  });
  assert.equal(cases.length,4);bundle={cases:cases.map(c=>c.case),nativeRaw:cases.map(({case:_,...raw})=>raw)};
}else bundle=loadCurrentFixture(destination);
let commands=0,views=0;const cases=[];
for(const c of bundle.cases){
  assertCurrentCatalog(c.catalog);let state=c.initial.state;
  const seed=/"seed":(\d+)/.exec(state)?.[1];assert(seed);
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),c.initial.views[seat]);views++;}
  for(const step of c.steps){
    assert.equal(state,step.state,'Never rewrite or relabel a Native room');
    const actual=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
    assert.deepEqual(actual,step.expected);
    assert.equal(actual.state,step.expected.state,'Preserve exact JSON and u64 decimal text');
    for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(actual.state,seat)),step.views[seat]);views++;}
    state=actual.state;commands++;
  }
  assert(state.includes(`"seed":${seed}`));cases.push({name:c.name,commands:c.steps.length,seedDecimal:seed});
}
if(mode==='wrap')writeCurrentFixture(destination,bundle);
const report={engineVersion:catalog.engineVersion,definitions:catalog.cards.length,sourceInventoryFiles:Object.keys(currentSourceManifest.files).length,currentSourceManifestSha256,wasmSha256:currentWasmSha256,cases,commands,views,exactStateStrings:true,scope:'Author fresh ABI94 Native/WASM comparison, no browser or public acceptance'};
if(reportPath)writeFileSync(reportPath,JSON.stringify(report,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(report));
