// Exact current-ABI comparison of original Native Room states and all seat views.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,catalog,moduleBytes} from './current-kernel.mjs';
import {currentIdentity,loadCurrentFixture,sha256} from '../../tools/current-native-source.mjs';
assert(process.argv[3]?.trim(), 'Usage: jc046_compare.mjs <repo-relative-fixture> <complete-55-invalid-directory> [current-manifest]');
assert(process.argv.length <= 5, 'Unexpected JC046 comparison arguments.');
const currentManifestPath=new URL('./current111/jc046-invalid.json.gz',import.meta.url);
const manifestBytes=readFileSync(process.argv[4] || currentManifestPath);
const manifestSha256=sha256(manifestBytes);
assert.equal(manifestSha256,sha256(readFileSync(currentManifestPath)),
  'JC046 invalid-state manifest must match the current source-bound identity.');
const manifest=loadCurrentFixture('rust-game-wasm/tests/current111/jc046-invalid.json.gz');
assert.equal(manifest.invalidRoomStates,55);
const invalidFiles=manifest.files;
assert.equal(invalidFiles.length,55,'The current JC046 manifest must identify all 55 invalid states.');
const expectedNames=invalidFiles.map(file=>file.name).sort();
assert.equal(new Set(expectedNames).size,55);
assert(expectedNames.every(name=>name.endsWith('.json')&&!name.includes('/')));
assert.deepEqual(readdirSync(process.argv[3]).sort(),expectedNames,
  'JC046 invalid-state directory must contain exactly the 55 current Native files; missing or extra entries are rejected.');
const invalidStatesInput=invalidFiles.map(file=>{
  const name=file.name;
  const bytes=readFileSync(`${process.argv[3]}/${name}`);
  assert.equal(bytes.length,file.bytes,`JC046 invalid-state byte length differs: ${name}`);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),file.sha256,
    `JC046 invalid-state SHA256 differs: ${name}`);
  const {state}=JSON.parse(bytes);
  assert.equal(typeof state,'string',name);
  const saved=JSON.parse(state);
  const versions={rules:currentIdentity.rulesVersion,cardPool:currentIdentity.cardPoolVersion,engine:currentIdentity.engineVersion};
  assert.deepEqual(saved.versions,versions,`${name}: rejection must exercise a current Room`);
  assert.deepEqual(saved.game.versions,versions,`${name}: rejection must exercise a current game`);
  return {name,state};
});
const fixture=loadCurrentFixture(process.argv[2] || 'rust-game-wasm/tests/current111/jc046.json.gz');
assert.deepEqual(fixture.catalog,catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.111-import20-rulings-complete');
assert.equal(catalog.cardPoolVersion,'limited-v2.83-batch20-next');
assert.equal(catalog.cards.length,193);
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
