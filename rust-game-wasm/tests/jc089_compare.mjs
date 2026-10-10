// Explicit offline native fixtures and a continuous paid response chain.
// Persisted u64 state remains opaque; this does not claim natural browser play.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { kernel as current, catalog, moduleBytes as bytes } from './current-kernel.mjs';
assert(catalog.cards.some(card => card.id === 'JC089'));
const root = resolve(process.argv[2]);
let commands = 0, rejected = 0, checkpoints = 0, views = 0, chainCommands = 0;
for (const name of readdirSync(resolve(root, 'oracle')).sort()) {
  if (!name.endsWith('.json')) continue;
  const row = JSON.parse(readFileSync(resolve(root, 'oracle', name), 'utf8'));
  let state = row.state;
  if (row.command) {
    const actual = JSON.parse(current.applyRoom(state, row.seat, JSON.stringify(row.command), '0'));
    assert.deepEqual(actual, row.expected, name);
    if (actual.errorCode) { assert.equal(actual.state, state); rejected++; }
    state = actual.state; commands++;
  } else checkpoints++;
  for (let seat = 0; seat < 4; seat++) {
    assert.deepEqual(JSON.parse(current.view(state, seat)), row.views[seat], `${name} seat ${seat}`); views++;
  }
}
const chain = JSON.parse(readFileSync(resolve(root, 'chains', 'paid-response-chain.json'), 'utf8'));
let state = chain.initialState;
const kinds = {};
for (const step of chain.steps) {
  assert.equal(step.state, state);
  const actual = JSON.parse(current.applyRoom(state, step.seat, JSON.stringify(step.command), '0'));
  assert.deepEqual(actual, step.expected);
  state = actual.state;
  for (let seat = 0; seat < 4; seat++) { assert.deepEqual(JSON.parse(current.view(state, seat)), step.views[seat]); views++; }
  kinds[step.command.action.kind] = (kinds[step.command.action.kind] || 0) + 1; chainCommands++;
}
assert.equal(state, chain.finalState);
assert.equal(kinds.beginResponse, 1); assert.equal(kinds.submitResponse, 1);
// Historical identities are covered by Sites' recorded 40-tuple rejection gate.
console.log(JSON.stringify({ engineVersion: catalog.engineVersion, poolVersion: catalog.cardPoolVersion,
  commands, rejected, checkpoints, chainCommands, views, kinds,
  wasmBytes: bytes.length, wasmSha256: createHash('sha256').update(bytes).digest('hex') }, null, 2));
