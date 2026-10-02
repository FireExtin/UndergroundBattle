import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import * as current from '../generated/hegemony_wasm.js';
import * as previous from '../generated/legacy-v0.2.1/hegemony_wasm.js';
import { routeKernels } from '../src/kernel-router.mjs';

current.initSync({ module: readFileSync(new URL('../generated/hegemony_wasm_bg.wasm', import.meta.url)) });
previous.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.1/hegemony_wasm_bg.wasm', import.meta.url)) });
test('real kernels preserve their full version tuple and reject unknown persisted identities', () => {
  const routed = routeKernels(current, previous);
  assert.equal(JSON.parse(routed.catalog()).engineVersion, 'rust-v0.2.2');
  for (const kernel of [current, previous]) {
    const initial = JSON.parse(kernel.newGame('room', 'invite', 'duel', 'P0', 'watchers', '18446744073709551615'));
    assert.deepEqual(JSON.parse(routed.view(initial.state, 0)), initial.view);
    assert.equal(routed.joinGame(initial.state, 'P1', 'hunters'), kernel.joinGame(initial.state, 'P1', 'hunters'));
    assert.equal(routed.apply(initial.state, 0, '{"kind":"ready"}'), kernel.apply(initial.state, 0, '{"kind":"ready"}'));
    const identity = JSON.parse(current.stateIdentity(initial.state));
    assert.deepEqual(Object.keys(identity).sort(), ['state_schema', 'versions']);
    assert.deepEqual(identity.versions, initial.view.versions);
    assert.throws(() => routed.view(initial.state.replace(identity.versions.engine, 'rust-v99'), 0));
    assert.throws(() => routed.view(initial.state.replace(identity.versions.cardPool, 'unsupported-pool'), 0));
    assert.throws(() => routed.view(initial.state.replace(identity.versions.rules, 'unsupported-rules'), 0));
    assert.throws(() => routed.view(initial.state.replace('"state_schema":2', '"state_schema":1'), 0));
  }
  assert.equal(JSON.parse(routed.newGame('new', 'invite', 'duel', 'P0', 'watchers', '1')).view.versions.engine, 'rust-v0.2.2');
});
