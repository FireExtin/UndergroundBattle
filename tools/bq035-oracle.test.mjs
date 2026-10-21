// Oracle completeness is part of the security boundary, not an optional test path.
import test from 'node:test';
import assert from 'node:assert/strict';
import { loadBq035Fixture, validateBq035Bundle, sha256 } from './bq035-oracle.mjs';
import { currentSourceManifestSha256 } from './current-native-source.mjs';
const fixture = loadBq035Fixture();
const editRaw = (entries, edit) => entries.map((entry, index) => {
  if (index) return entry;
  const value = JSON.parse(entry.json); edit(value);
  const json = JSON.stringify(value);
  return { ...entry, json, bytes: Buffer.byteLength(json), sha256: sha256(json) };
});
test('Native93 oracle contains all 13/378/1510 positive observations and all 56 exact-error negatives', () => {
  assert.deepEqual(fixture.counts, { scenarios: 13, commands: 378, projections: 1510, rejected: 3, invalid: 56 });
  assert(Object.keys(fixture.sourceBindings.files).length > 0);
  assert.equal(fixture.currentSourceManifestSha256,currentSourceManifestSha256);
});
test('missing or duplicate positive and negative files fail before comparison', () => {
  for (const key of ['positive', 'negative']) {
    assert.throws(() => validateBq035Bundle({ ...fixture, [key]: fixture[key].slice(1) }), /Incomplete/);
    assert.throws(() => validateBq035Bundle({ ...fixture, [key]: [fixture[key][1], ...fixture[key].slice(1)] }), /Incomplete/);
  }
});
test('changed raw bytes, altered source hashes and unlisted source paths fail closed', () => {
  assert.throws(() => validateBq035Bundle({ ...fixture, positive: [{ ...fixture.positive[0], json: fixture.positive[0].json + ' ' }, ...fixture.positive.slice(1)] }));
  assert.throws(() => validateBq035Bundle({ ...fixture, negative: [{ ...fixture.negative[0], sha256: '0'.repeat(64) }, ...fixture.negative.slice(1)] }), /hash mismatch/);
  assert.throws(() => validateBq035Bundle({ ...fixture, sourceBindings: { algorithm: 'sha256', files: { ...fixture.sourceBindings.files, 'rust-game/src/engine.rs': '0'.repeat(64) } } }), /differs from current source/);
  assert.throws(() => validateBq035Bundle({ ...fixture, sourceBindings: { algorithm: 'sha256', files: { ...fixture.sourceBindings.files, '../unlisted': '0'.repeat(64) } } }), /outside current source inventory/);
});
test('a different compiler binding inside a correctly hashed raw input still rejects', () => {
  const negative = editRaw(fixture.negative, row => { row.sourceBindings.files['rust-game/src/engine.rs'] = '0'.repeat(64); });
  assert.throws(() => validateBq035Bundle({ ...fixture, negative }), /Mixed compiler inputs/);
});
test('correctly hashed discontinuous or truncated command traces cannot become a passing subset', () => {
  const discontinuous = editRaw(fixture.positive, row => { row.steps[0].state += ' '; });
  assert.throws(() => validateBq035Bundle({ ...fixture, positive: discontinuous }), /discontinuous opaque state/);
  const truncated = editRaw(fixture.positive, row => { row.steps.pop(); });
  assert.throws(() => validateBq035Bundle({ ...fixture, positive: truncated }), /Incomplete continuous Native command set/);
});

test('a different complete source manifest or current WASM hash cannot qualify the archive',()=>{
 assert.throws(()=>validateBq035Bundle({...fixture,currentSourceManifestSha256:'0'.repeat(64)}));
 assert.throws(()=>validateBq035Bundle({...fixture,wasmSha256:'0'.repeat(64)}));
});
