// Execute fresh Native Room evidence against this locally rebuilt current ABI.
import assert from 'node:assert/strict';
import {existsSync,readFileSync,writeFileSync} from 'node:fs';
import {kernel,catalog,moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';
import {currentSourceManifest,currentSourceManifestSha256,currentWasmSha256,sha256,writeCurrentFixture,loadCurrentFixture} from './current-native-source.mjs';
const nativeBytes=readFileSync(process.argv[2]);
const native=JSON.parse(nativeBytes);
assert.equal(native.format,'jc005-society-native94-v1');
assert.deepEqual(native.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.94-batch20-third');
assert.equal(catalog.cards.length,213);
assert.equal(catalog.cards.filter(c=>c.supported).length,213);
assert(!catalog.cards.some(c=>c.id==='BQ041'||c.id==='XG14'||c.id==='CQ11'));
assert.equal(native.cases.length,3);
let commands=0,rejections=0,responses=0;
for(const c of native.cases){
  assert(BigInt(c.seedDecimal)>BigInt(Number.MAX_SAFE_INTEGER));
  let state=c.initial.state;
  assert(state.includes(`"seed":${c.seedDecimal}`));
  for(let seat=0;seat<4;seat++)assert.deepEqual(JSON.parse(kernel.view(state,seat)),c.initial.views[seat]);
  for(const step of c.steps){
    assert.equal(step.state,state,'One continuous Native command chain');
    const result=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
    assert.deepEqual(result,step.expected,step.command.commandId);
    for(let seat=0;seat<4;seat++)assert.deepEqual(JSON.parse(kernel.view(result.state,seat)),step.views[seat]);
    state=result.state;commands++;
    if(result.outcome==='rejected')rejections++;
    else if(step.command.action.kind==='submitResponse')responses++;
  }
}
assert.equal(rejections,3);assert.equal(responses,2);
const output='docs/evidence/jc060-attached-character-2026-10-22/abi94-society.current.json.gz';
if(existsSync(output))assert.deepEqual(loadCurrentFixture(output),native,'Never overwrite or relabel old evidence');
else writeCurrentFixture(output,native);
const inventory='docs/evidence/jc060-attached-character-2026-10-22/abi94-source-inventory.json';
if(existsSync(inventory))assert.deepEqual(JSON.parse(readFileSync(inventory)),currentSourceManifest);
else writeFileSync(inventory,JSON.stringify(currentSourceManifest,null,2)+'\n',{flag:'wx'});
process.stdout.write(JSON.stringify({format:'jc005-society-abi94-author-parity-v1',identity:Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]])),cases:3,commands,rejectedCommands:rejections,acceptedResponseSubmits:responses,fourViews:4*(commands+3),originalHighU64Seeds:true,wasmBytes:moduleBytes.length,wasmSha256:currentWasmSha256,sourceManifestSha256:currentSourceManifestSha256,sourceManifestFiles:Object.keys(currentSourceManifest.files).length,nativeRawJsonBytes:nativeBytes.length,nativeRawJsonSha256:sha256(nativeBytes),output,scope:'Author targeted Native/WASM comparison only; no full-suite, independent, publication or browser acceptance'},null,2)+'\n');
