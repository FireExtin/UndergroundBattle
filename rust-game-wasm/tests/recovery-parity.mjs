// Run after the native recovery tests export HEGEMONY_RECOVERY_FIXTURE_DIR.
// The outer records are JSON; their private state strings stay opaque.
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { gunzipSync } from 'node:zlib';
import path from 'node:path';
import { kernel, moduleBytes } from './current-kernel.mjs';

const directory = process.argv[2];
assert.ok(directory, 'Usage: node recovery-parity.mjs NATIVE_FIXTURE_DIRECTORY');
const files = readdirSync(directory).filter(name => /^case-\d+\.json(?:\.gz)?$/.test(name)).sort();
assert.equal(files.length, 4, 'Expected this directed group, not an unrelated fixture directory');
const cases = [];
let firstState;
for (const file of files) {
  const input = readFileSync(path.join(directory, file));
  const bytes = file.endsWith('.gz') ? gunzipSync(input) : input;
  const record = JSON.parse(bytes);
  firstState ??= record.validRoomState;
  assert.equal(record.validViews.length, 4);
  for (let seat = 0; seat < 4; seat++) {
    assert.deepEqual(JSON.parse(kernel.view(record.validRoomState, seat)), record.validViews[seat], `${file}: native seat ${seat}`);
  }
  if (record.shouldReject) {
    assert.throws(() => kernel.view(record.alteredRoomState, 0), error => String(error) === record.nativeError, `${file}: same native rejection`);
  } else {
    for (let seat = 0; seat < 4; seat++) {
      assert.deepEqual(JSON.parse(kernel.view(record.alteredRoomState, seat)), record.validViews[seat], `${file}: honest target loss seat ${seat}`);
    }
    assert.ok(record.alteredRoomState.includes('"seed":18446744073709551488'), 'Native large u64 seed remains in the untouched opaque string');
  }
  cases.push({ file, name: record.name, shouldReject: record.shouldReject, sha256: createHash('sha256').update(bytes).digest('hex'), inputSha256: createHash('sha256').update(input).digest('hex') });
}
assert.equal(cases.filter(item => item.shouldReject).length, 2);
console.log(JSON.stringify({
  scope: 'WASM Room decoder: four native fixtures, 16 valid seat views, two same-error rejections, two honest target-loss snapshots accepted in all four seats. Native tests separately exercise Game and Room entrances. No browser, D1 or full-match claim.',
  stateIdentity: JSON.parse(kernel.stateIdentity(firstState)),
  wasmSha256: createHash('sha256').update(moduleBytes).digest('hex'),
  fixtureCount: cases.length,
  validSeatViews: 16,
  malformedRoomRejections: 2,
  honestTargetLossSnapshots: 2,
  cases,
}, null, 2));
