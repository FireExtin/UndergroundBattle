// Full current ABI75 comparison against independently asserted Native traces.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
const fixture=JSON.parse(readFileSync(process.argv[2] || new URL('../../web/src/game/jc060Native75.fixture.json',import.meta.url)));
assert.deepEqual(fixture.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.75-jc060-occult-adviser-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.68-jc060-occult-adviser-candidate');
assert.equal(createHash('sha256').update(moduleBytes).digest('hex'),fixture.wasmSha256);
const previous=JSON.parse(readFileSync(new URL('../../web/src/game/jc037Native74.fixture.json',import.meta.url))).scenarios[0].steps[0];
assert.throws(()=>kernel.view(previous.state,previous.seat),/旧局schema\/固定版本/);
assert.throws(()=>kernel.applyRoom(previous.state,previous.seat,JSON.stringify(previous.command),previous.serverNowMs),/旧局schema\/固定版本/);
let steps=0,views=0,rejected=0;
for(const scenario of fixture.scenarios) {
  let state=scenario.initial.state;
  scenario.initial.views.forEach((view,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),view);views++;});
  for(const step of scenario.steps) {
    assert.equal(step.state,state);
    const actual=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
    assert.deepEqual(actual,step.expected,scenario.name);
    state=actual.state;steps++;
    if(actual.errorCode)rejected++;
    step.views.forEach((view,seat)=>{assert.deepEqual(JSON.parse(kernel.view(state,seat)),view);views++;});
  }
}
console.log(JSON.stringify({scenarios:fixture.scenarios.length,steps,seatViews:views,rejected,
  wasmSha256:fixture.wasmSha256,completeNativeStatesEqual:true,previous74Rejected:true}));
