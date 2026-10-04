import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { createHash } from 'node:crypto';
const abi = new URL(process.env.HEGEMONY_WASM_TEST_ABI || '../pkg/hegemony_wasm.js', import.meta.url);
const { initSync, catalog, newGame, newGameWithDeck, joinGame, joinGameWithDeck, apply, applyRoom, pollRoom, quoteRoom, view } = await import(abi.href);

const moduleBytes = await readFile(new URL('./hegemony_wasm_bg.wasm', abi));
initSync({ module: moduleBytes });
const fixture = JSON.parse(await readFile(process.argv[2], 'utf8'));
assert.deepEqual(JSON.parse(catalog()), fixture.catalog);
let transitions = 0, projections = 0, quotes = 0, rejectedCommands = 0, rejectedDeckCreations = 0, slowestMs = 0;
const choiceKinds = new Set();
let cases = 0;
async function readScenario(entry) {
  if (!entry.fixtureFile) return entry;
  const bytes = await readFile(resolve(dirname(process.argv[2]), entry.fixtureFile));
  if (entry.bytes !== undefined) assert.equal(bytes.byteLength, entry.bytes);
  if (entry.sha256 !== undefined) assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256);
  return JSON.parse(bytes.toString('utf8'));
}
async function compareScenario(entry) {
  const scenario = await readScenario(entry);
  const initialTransitions = transitions, initialProjections = projections;
  if (entry.fixtureFile) {
    assert.equal(fixture.caseFormat, 'external-cases-v1');
    assert.equal(scenario.name, entry.name);
    assert.equal(scenario.seed, entry.seed);
  }
  for (const rejected of scenario.rejectedNewGameWithDeck || []) {
    assert.throws(() => newGameWithDeck(...rejected.args), error => String(error) === rejected.error);
    rejectedDeckCreations++;
  }
  let state;
  for (const step of scenario.steps) {
    const started = performance.now();
    let serialized;
    if (step.operation === 'initialFixture') { state = step.args[0]; serialized = JSON.stringify({ state, view: JSON.parse(view(state, step.seat)), version: step.version, seat: step.seat }); }
    else if (step.operation === 'newGame') serialized = newGame(...step.args);
    else if (step.operation === 'newGameWithDeck') serialized = newGameWithDeck(...step.args);
    else if (step.operation === 'joinGame') serialized = joinGame(state, ...step.args);
    else if (step.operation === 'joinGameWithDeck') serialized = joinGameWithDeck(state, ...step.args);
    else if (step.operation === 'applyRoom') serialized = applyRoom(state, step.args[0], JSON.stringify(step.args[1]), step.args[2]);
    else if (step.operation === 'pollRoom') serialized = pollRoom(state, ...step.args);
    else serialized = apply(state, step.args[0], JSON.stringify(step.args[1]));
    const result = JSON.parse(serialized); // Parse ONLY the outer envelope; .state stays an opaque string.
    assert.equal(typeof result.state, 'string');
    assert.equal(result.state, step.state ?? step.transition?.state, `${scenario.name}: state differs at version ${step.version}`);
    assert.equal(result.version, step.version);
    assert.equal(result.seat, step.seat);
    assert.deepEqual(result.view, step.view ?? step.transition?.view);
    if (step.transition) assert.deepEqual(result, step.transition, `${scenario.name}: full journal/outcome differs at revision ${step.version}`);
    if (step.transition?.outcome === 'rejected') rejectedCommands++;
    state = result.state;
    assert.ok(state.includes(`"seed":${scenario.seed}`), 'u64 decimal seed lost precision');
    for (let seat = 0; seat < step.views.length; seat++) {
      const projected = JSON.parse(view(state, seat));
      assert.deepEqual(projected, step.views[seat]);
      assert.equal(projected.you, `p${seat}`);
      if (projected.pendingChoice) choiceKinds.add(projected.pendingChoice.kind);
      for (const region of projected.regions) for (const card of region.characters) {
        if (card.faceDown && card.controller !== projected.you) {
          assert.equal(card.name, '暗藏者');
          for (const secret of ['cardId', 'text', 'cost', 'icons', 'defense', 'color', 'magic', 'currentSubtypes', 'currentRenown', 'currentBarrier', 'currentDamagePrevention', 'currentPrintedDefense']) assert.ok(!(secret in card));
        }
      }
      projections++;
    }
    transitions++;
    slowestMs = Math.max(slowestMs, performance.now() - started);
  }
  assert.throws(() => view(state, 99));
  assert.throws(() => apply(state, 99, '{"kind":"pass"}'));
  assert.throws(() => apply(state, 0, '{not JSON}'));
  for (const [seat, action] of scenario.rejectedActions ?? []) {
    assert.throws(() => apply(state, seat, JSON.stringify(action)));
  }
  for (const rejected of scenario.rejectedCommands ?? []) {
    assert.deepEqual(JSON.parse(applyRoom(state, rejected.seat, JSON.stringify(rejected.command), rejected.serverNow)), rejected.expected);
    rejectedCommands++;
  }
  for (const quoted of scenario.quotes ?? []) {
    assert.deepEqual(JSON.parse(quoteRoom(quoted.state, quoted.seat, JSON.stringify(quoted.request))), quoted.expected);
    quotes++;
  }
  if (entry.transitions !== undefined) assert.equal(transitions - initialTransitions, entry.transitions);
  if (entry.projections !== undefined) assert.equal(projections - initialProjections, entry.projections);
  cases++;
  console.log(JSON.stringify({ case: cases, name: scenario.name, transitions: transitions - initialTransitions,
    projections: projections - initialProjections, sha256: entry.sha256, ok: true }));
  // Only counters and names survive; no complete scenario is retained here.
}
for (const entry of fixture.cases) {
  await compareScenario(entry);
  globalThis.gc?.();
}
if (fixture.caseCount !== undefined) assert.equal(cases, fixture.caseCount);
if (fixture.transitions !== undefined) assert.equal(transitions, fixture.transitions);
if (fixture.projections !== undefined) assert.equal(projections, fixture.projections);
for (const previousState of fixture.rejectedStates ?? []) {
  assert.throws(() => view(previousState, 0));
  assert.throws(() => apply(previousState, 0, '{"kind":"pass"}'));
}
assert.throws(() => newGame('bad', 'invite', 'duel', 'P0', 'watchers', '18446744073709551616'));
assert.throws(() => newGame('bad', 'invite', 'duel', 'P0', 'watchers', '9007199254740993.0'));
console.log(JSON.stringify({ ok: true, cases, wasmBytes: moduleBytes.byteLength, transitions, projections, quotes, rejectedCommands, rejectedDeckCreations, choiceKinds: [...choiceKinds].sort(), slowestFixtureStepMs: Math.round(slowestMs * 100) / 100, opaqueState: true, maximumU64SeedExact: true, nativeWasmStateAndViewsMatch: true }));
