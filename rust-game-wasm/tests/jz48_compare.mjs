// Explicit offline native fixtures and a continuous paid response chain.
// Persisted u64 state remains opaque; this does not claim natural browser play.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import * as current from '../pkg/hegemony_wasm.js';
import * as prior from '../legacy-v0.2.43/hegemony_wasm.js';
const bytes = readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm', import.meta.url));
current.initSync({ module: bytes });
const oldBytes = readFileSync(new URL('../legacy-v0.2.43/hegemony_wasm_bg.wasm', import.meta.url));
assert.equal(createHash('sha256').update(oldBytes).digest('hex'), '3b0bee7b1c6788bc0a299338322f1c07d6108161a75e4999ac066971659520bb');
prior.initSync({ module: oldBytes });
const catalog = JSON.parse(current.catalog()), oldCatalog = JSON.parse(prior.catalog());
assert.equal(catalog.engineVersion, 'rust-v0.2.44-jz48-criminal-condition-candidate');
assert.equal(catalog.cardPoolVersion, 'limited-v2.41-jz48-criminal-condition-candidate');
assert.equal(catalog.cards.length, 102);
assert.deepEqual(catalog.cards.filter(c => c.id !== 'JZ48'), oldCatalog.cards);
for (const field of ['world', 'decks', 'societies', 'deckBuildRules']) assert.deepEqual(catalog[field], oldCatalog[field]);
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
const oldRoom = JSON.parse(prior.newGame('JZ48-old', 'OLD', 'teams', 'P0', 'watchers', '9')).state;
assert.throws(() => current.view(oldRoom, 0), /version|版本|身份|规则/i);
assert.equal(JSON.parse(prior.view(oldRoom, 0)).versions.engine, oldCatalog.engineVersion);
console.log(JSON.stringify({ engineVersion: catalog.engineVersion, poolVersion: catalog.cardPoolVersion,
  commands, rejected, checkpoints, chainCommands, views, kinds,
  wasmBytes: bytes.length, wasmSha256: createHash('sha256').update(bytes).digest('hex'),
  frozen43Bytes: oldBytes.length, frozen43Sha256: createHash('sha256').update(oldBytes).digest('hex') }, null, 2));
