// Fresh Native Room witnesses run against the actual current WASM. No state
// version rewriting or old-fixture adaptation is permitted.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import {kernel,catalog,moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog,writeCurrentFixture,sha256,currentSourceManifestSha256} from './current-native-source.mjs';
const [nativePath,resultPath,fixturePath='rust-game-wasm/tests/current112/shared-rewards.json.gz']=process.argv.slice(2);
assert(nativePath&&resultPath&&!existsSync(resultPath),'Fresh Native input and NEW result required');
const fixture=JSON.parse(readFileSync(nativePath,'utf8'));
function safeNumbers(value){
 if(typeof value==='number')assert(Number.isSafeInteger(value),'Protocol numbers must be safe; private u64 state stays an opaque string');
 else if(value&&typeof value==='object')for(const item of Object.values(value))safeNumbers(item);
}
safeNumbers(fixture);
assert.equal(fixture.format,'shared-reward-native112-v1');
assert.equal(catalog.engineVersion,'rust-v0.2.112-shared-keyword-rewards');
assertCurrentCatalog(fixture.catalog);
assert.deepEqual(fixture.cases.map(c=>c.name),['WM030','XQ19','CQ24','BQ026']);
let commands=0,views=0;
for(const scenario of fixture.cases){
 let state=scenario.initial.state;
 assert(state.includes(`"seed":${scenario.seedDecimal}`));
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),scenario.initial.views[seat]);views++;}
 for(const row of scenario.steps){
  assert.equal(state,row.state);
  const actual=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs));
  safeNumbers(actual);
  assert.deepEqual(actual,row.expected);state=actual.state;
  assert(state.includes(`"seed":${scenario.seedDecimal}`));
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),row.views[seat]);views++;}
  commands++;
 }
}
assert(fixturePath.startsWith('rust-game-wasm/tests/current112/')&&fixturePath.endsWith('.json.gz'));
writeCurrentFixture(fixturePath,fixture);
const result={engineVersion:catalog.engineVersion,cardPoolVersion:catalog.cardPoolVersion,
 currentSourceManifestSha256,wasmSha256:sha256(moduleBytes),wasmBytes:moduleBytes.length,
 nativeSha256:sha256(readFileSync(nativePath)),scenarios:fixture.cases.map(c=>c.name),commands,views,
 fullCatalogEqual:true,opaqueStateAndU64Exact:true,
 scope:'Author real WASM Room accept/respond/placement on four real granted-reward recipients. Not browser, all272 card acceptance, or independent review.'};
writeFileSync(resultPath,JSON.stringify(result,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(result));
