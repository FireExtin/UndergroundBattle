// Current Native outputs only; opaque states are never parsed or rewritten.
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { kernel, moduleBytes } from './current-kernel.mjs';
const root=resolve(process.argv[2]);
let steps=0, snapshots=0, projections=0, rejected=0;
for(const name of readdirSync(resolve(root,'trace')).filter(n=>n.endsWith('.json')).sort()) {
  const row=JSON.parse(readFileSync(resolve(root,'trace',name)));
  if(row.command) {
    assert.deepEqual(JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify(row.command),'0')),row.expected,name);
    for(let seat=0;seat<row.views.length;seat++) {
      assert.deepEqual(JSON.parse(kernel.view(row.expected.state,seat)),row.views[seat],name);projections++;
    }
    steps++;
  } else {
    for(let seat=0;seat<row.views.length;seat++) {
      assert.deepEqual(JSON.parse(kernel.view(row.state,seat)),row.views[seat],name);projections++;
    }snapshots++;
  }
}
for(const name of readdirSync(resolve(root,'invalid')).sort()) {
  const row=JSON.parse(readFileSync(resolve(root,'invalid',name)));
  assert.throws(()=>kernel.view(row.state,0),name);
  assert.throws(()=>kernel.applyRoom(row.state,0,JSON.stringify({commandId:'illegal',expectedVersion:0,action:{kind:'game',action:{kind:'pass'}}}),'0'),name);
  rejected++;
}
console.log(JSON.stringify({scope:'current Native→WASM full transitions/four-seat views and invalid state rejection',steps,snapshots,projections,rejected,
  wasmSha256:createHash('sha256').update(moduleBytes).digest('hex')}));
