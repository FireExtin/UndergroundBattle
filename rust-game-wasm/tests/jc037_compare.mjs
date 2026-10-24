// Compare source-produced Native states, complete commands and all seat views.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
assert.equal(catalog.engineVersion,'rust-v0.2.112-shared-keyword-rewards');
assert.equal(catalog.cardPoolVersion,'limited-v2.83-batch20-next');
const fixture=loadCurrentFixture(process.argv[2] || 'rust-game-wasm/tests/current112/jc037.json.gz');
assert.deepEqual(fixture.catalog,catalog);
assert.equal(createHash('sha256').update(moduleBytes).digest('hex'),fixture.wasmSha256);
let steps=0,views=0,rejected=0;
for(const scenario of fixture.scenarios) {
  let state=scenario.initial.state;
  scenario.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),v);views++;});
  for(const step of scenario.steps) {
    assert.equal(step.state,state);
    const actual=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
    assert.deepEqual(actual,step.expected,scenario.name);
    state=actual.state;steps++;
    if(actual.errorCode)rejected++;
    step.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),v);views++;});
  }
}
console.log(JSON.stringify({scenarios:fixture.scenarios.length,steps,seatViews:views,rejected,wasmSha256:fixture.wasmSha256,completeNativeStatesEqual:true}));
