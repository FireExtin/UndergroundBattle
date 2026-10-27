// Fresh Native Room evidence only; compare before packaging, never relabel old fixtures.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {kernel} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog,assertCurrentSourceBindings,writeCurrentFixture,sha256} from '../current-native-source.mjs';
assert(process.argv[2], 'Provide the fresh XQ35 Native Room JSON.');
const bytes=readFileSync(process.argv[2]);
const raw=JSON.parse(bytes);
assertCurrentSourceBindings(raw.sourceBindings);assertCurrentCatalog(raw.catalog);
assert.equal(raw.name,'xq35-grave-return');assert(raw.steps.length>1);
let state=raw.initial.state,projections=0;
for(const step of raw.steps){
 assert.equal(step.state,state);
 const actual=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
 assert.deepEqual(actual,step.expected);state=actual.state;
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),step.views[seat]);projections++;}
 assert.deepEqual(JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs)),step.duplicate);
}
writeCurrentFixture('rust-game-wasm/tests/current113/xq35.json.gz',raw);
console.log(JSON.stringify({nativeRawSha256:sha256(bytes),scenarios:1,commands:raw.steps.length,projections,duplicates:raw.steps.length,preparedLayout:true}));
