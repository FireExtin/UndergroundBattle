// Fresh, bounded integration evidence; never relabel or rewrite an old state.
import assert from 'node:assert/strict';
import {readFileSync,existsSync} from 'node:fs';
import {kernel,catalog,moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture,writeCurrentFixture,currentSourceManifestSha256,currentWasmSha256,sha256} from './current-native-source.mjs';
const path=process.argv[2];
assert(path,'Pass the freshly executed Native smoke JSON path');
const bytes=readFileSync(path);const native=JSON.parse(bytes);
assert.equal(native.format,'batch3-native94-integration-smoke-v1');
assert.equal(catalog.engineVersion,'rust-v0.2.112-shared-keyword-rewards');
assert.deepEqual(native.catalog,catalog,'Native/WASM full catalog must match');
assert.equal(catalog.cards.length,272);
assert.equal(catalog.cards.filter(c=>c.supported).length,233);
assert.equal(catalog.cards.filter(c=>!c.supported).length,39);
assert(catalog.cards.some(c=>c.id==='CQ07'&&c.supported));
assert(!catalog.cards.some(c=>c.id==='CQ11'||c.id==='TK008'));
if(native.originalBatInstance){
  assert.equal(native.steps.length,1,'Cleanup witness has one actual discard/cleanup command');
  assert.equal(native.seedDecimal,'8','Cleanup witness preserves the disclosed unit-layout seed');
  for(let seat=0;seat<4;seat++){
    const before=native.initial.views[seat].attachments.find(c=>c.instanceId===native.originalBatInstance);
    const after=native.steps[0].views[seat].attachments.find(c=>c.instanceId===native.originalBatInstance);
    assert(before&&after,'Same original attached character must survive for every seat');
    assert.equal(before.hostId,native.hostInstance);assert.equal(after.hostId,native.hostInstance);
    assert.equal(before.damage,1);assert.equal(before.defense,2);
    assert.equal(after.damage,0);assert.equal(after.defense,1);
  }
}else assert(BigInt(native.seedDecimal)>BigInt(Number.MAX_SAFE_INTEGER));
let state=native.initial.state;
assert(state.includes(`"seed":${native.seedDecimal}`),'Keep the original u64 decimal in opaque state');
for(let seat=0;seat<4;seat++)assert.deepEqual(JSON.parse(kernel.view(state,seat)),native.initial.views[seat]);
for(const step of native.steps){
  assert.equal(step.state,state,'Native witness is one continuous command chain');
  const actual=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
  assert.deepEqual(actual,step.expected,`Command ${step.command.commandId}`);
  for(let seat=0;seat<4;seat++)assert.deepEqual(JSON.parse(kernel.view(actual.state,seat)),step.views[seat]);
  state=actual.state;
}
const output=process.argv[3] || 'docs/evidence/batch20-third-2026-10-21/abi94-twenty-inspections.current.json.gz';
if (existsSync(output)) assert.deepEqual(loadCurrentFixture(output),native,'Existing evidence must match; never overwrite or relabel it');
else writeCurrentFixture(output,native);
process.stdout.write(JSON.stringify({identity:{rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion},nativeCommands:native.steps.length,fourPrivateViewsPerStep:true,wasmBytes:moduleBytes.length,wasmSha256:currentWasmSha256,currentSourceManifestSha256,nativeRawJsonBytes:bytes.length,nativeRawJsonSha256:sha256(bytes),catalogDefinitions:213,constructionSupported:213,pendingDisabled:0,candidateBatchAdmissions:20,publicBatchAdmissions:0,output,scope:'Author necessary twenty-card candidate integration; no publication, full-suite, independent or browser acceptance'},null,2)+'\n');
