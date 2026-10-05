// Local actual-ABI identity check. No browser, network, credentials or state edits.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '../..');
const out = resolve(process.argv[2]);
mkdirSync(out, { recursive: true });
const variants = [];
for (const folder of ['legacy-v0.2.26-resource-policy', 'pkg']) {
  const path = resolve(root, 'rust-game-wasm', folder);
  const abi = await import(pathToFileURL(resolve(path, 'hegemony_wasm.js')));
  const bytes = readFileSync(resolve(path, 'hegemony_wasm_bg.wasm'));
  abi.initSync({ module: bytes });
  const catalog = JSON.parse(abi.catalog());
  let room = JSON.parse(abi.newGame(`identity-${folder}`, 'LOCAL', 'teams', 'P0', 'watchers', '9007199254740993'));
  for (let seat = 1; seat < 4; seat++) room = JSON.parse(abi.joinGame(room.state, `P${seat}`, 'watchers'));
  const state = room.state;
  const views = [0,1,2,3].map(seat => abi.view(state, seat));
  const restored = JSON.parse(abi.pollRoom(state, 0, '0'));
  assert.equal(restored.state, state);
  assert.deepEqual([0,1,2,3].map(seat => abi.view(restored.state, seat)), views);
  variants.push({ abi, catalog, state, views, wasmBytes: bytes.length, wasmSha256: createHash('sha256').update(bytes).digest('hex') });
  writeFileSync(resolve(out, `${folder}-identity-state.json`), state);
}
const [old, current] = variants;
assert.equal(old.catalog.engineVersion, 'rust-v0.2.26-msjc07-resource-policy-candidate');
assert.equal(old.wasmSha256, '79b0a9fa5e9f2acc015230ec3cd42e7f0fae41d8e8f720be7e6c81db52cd824e');
assert.equal(current.catalog.engineVersion, 'rust-v0.2.27-white-wound-society-candidate');
assert.equal(current.catalog.cardPoolVersion, 'limited-v2.24-white-wound-society-candidate');
assert.equal(current.catalog.cards.length, 88);
assert.deepEqual(current.catalog.societies.map(c => c.id), ['MSJC09','MSJC01','MSJC07','MSJC06']);
for (const prior of old.catalog.cards) assert.deepEqual(current.catalog.cards.find(c => c.id === prior.id), prior);
for (const prior of old.catalog.societies) assert.deepEqual(current.catalog.societies.find(c => c.id === prior.id), prior);
assert.deepEqual(current.catalog.decks, old.catalog.decks);
assert.deepEqual(current.catalog.deckBuildRules, old.catalog.deckBuildRules);
const whiteBefore = new Set(old.catalog.cards.filter(c => c.color === '白').map(c => c.name)).size;
const whiteAfter = new Set(current.catalog.cards.filter(c => c.color === '白').map(c => c.name)).size;
assert.equal(whiteBefore, 8); assert.equal(whiteAfter, 9);
assert.equal(current.catalog.cards.find(c => c.id === 'LC06').magic, '神圣');
for (const [reader, incompatible] of [[old, current], [current, old]]) {
  for (let seat = 0; seat < 4; seat++) assert.throws(() => reader.abi.view(incompatible.state, seat));
  assert.throws(() => reader.abi.apply(incompatible.state, 0, JSON.stringify({ kind: 'ready' })));
}
const proof = {
  ok: true, independentLocalFourSeatRoomsOnly: true, syntheticStateEdits: false,
  actualVersions: variants.map(v => ({ engine: v.catalog.engineVersion, cardPool: v.catalog.cardPoolVersion, wasmBytes: v.wasmBytes, wasmSha256: v.wasmSha256 })),
  ownOpaqueStateAndFourViewsExactAfterReadOnlyPoll: true,
  crossVersionFourViewsAndActionsRejectedBothWays: true,
  old87CompiledDefinitionsAndThreeSocietiesExact: true, oldFivePresetsAndConstructionRulesExact: true,
  whiteDistinctNamesBefore: whiteBefore, whiteDistinctNamesAfter: whiteAfter, maximumLegalWhiteCopiesAfter: whiteAfter * 3,
  currentPrintedHolyDomain: true, publicSiteTouched: false,
};
writeFileSync(resolve(out, 'actual-abi-source-and-identity-proof.json'), JSON.stringify(proof, null, 2) + '\n');
writeFileSync(resolve(out, 'current-catalog.json'), JSON.stringify(current.catalog, null, 2) + '\n');
console.log(JSON.stringify(proof));
