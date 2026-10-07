import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import * as current from '../generated/hegemony_wasm.js';
import { routeKernels } from '../src/kernel-router.mjs';
import { frozenKernel } from './fixtures/frozen-kernel.mjs';

const currentBytes = readFileSync(new URL('../generated/hegemony_wasm_bg.wasm', import.meta.url));
current.initSync({ module: currentBytes });
const latest = JSON.parse(current.catalog());
const historicalRoot = existsSync('rust-game-wasm') ? 'rust-game-wasm' : '../rust-game-wasm';
const isUnsupported = error => error.status === 410 && error.code === 'unsupported_room_version' && /新建牌桌/.test(error.message);

test('current-only Site keeps the candidate engine45 bytes and its exact new-room flow', async () => {
  assert.equal(createHash('sha256').update(currentBytes).digest('hex'), 'ffbabae8bdb36352ae11acab613365ea3bdcdb0681d246a8d11b354864f01099');
  const prior = JSON.parse((await frozenKernel('legacy-v0.2.44')).catalog());
  assert.equal(latest.engineVersion, 'rust-v0.2.45-jc089-poison-blood-candidate');
  assert.equal(latest.cardPoolVersion, 'limited-v2.42-jc089-poison-blood-candidate');
  assert.equal(latest.cards.length, 103); assert.equal(latest.societies.length, 8);
  assert.deepEqual(latest.cards.filter(card => card.id !== 'JC089'), prior.cards);
  for (const field of ['societies', 'decks', 'world', 'deckBuildRules']) assert.deepEqual(latest[field], prior[field]);
  const routed = routeKernels(current);
  const room = JSON.parse(routed.newGame('new45', 'invite', 'duel', 'P0', 'watchers', '18446744073709551615'));
  assert.equal(room.view.versions.engine, latest.engineVersion);
  assert.equal(room.view.versions.cardPool, latest.cardPoolVersion);
  assert.equal(routed.catalog(room.state), current.catalog());
  assert.deepEqual(JSON.parse(routed.view(room.state, 0)), room.view);
  assert.equal(routed.joinGame(room.state, 'P1', 'hunters'), current.joinGame(room.state, 'P1', 'hunters'));
  const command = JSON.stringify({ commandId: 'ready-current', expectedVersion: 0, action: { kind: 'game', action: { kind: 'ready' } } });
  assert.equal(routed.applyRoom(room.state, 0, command, '1000'), current.applyRoom(room.state, 0, command, '1000'));
  assert.equal(routed.pollRoom(room.state, 0, '1000'), current.pollRoom(room.state, 0, '1000'));
  assert(room.state.includes('"seed":18446744073709551615'));
});

test('every historical tuple and altered identity rejects before any current state operation', async t => {
  const stateOperations = ['view', 'apply', 'joinGame', 'joinGameWithDeck', 'applyRoom', 'pollRoom', 'quoteRoom'];
  let interpreted = 0;
  const guarded = { ...current };
  for (const name of stateOperations) guarded[name] = () => { interpreted++; throw Error('A historical state must never reach the current reducer/projector'); };
  const routed = routeKernels(guarded);
  const folders = readdirSync(historicalRoot).filter(name => /^legacy-v0\.2\.\d+(?:-resource-policy)?$/.test(name));
  assert.equal(folders.length, 40);
  const reject = state => {
    for (const name of ['assertSupported', 'catalog', 'supportsPacing', ...stateOperations]) assert.throws(() => routed[name](state, 0, '{}', '1000'), isUnsupported);
  };
  for (const folder of folders) {
    const old = await frozenKernel(folder);
    const room = JSON.parse(old.newGame('old', 'invite', 'duel', 'P0', 'watchers', '18446744073709551615'));
    reject(room.state);
  }
  const room = JSON.parse(current.newGame('current', 'invite', 'duel', 'P0', 'watchers', '1'));
  for (const [before, after] of [[latest.rulesVersion, 'unknown-rules'], [latest.cardPoolVersion, 'unknown-pool'], [latest.engineVersion, 'unknown-engine'], ['"state_schema":3', '"state_schema":2'], ['"state_schema":3', '"state_schema":4']]) reject(room.state.replace(before, after));
  assert.equal(interpreted, 0);
  t.diagnostic(JSON.stringify({ historicalTuplesRejected: folders.length, alteredIdentityCases: 5, currentStateOperationsOnRejectedStates: interpreted }));
});

test('production generated modules and Worker contain only the reviewed current WASM', () => {
  assert.deepEqual(readdirSync('generated').sort(), ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']);
  const files = readdirSync('dist/server').filter(name => name.endsWith('.wasm'));
  assert.equal(files.length, 1);
  assert.deepEqual(readFileSync('dist/server/' + files[0]), currentBytes);
  assert(readFileSync('dist/server/index.js').length + currentBytes.length < 64 * 1024 * 1024);
});
