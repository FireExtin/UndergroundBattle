import * as abi from '/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm.js';
import { readFileSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
const wasm = readFileSync('/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
abi.initSync({ module: wasm });
const trace = JSON.parse(readFileSync('/tmp/pause-resume-evidence/native-trace.json','utf8'));
let state = trace.initialState, views = 0;
for (const step of trace.steps) {
  const actual = JSON.parse(step.command ? abi.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs) : abi.pollRoom(state,step.seat,step.serverNowMs));
  assert.deepEqual(actual,step.transition);
  state = actual.state;
  for (let seat=0;seat<4;seat++) { assert.deepEqual(JSON.parse(abi.view(state,seat)),step.views[seat]);views++; }
  if (actual.view.pause) {
    assert.throws(()=>abi.quoteRoom(state,0,JSON.stringify({windowId:actual.view.responseWindow.id,intentId:'saved-composition',draft:null})),error=>String(error).includes('暂停'));
  }
}
const catalog = JSON.parse(abi.catalog());
const result={actualWasm:true,transitions:trace.steps.length,views,opaqueStateByteEqual:true,fullTransitionEqual:true,engine:catalog.engineVersion,pool:catalog.cardPoolVersion,wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex')};
writeFileSync('/tmp/pause-resume-evidence/native-wasm-parity.json',JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result));
