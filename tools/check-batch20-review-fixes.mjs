// Exact fresh Native/production-WASM comparison for normal review regressions.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import {kernel,catalog,moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog,writeCurrentFixture,sha256,currentSourceManifestSha256} from './current-native-source.mjs';
const [nativePath,resultPath,fixturePath='rust-game-wasm/tests/current112/batch20-review-normal.json.gz']=process.argv.slice(2);
assert(nativePath&&resultPath&&!existsSync(resultPath),'Fresh input and NEW result required');
const nativeBytes=readFileSync(nativePath),fixture=JSON.parse(nativeBytes);
assert.equal(fixture.format,'abi112-review-normal-native-v1');
assert.equal(catalog.engineVersion,'rust-v0.2.112-shared-keyword-rewards');
assertCurrentCatalog(fixture.catalog);
assert.deepEqual(fixture.cases.map(c=>c.name),['cq28-mill-0','cq28-mill-3',
  'mystic-order-false','mystic-order-true','eye-order-false','eye-order-true']);
let commands=0,views=0,duplicates=0;
for(const scenario of fixture.cases){
  let state=scenario.initial.state;
  assert(state.includes(`"seed":${scenario.seedDecimal}`));
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),scenario.initial.views[seat]);views++;}
  for(const row of scenario.steps){
    assert.equal(row.state,state);
    const actual=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs));
    assert.deepEqual(actual,row.expected,`${scenario.name} command${commands}`);state=actual.state;commands++;
    assert(state.includes(`"seed":${scenario.seedDecimal}`));
    for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),row.views[seat]);views++;}
    const duplicate=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs));
    assert.equal(duplicate.outcome,'rejected');assert.equal(duplicate.changed,false);
    assert.equal(duplicate.state,state);assert.deepEqual(duplicate.journal,[]);duplicates++;
  }
}
assert(fixturePath.startsWith('rust-game-wasm/tests/current112/')&&fixturePath.endsWith('.json.gz'));
writeCurrentFixture(fixturePath,fixture);
const result={engineVersion:catalog.engineVersion,currentSourceManifestSha256,wasmSha256:sha256(moduleBytes),
  nativeSha256:sha256(nativeBytes),scenarios:fixture.cases.map(c=>c.name),commands,views,duplicates,
  fullCatalogEqual:true,opaqueStateAndU64Exact:true,
  scope:'Author actual current WASM/Native Room normal command comparison, not a browser match or independent review.'};
writeFileSync(resultPath,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify(result));
