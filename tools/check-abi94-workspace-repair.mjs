// Fresh native Room traces, current guest, and unchanged opaque state strings.
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import path from 'node:path';
import {kernel} from '../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog, assertCurrentSourceBindings, currentWasmSha256,
  currentSourceManifestSha256, sha256, writeCurrentFixture} from './current-native-source.mjs';

const [directory, output] = process.argv.slice(2);
assert(directory, 'Pass the newly exported BQ035_NATIVE_DIR');
const expectedNames = ['abi94-repair-bq037-mobility', 'abi94-repair-cq07-mobility',
  'abi94-repair-xq46-supported-renown'];
const files = readdirSync(directory).filter(n => n.endsWith('.json')).sort();
assert.equal(files.length, 3);
const scenarios = [], inputs = [];
let commands = 0, views = 0, rejected = 0;
for (const file of files) {
  const bytes = readFileSync(path.join(directory, file));
  const scenario = JSON.parse(bytes);
  assertCurrentCatalog(scenario.catalog);
  assertCurrentSourceBindings(scenario.sourceBindings);
  let previous = scenario.initial.state;
  for (let seat = 0; seat < scenario.initial.views.length; seat++) {
    assert.deepEqual(JSON.parse(kernel.view(previous, seat)), scenario.initial.views[seat]);
    views++;
  }
  for (const step of scenario.steps) {
    assert.equal(step.state, previous);
    const actual = JSON.parse(kernel.applyRoom(step.state, step.seat,
      JSON.stringify(step.command), step.serverNowMs));
    assert.deepEqual(actual, step.expected, `${scenario.name}/${commands}`);
    assert.equal(actual.state, step.expected.state);
    assert(actual.state.includes('"seed":18446744073709551488'));
    for (let seat = 0; seat < step.views.length; seat++) {
      assert.deepEqual(JSON.parse(kernel.view(actual.state, seat)), step.views[seat]);
      views++;
    }
    if (actual.outcome === 'rejected') rejected++;
    previous = actual.state;
    commands++;
  }
  scenarios.push(scenario);
  inputs.push({file, bytes: bytes.length, sha256: sha256(bytes)});
}
assert.deepEqual(scenarios.map(s => s.name).sort(), expectedNames);
if (output) writeCurrentFixture(output, {format: 'abi94-workspace-repair-room-v1', scenarios});
console.log(JSON.stringify({ok: true, nativeScenarios: scenarios.length, commands, views,
  rejectedCommands: rejected, wasmSha256: currentWasmSha256, currentSourceManifestSha256,
  fullTransitionsEqual: true, opaqueStateUnchanged: true, inputs,
  scope: 'Fresh native prepared layouts and real Room commands, current WASM parity; not browser acceptance'}));
