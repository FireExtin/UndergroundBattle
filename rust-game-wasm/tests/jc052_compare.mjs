// Compare current Native outputs without parsing or rewriting opaque states.
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { kernel, catalog, moduleBytes } from './current-kernel.mjs';
const versions=new Map([
  ['rust-v0.2.73-jc052-original-region-candidate','limited-v2.66-jc052-original-region-candidate'],
  ['rust-v0.2.74-jc037-hidden-investigation-candidate','limited-v2.67-jc037-hidden-investigation-candidate'],
  ['rust-v0.2.75-jc060-occult-adviser-candidate','limited-v2.68-jc060-occult-adviser-candidate'],
]);
assert(versions.has(catalog.engineVersion),'Build an explicitly supported current ABI first.');
assert.equal(catalog.cardPoolVersion,versions.get(catalog.engineVersion));
const root=resolve(process.argv[2]);
let steps=0,snapshots=0,projections=0,rejected=0;
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
    }
    snapshots++;
  }
}
for(const name of readdirSync(resolve(root,'invalid')).sort()) {
  const row=JSON.parse(readFileSync(resolve(root,'invalid',name)));
  assert.throws(()=>kernel.view(row.state,0),name);
  assert.throws(()=>kernel.applyRoom(row.state,0,JSON.stringify({commandId:'illegal',expectedVersion:0,action:{kind:'game',action:{kind:'pass'}}}),'0'),name);
  rejected++;
}
console.log(JSON.stringify({scope:'current Native/WASM full transitions, four-seat views, invalid restoration',steps,snapshots,projections,rejected,
  wasmSha256:createHash('sha256').update(moduleBytes).digest('hex')}));
