// Fresh, bounded integration evidence; never relabel or rewrite an old state.
import assert from 'node:assert/strict';
import {readFileSync,existsSync} from 'node:fs';
import {kernel,catalog,moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture,writeCurrentFixture,currentSourceManifestSha256,currentWasmSha256,sha256} from './current-native-source.mjs';
const path=process.argv[2];
assert(path,'Pass the freshly executed Native smoke JSON path');
const bytes=readFileSync(path);const native=JSON.parse(bytes);
assert.equal(native.format,'batch3-native94-integration-smoke-v1');
assert.equal(catalog.engineVersion,'rust-v0.2.94-batch20-third');
assert.deepEqual(native.catalog,catalog,'Native/WASM full catalog must match');
assert.equal(catalog.cards.length,212);
assert.equal(catalog.cards.filter(c=>c.supported).length,193);
assert.equal(catalog.cards.filter(c=>!c.supported).length,19);
assert(!catalog.cards.some(c=>c.id==='CQ11'||c.id==='TK008'));
assert(BigInt(native.seedDecimal)>BigInt(Number.MAX_SAFE_INTEGER));
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
const output=process.argv[3] || 'docs/evidence/batch20-third-2026-10-21/abi94-integration-smoke-privacy.current.json.gz';
if (existsSync(output)) assert.deepEqual(loadCurrentFixture(output),native,'Existing evidence must match; never overwrite or relabel it');
else writeCurrentFixture(output,native);
process.stdout.write(JSON.stringify({identity:{rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion},nativeCommands:native.steps.length,fourPrivateViewsPerStep:true,wasmBytes:moduleBytes.length,wasmSha256:currentWasmSha256,currentSourceManifestSha256,nativeRawJsonBytes:bytes.length,nativeRawJsonSha256:sha256(bytes),catalogDefinitions:212,constructionSupported:193,pendingDisabled:19,publicBatchAdmissions:0,output,scope:'Author Native/WASM integration smoke only; no full-suite, independent or browser acceptance'},null,2)+'\n');
