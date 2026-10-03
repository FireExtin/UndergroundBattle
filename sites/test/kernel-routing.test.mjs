import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import * as current from '../generated/hegemony_wasm.js';
import * as previous from '../generated/legacy-v0.2.1/hegemony_wasm.js';
import * as intermediate from '../generated/legacy-v0.2.2/hegemony_wasm.js';
import * as last from '../generated/legacy-v0.2.3/hegemony_wasm.js';
import * as stable from '../generated/legacy-v0.2.4/hegemony_wasm.js';
import * as paced from '../generated/legacy-v0.2.5/hegemony_wasm.js';
import * as attached from '../generated/legacy-v0.2.6/hegemony_wasm.js';
import * as grave from '../generated/legacy-v0.2.7/hegemony_wasm.js';
import * as playable from '../generated/legacy-v0.2.8/hegemony_wasm.js';
import { routeKernels } from '../src/kernel-router.mjs';

current.initSync({ module: readFileSync(new URL('../generated/hegemony_wasm_bg.wasm', import.meta.url)) });
previous.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.1/hegemony_wasm_bg.wasm', import.meta.url)) });
intermediate.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.2/hegemony_wasm_bg.wasm', import.meta.url)) });
last.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.3/hegemony_wasm_bg.wasm', import.meta.url)) });
stable.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.4/hegemony_wasm_bg.wasm', import.meta.url)) });
paced.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.5/hegemony_wasm_bg.wasm', import.meta.url)) });
attached.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.6/hegemony_wasm_bg.wasm', import.meta.url)) });
grave.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.7/hegemony_wasm_bg.wasm', import.meta.url)) });
playable.initSync({ module: readFileSync(new URL('../generated/legacy-v0.2.8/hegemony_wasm_bg.wasm', import.meta.url)) });
test('real kernels preserve their full version tuple and reject unknown persisted identities', () => {
  const routed = routeKernels(current, [previous, intermediate, last, stable, paced, attached, grave, playable]);
  const candidateVersion = JSON.parse(current.catalog()).engineVersion;
  assert.equal(candidateVersion, 'rust-v0.2.8-msjc09-candidate');
  assert.equal(JSON.parse(playable.catalog()).engineVersion, 'rust-v0.2.8');
  for (const kernel of [current, previous, intermediate, last, stable, paced, attached, grave, playable]) {
    const initial = JSON.parse(kernel.newGame('room', 'invite', 'duel', 'P0', 'watchers', '18446744073709551615'));
    assert.deepEqual(JSON.parse(routed.view(initial.state, 0)), initial.view);
    assert.equal(routed.catalog(initial.state), kernel.catalog());
    assert.equal(routed.joinGame(initial.state, 'P1', 'hunters'), kernel.joinGame(initial.state, 'P1', 'hunters'));
    assert.equal(routed.supportsPacing(initial.state), !!kernel.pollRoom);
    if (kernel.pollRoom) {
      const raw = JSON.stringify({ commandId: 'original-ready', expectedVersion: 0, action: { kind: 'game', action: { kind: 'ready' } } });
      assert.equal(routed.applyRoom(initial.state, 0, raw, '1000'), kernel.applyRoom(initial.state, 0, raw, '1000'));
      assert.throws(() => routed.apply(initial.state, 0, '{"kind":"ready"}'));
    } else {
      assert.equal(routed.apply(initial.state, 0, '{"kind":"ready"}'), kernel.apply(initial.state, 0, '{"kind":"ready"}'));
      assert.throws(() => routed.pollRoom(initial.state, 0, '1000'), /旧牌桌/);
    }
    const identity = JSON.parse(current.stateIdentity(initial.state));
    assert.deepEqual(Object.keys(identity).sort(), ['state_schema', 'versions']);
    assert.deepEqual(identity.versions, initial.view.versions);
    assert.throws(() => routed.view(initial.state.replace(identity.versions.engine, 'rust-v99'), 0));
    assert.throws(() => routed.view(initial.state.replace(identity.versions.cardPool, 'unsupported-pool'), 0));
    assert.throws(() => routed.view(initial.state.replace(identity.versions.rules, 'unsupported-rules'), 0));
    assert.throws(() => routed.view(initial.state.replace(`"state_schema":${identity.state_schema}`, '"state_schema":1'), 0));
    if (!kernel.joinGameWithDeck) {
      assert.throws(() => routed.joinGameWithDeck(initial.state, 'P1', '{}'), /旧牌桌/);
      assert.throws(() => routed.apply(initial.state, 0, '{"kind":"deck","deckDraft":{}}'), /旧牌桌/);
    } else {
      const c = JSON.parse(kernel.catalog());
      const draft = { id: 'old-or-new-custom', name: '冻结牌组', description: '', societyId: null,
        cards: c.decks.find(d => d.id === 'watchers').cards, rulesVersion: c.rulesVersion,
        cardPoolVersion: c.cardPoolVersion, engineVersion: c.engineVersion, updatedAt: '2026-10-02T00:00:00Z' };
      assert.equal(routed.joinGameWithDeck(initial.state, 'P1', JSON.stringify(draft)), kernel.joinGameWithDeck(initial.state, 'P1', JSON.stringify(draft)));
    }
  }
  assert.equal(JSON.parse(routed.newGame('new', 'invite', 'duel', 'P0', 'watchers', '1')).view.versions.engine, candidateVersion);
});
