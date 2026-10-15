// Compare source-produced Native states, complete commands and all seat views.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
const versions=new Map([
  ['rust-v0.2.74-jc037-hidden-investigation-candidate',['74','limited-v2.67-jc037-hidden-investigation-candidate']],
  ['rust-v0.2.75-jc060-occult-adviser-candidate',['75','limited-v2.68-jc060-occult-adviser-candidate']],
  ['rust-v0.2.76-jc095-influence-conversion-candidate',['76','limited-v2.69-jc095-influence-conversion-candidate']],
  ['rust-v0.2.77-jc046-mandatory-sacrifice-candidate',['77','limited-v2.70-jc046-mandatory-sacrifice-candidate']],
]);
assert(versions.has(catalog.engineVersion),'Build an explicitly supported current ABI first.');
const [suffix,expectedPool]=versions.get(catalog.engineVersion);
assert.equal(catalog.cardPoolVersion,expectedPool);
const fixture=JSON.parse(readFileSync(process.argv[2] || new URL(`../../web/src/game/jc037Native${suffix}.fixture.json`,import.meta.url)));
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
