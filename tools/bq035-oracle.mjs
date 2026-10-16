// Complete, source-bound Native78 inputs. Opaque state strings are never rewritten.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { gzipSync, gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const moduleUrl = import.meta.url;
const repository = fileURLToPath(new URL('../', moduleUrl));
export const bq035PositiveNames = [
  'accepted-blood-real-destroy', 'accepted-knight-real-bounce',
  'converted-jc095-keeps-mobility-separate-from-entry', 'duel-grant-can-use-nonadjacent-slot',
  'granted-glory-real-source-bounce', 'paid-attachments-mobility-glory-next-turn',
  'paid-loyalty-and-extra-payload-rejected', 'printed-glory-real-source-bounce',
  'printed-jc061-mobility-then-paid-entry', 'queued-blood-real-destroy', 'queued-knight-real-bounce',
  'wm023-paid-conversion-blood-knight-move', 'wm023-duel-paid-conversion-blood-knight-move',
].sort();
const numbered = (prefix, count) => Array.from({ length: count }, (_, i) => `${prefix}-${String(i).padStart(2, '0')}`);
export const bq035NegativeNames = [
  ...numbered('pending-observation', 23), ...numbered('pending-shape', 7),
  ...numbered('response-frame', 15), ...numbered('capture-record', 5),
  'direct-frame-and-stack', 'duplicate-stack-and-declaration',
  'impossible-demon-vampire-history', 'duplicate-live-recipient-uid',
  'wm023-own-action-missing-region-uid', 'wm023-own-action-mobility-target-transplant',
].sort();
export const bq035Identity = {
  rulesVersion: 'hegemony-pdf-v1', cardPoolVersion: 'limited-v2.71-bq035-vampire-mobility-candidate',
  engineVersion: 'rust-v0.2.78-bq035-vampire-mobility-candidate',
};
export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
export function assertSourceBindings(binding) {
  assert.equal(binding.algorithm, 'sha256');
  // These are frozen v60 Native78 observations, not fresh candidate79 tests.
  const saved = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi78/native-source-binding.json'), 'utf8'));
  const expected = Object.keys(saved.files).sort();
  assert.equal(expected.length, 121);
  assert.deepEqual(Object.keys(binding.files).sort(), expected, 'Incomplete Native source binding');
  for (const name of expected) assert.equal(binding.files[name], saved.files[name], `Stale Native source: ${name}`);
}
function decodeInputs(entries, names, kind, binding) {
  assert.deepEqual(entries.map(entry => entry.file).sort(), names.map(name => `${name}.json`), `Incomplete ${kind} input set`);
  return entries.map(entry => {
    assert.equal(Buffer.byteLength(entry.json), entry.bytes, entry.file);
    assert.equal(sha256(entry.json), entry.sha256, `Native input hash mismatch: ${entry.file}`);
    const row = JSON.parse(entry.json);
    assert.deepEqual(row.sourceBindings, binding, `Mixed compiler inputs: ${entry.file}`);
    if (kind === 'positive') assert.equal(row.name + '.json', entry.file);
    return { ...row, inputName: entry.file };
  });
}
export function validateBq035Bundle(bundle) {
  assert.equal(bundle.format, 'bq035-native78-raw-v1');
  assertSourceBindings(bundle.sourceBindings);
  const scenarios = decodeInputs(bundle.positive, bq035PositiveNames, 'positive', bundle.sourceBindings);
  const invalid = decodeInputs(bundle.negative, bq035NegativeNames, 'negative', bundle.sourceBindings);
  let commands = 0, projections = 0, rejected = 0;
  for (const scenario of scenarios) {
    assert.deepEqual(scenario.catalog, bundle.catalog);
    let state = scenario.initial.state;
    assert.equal(typeof state, 'string');
    assert(scenario.initial.views.length === 2 || scenario.initial.views.length === 4);
    projections += scenario.initial.views.length;
    assert(scenario.steps.length > 0);
    for (const step of scenario.steps) {
      assert.equal(step.state, state, `${scenario.name}: discontinuous opaque state`);
      assert.equal(typeof step.expected.state, 'string');
      assert.equal(step.serverNowMs, '0');
      assert.equal(step.views.length, scenario.initial.views.length);
      assert.deepEqual(step.expected.view, step.views[step.seat]);
      if (step.expected.errorCode) {
        assert.equal(step.expected.outcome, 'rejected');
        assert.equal(step.expected.state, state);
        rejected++;
      } else assert.equal(step.expected.outcome, 'accepted');
      state = step.expected.state;
      commands++; projections += step.views.length;
    }
  }
  for (const row of invalid) {
    assert.equal(typeof row.state, 'string');
    assert(row.gameError.length > 0 && row.roomError.length > 0, row.inputName);
  }
  assert.equal(rejected, 3);
  assert.equal(commands, 378, 'Incomplete continuous Native command set');
  assert.equal(projections, 1510, 'Incomplete every-seat Native projections');
  assert.deepEqual(bundle.counts, { scenarios: scenarios.length, commands, projections, rejected, invalid: invalid.length });
  for (const key of Object.keys(bq035Identity)) assert.equal(bundle.catalog[key], bq035Identity[key]);
  assert.equal(bundle.catalog.cards.length, 144);
  assert.match(bundle.wasmSha256, /^[a-f0-9]{64}$/);
  return { ...bundle, scenarios, invalid };
}
export function loadBq035Fixture(filename = path.join(repository, 'rust-game-wasm/tests/bq035Native78.fixture.json.gz')) {
  return validateBq035Bundle(JSON.parse(gunzipSync(readFileSync(filename), { maxOutputLength: 128 * 1024 * 1024 }).toString('utf8')));
}
export function compareBq035Fixture(fixture, abi, wasm) {
  assert.equal(sha256(wasm), fixture.wasmSha256, 'Native78 fixture WASM identity mismatch');
  assert.deepEqual(JSON.parse(abi.catalog()), fixture.catalog);
  let commands = 0, projections = 0, rejected = 0;
  const checkViews = (state, views) => views.forEach((expected, seat) => {
    assert.deepEqual(JSON.parse(abi.view(state, seat)), expected);
    projections++;
  });
  for (const scenario of fixture.scenarios) {
    let state = scenario.initial.state;
    checkViews(state, scenario.initial.views);
    for (const step of scenario.steps) {
      assert.equal(state, step.state);
      const result = JSON.parse(abi.applyRoom(state, step.seat, JSON.stringify(step.command), step.serverNowMs));
      assert.deepEqual(result, step.expected, `${scenario.name}: full transition at ${commands}`);
      assert.equal(result.state, step.expected.state);
      state = result.state;
      checkViews(state, step.views);
      commands++; if (result.errorCode) rejected++;
    }
  }
  for (const row of fixture.invalid) {
    for (const seat of [0, 1, 2, 3]) {
      assert.throws(() => abi.view(row.state, seat), error => String(error) === row.roomError, `${row.inputName}: exact restore error`);
      assert.throws(() => abi.applyRoom(row.state, seat, JSON.stringify({ commandId: 'invalid-bq035', expectedVersion: 0, action: { kind: 'game', action: { kind: 'pass' } } }), '0'), error => String(error) === row.roomError, `${row.inputName}: exact command restore error`);
    }
  }
  assert.equal(commands, fixture.counts.commands); assert.equal(projections, fixture.counts.projections); assert.equal(rejected, fixture.counts.rejected);
  return { ...fixture.counts, negativeRestoreCalls: fixture.invalid.length * 8, wasmSha256: fixture.wasmSha256, completeOpaqueStateAndTransitionMatch: true };
}
function readInputs(directory, names) {
  assert.deepEqual(readdirSync(directory).sort(), names.map(name => `${name}.json`));
  return names.map(name => {
    const file = name + '.json', bytes = readFileSync(path.join(directory, file));
    return { file, bytes: bytes.length, sha256: sha256(bytes), json: bytes.toString('utf8') };
  });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, positiveDir, negativeDir, output] = process.argv.slice(2);
  if (operation !== 'pack' || !output) throw Error('Usage: node tools/bq035-oracle.mjs pack <complete-positive-dir> <complete-negative-dir> <new-output.gz>');
  const positive = readInputs(positiveDir, bq035PositiveNames), negative = readInputs(negativeDir, bq035NegativeNames);
  const first = JSON.parse(positive[0].json), rows = positive.map(entry => JSON.parse(entry.json));
  const { kernel, moduleBytes } = await import('../rust-game-wasm/tests/current78-kernel.mjs');
  const bundle = { format: 'bq035-native78-raw-v1', sourceBindings: first.sourceBindings, catalog: first.catalog,
    wasmSha256: sha256(moduleBytes), counts: { scenarios: rows.length, commands: rows.reduce((n, row) => n + row.steps.length, 0),
      projections: rows.reduce((n, row) => n + row.initial.views.length + row.steps.reduce((m, step) => m + step.views.length, 0), 0),
      rejected: rows.reduce((n, row) => n + row.steps.filter(step => step.expected.errorCode).length, 0), invalid: negative.length }, positive, negative };
  const report = compareBq035Fixture(validateBq035Bundle(bundle), kernel, moduleBytes);
  const compressed = gzipSync(JSON.stringify(bundle), { level: 9 });
  writeFileSync(output, compressed, { flag: 'wx' });
  console.log(JSON.stringify({ ...report, fixtureBytes: compressed.length, fixtureSha256: sha256(compressed), sourceFiles: Object.keys(bundle.sourceBindings.files).length }));
}
