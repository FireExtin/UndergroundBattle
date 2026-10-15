// Exact ABI77 comparison of original Native Room states and all seat views.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
const fixture=JSON.parse(readFileSync(process.argv[2] || new URL('../../web/src/game/jc046Native77.fixture.json',import.meta.url)));
assert.deepEqual(fixture.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.77-jc046-mandatory-sacrifice-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.70-jc046-mandatory-sacrifice-candidate');
assert.equal(catalog.cards.length,143);
assert.equal(fixture.scenarios.length,37);
assert.equal(new Set(fixture.scenarios.map(s=>s.name)).size,37);
assert.equal(createHash('sha256').update(moduleBytes).digest('hex'),fixture.wasmSha256);
const previousFixture=JSON.parse(readFileSync(new URL('../../web/src/game/jc095Native76.fixture.json',import.meta.url)));
assert.equal(previousFixture.catalog.engineVersion,'rust-v0.2.76-jc095-influence-conversion-candidate');
const previous=previousFixture.scenarios[0].steps[0];
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
if(process.argv[3]) {
  const names=readdirSync(process.argv[3]).filter(name=>name.endsWith('.json')).sort();
  assert(names.length>0,'The supplied JC046 invalid-state directory must contain actual Native exports.');
  const probe=fixture.scenarios[0].steps[0];
  assert(!probe.expected.errorCode,'The invalid-state probe must use an accepted Native command.');
  for(const name of names) {
    const {state}=JSON.parse(readFileSync(`${process.argv[3]}/${name}`));
    assert.equal(typeof state,'string',name);
    assert.throws(()=>kernel.view(state,0),undefined,name);
    assert.throws(()=>kernel.applyRoom(state,probe.seat,JSON.stringify(probe.command),probe.serverNowMs),undefined,name);
    invalidStates++;
  }
}
console.log(JSON.stringify({scenarios:fixture.scenarios.length,steps,seatViews:views,rejected,invalidStates,
  invalidStateEntrypoints:invalidStates*2,invalidStateDirectoryProvided:!!process.argv[3],
  wasmSha256:fixture.wasmSha256,completeNativeStatesEqual:true,previous76Rejected:true}));
