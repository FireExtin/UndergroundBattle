// Fresh current Native probe; the original ABI87 P2 archive stays untouched.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import {assertSources,sha256} from '../../tools/xq23-oracle.mjs';
import {kernel,moduleBytes,catalog} from './current88-kernel.mjs';

const file=process.argv[2]||new URL('../../docs/evidence/xq23-2026-10-19/current-p2.native.json.gz',import.meta.url);
const input=readFileSync(file),raw=input[0]===0x1f&&input[1]===0x8b?gunzipSync(input):input;
const f=JSON.parse(raw);assert.equal(f.format,'jc041-source-region-current-v1');assertSources(f.sourceBindings);
assert.equal(f.currentIdentity,catalog.engineVersion);assert.equal(f.expectedAccepted,false);
assert.equal(f.archivedStateSha256,'ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c');
assert.deepEqual(f.cases.map(c=>c.name).sort(),['source-self','known-hand-nonregion','known-paid-asset-nonregion','known-region-wrong-slot'].sort());
assert.equal(f.legal.length,4);let badChecks=0,commands=0,views=0;
f.originalViews.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(f.originalState,seat)),v);views++;});
for(const c of f.cases){
  assert(c.gameError&&c.roomError);
  const error=e=>{assert.equal(String(e),c.roomError);return true;};
  for(let seat=0;seat<4;seat++){assert.throws(()=>kernel.view(c.state,seat),error);badChecks++;}
  assert.throws(()=>kernel.applyRoom(c.state,0,JSON.stringify(c.command),'0'),error);badChecks++;
  assert.throws(()=>kernel.pollRoom(c.state,0,'0'),error);badChecks++;
}
for(const s of f.legal){
  let state=s.initial.state;
  s.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),v);views++;});
  for(const row of s.steps){
    assert.equal(row.state,state);
    assert.deepEqual(JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;
    row.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(row.expected.state,seat)),v);views++;});state=row.expected.state;
  }
}
assert.deepEqual({badChecks,commands,views},{badChecks:24,commands:56,views:244});
console.log(JSON.stringify({scope:'Fresh Native88 source-region probe; no old-state version or UID replacement',badChecks,commands,views,nativeInputs:139,nativeRawBytes:raw.length,nativeRawSha256:sha256(raw),wasmSha256:sha256(moduleBytes),allOpaqueStatesJournalsAndViewsEqual:true}));
