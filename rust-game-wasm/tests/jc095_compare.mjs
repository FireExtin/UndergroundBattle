// Full current ABI76 comparison against independently asserted Native traces.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
const fixture=JSON.parse(readFileSync(process.argv[2] || new URL('../../web/src/game/jc095Native76.fixture.json',import.meta.url)));
assert.deepEqual(fixture.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.76-jc095-influence-conversion-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.69-jc095-influence-conversion-candidate');
assert.equal(createHash('sha256').update(moduleBytes).digest('hex'),fixture.wasmSha256);
const previous=JSON.parse(readFileSync(new URL('../../web/src/game/jc060Native75.fixture.json',import.meta.url))).scenarios[0].steps[0];
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
let invalidStates=0;
if(process.argv[3]) for(const name of readdirSync(process.argv[3]).filter(n=>n.endsWith('.json'))) {
  const {state}=JSON.parse(readFileSync(`${process.argv[3]}/${name}`));
  assert.throws(()=>kernel.view(state,0),undefined,name);
  const probe=fixture.scenarios[0].steps[0];
  assert.throws(()=>kernel.applyRoom(state,probe.seat,JSON.stringify(probe.command),probe.serverNowMs),undefined,name);
  invalidStates++;
}
console.log(JSON.stringify({scenarios:fixture.scenarios.length,steps,seatViews:views,rejected,invalidStates,
  wasmSha256:fixture.wasmSha256,completeNativeStatesEqual:true,previous75Rejected:true}));
