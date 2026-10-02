import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { initSync, catalog, newGame, newGameWithDeck, joinGame, joinGameWithDeck, apply, view } from '../pkg/hegemony_wasm.js';

const moduleBytes = await readFile(new URL('../pkg/hegemony_wasm_bg.wasm', import.meta.url));
initSync({ module: moduleBytes });
const fixture = JSON.parse(await readFile(process.argv[2], 'utf8'));
assert.deepEqual(JSON.parse(catalog()), fixture.catalog);
let transitions = 0, projections = 0, slowestMs = 0;
const choiceKinds = new Set();
for (const scenario of fixture.cases) {
  let state;
  for (const step of scenario.steps) {
    const started = performance.now();
    let serialized;
    if (step.operation === 'initialFixture') { state = step.args[0]; serialized = JSON.stringify({ state, view: JSON.parse(view(state, step.seat)), version: step.version, seat: step.seat }); }
    else if (step.operation === 'newGame') serialized = newGame(...step.args);
    else if (step.operation === 'newGameWithDeck') serialized = newGameWithDeck(...step.args);
    else if (step.operation === 'joinGame') serialized = joinGame(state, ...step.args);
    else if (step.operation === 'joinGameWithDeck') serialized = joinGameWithDeck(state, ...step.args);
    else serialized = apply(state, step.args[0], JSON.stringify(step.args[1]));
    const result = JSON.parse(serialized); // Parse ONLY the outer envelope; .state stays an opaque string.
    assert.equal(typeof result.state, 'string');
    assert.equal(result.state, step.state, `${scenario.name}: state differs at version ${step.version}`);
    assert.equal(result.version, step.version);
    assert.equal(result.seat, step.seat);
    assert.deepEqual(result.view, step.view);
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
          for (const secret of ['cardId', 'text', 'cost', 'icons', 'defense', 'color', 'magic']) assert.ok(!(secret in card));
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
}
for (const previousState of fixture.rejectedStates ?? []) {
  assert.throws(() => view(previousState, 0));
  assert.throws(() => apply(previousState, 0, '{"kind":"pass"}'));
}
assert.throws(() => newGame('bad', 'invite', 'duel', 'P0', 'watchers', '18446744073709551616'));
assert.throws(() => newGame('bad', 'invite', 'duel', 'P0', 'watchers', '9007199254740993.0'));
console.log(JSON.stringify({ ok: true, wasmBytes: moduleBytes.byteLength, transitions, projections, choiceKinds: [...choiceKinds].sort(), slowestFixtureStepMs: Math.round(slowestMs * 100) / 100, opaqueState: true, maximumU64SeedExact: true, nativeWasmStateAndViewsMatch: true }));
