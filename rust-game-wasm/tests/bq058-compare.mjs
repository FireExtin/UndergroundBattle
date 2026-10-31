// Only fresh BQ058 native evidence from this exact current kernel is accepted.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,moduleBytes,catalog} from './current-kernel.mjs';
const packet=JSON.parse(readFileSync(process.argv[2],'utf8'));
assert.deepEqual(packet.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.114-bq058-nontask-wip');
assert.equal(catalog.cards.find(c=>c.id==='BQ058').supported,false);
assert.equal(packet.cases.length,3);
let commands=0,views=0;
for(const c of packet.cases){
  for(let seat=0;seat<c.initial.views.length;seat++){
    assert.deepEqual(JSON.parse(kernel.view(c.initial.state,seat)),c.initial.views[seat],`${c.name} initial ${seat}`);views++;
  }
  for(const row of c.steps){
    assert.deepEqual(JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,c.name);commands++;
    for(let seat=0;seat<row.views.length;seat++){
      assert.deepEqual(JSON.parse(kernel.view(row.expected.state,seat)),row.views[seat],`${c.name} seat ${seat}`);views++;
    }
  }
}
console.log(JSON.stringify({scope:packet.scope,cases:packet.cases.length,commands,views,engine:catalog.engineVersion,wasmSha256:createHash('sha256').update(moduleBytes).digest('hex')}));
