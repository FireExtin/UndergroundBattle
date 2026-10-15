// Exact ABI77 comparison of original Native Room states and all seat views.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
assert(process.argv[3]?.trim(), 'Usage: jc046_compare.mjs <fixture> <complete-55-invalid-directory> [frozen-manifest]');
assert(process.argv.length <= 5, 'Unexpected JC046 comparison arguments.');
const manifestBytes=readFileSync(process.argv[4] || new URL('../../docs/evidence/jc046-2026-10-15/native-evidence-manifest.json',import.meta.url));
const manifestSha256=createHash('sha256').update(manifestBytes).digest('hex');
assert.equal(manifestSha256,'bedea2a79fc7cc98a6c1d0ed98a5d166396d638dba07060a91b6436ea5698661',
  'JC046 invalid-state manifest must match the frozen reviewed identity.');
const manifest=JSON.parse(manifestBytes);
assert.equal(manifest.invalidRoomStates,55);
const invalidPrefix='docs/evidence/jc046-2026-10-15/invalid-room/';
const invalidFiles=manifest.files.filter(file=>file.path.startsWith(invalidPrefix));
assert.equal(invalidFiles.length,55,'The frozen JC046 manifest must identify all 55 invalid states.');
const expectedNames=invalidFiles.map(file=>file.path.slice(invalidPrefix.length)).sort();
assert.equal(new Set(expectedNames).size,55);
assert(expectedNames.every(name=>name.endsWith('.json')&&!name.includes('/')));
assert.deepEqual(readdirSync(process.argv[3]).sort(),expectedNames,
  'JC046 invalid-state directory must contain exactly the 55 frozen files; missing or extra entries are rejected.');
const invalidStatesInput=invalidFiles.map(file=>{
  const name=file.path.slice(invalidPrefix.length);
  const bytes=readFileSync(`${process.argv[3]}/${name}`);
  assert.equal(bytes.length,file.bytes,`JC046 invalid-state byte length differs: ${name}`);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),file.sha256,
    `JC046 invalid-state SHA256 differs: ${name}`);
  const {state}=JSON.parse(bytes);
  assert.equal(typeof state,'string',name);
  return {name,state};
});
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
const probe=fixture.scenarios[0].steps[0];
assert(!probe.expected.errorCode,'The invalid-state probe must use an accepted Native command.');
for(const {name,state} of invalidStatesInput) {
  assert.throws(()=>kernel.view(state,0),undefined,name);
  assert.throws(()=>kernel.applyRoom(state,probe.seat,JSON.stringify(probe.command),probe.serverNowMs),undefined,name);
  invalidStates++;
}
assert.equal(invalidStates,55);
console.log(JSON.stringify({scenarios:fixture.scenarios.length,steps,seatViews:views,rejected,invalidStates,
  invalidStateEntrypoints:invalidStates*2,invalidStateDirectoryProvided:true,invalidManifestSha256:manifestSha256,
  wasmSha256:fixture.wasmSha256,completeNativeStatesEqual:true,previous76Rejected:true}));
