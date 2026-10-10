import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import * as current from '../generated/hegemony_wasm.js';
import { routeKernels } from '../src/kernel-router.mjs';
import { historicalIdentities } from './fixtures/historical-rooms.mjs';

const currentBytes = readFileSync(new URL('../generated/hegemony_wasm_bg.wasm', import.meta.url));
current.initSync({ module: currentBytes });
const latest = JSON.parse(current.catalog());
const isUnsupported = error => error.status === 410 && error.code === 'unsupported_room_version' && /新建牌桌/.test(error.message);

test('current-only Site keeps the source ABI bytes, reviewed prior definitions and exact new-room flow', () => {
  const abiRoot = existsSync(new URL('../../rust-game-wasm/Cargo.toml', import.meta.url))
    ? new URL('../../rust-game-wasm/', import.meta.url)
    : new URL('../rust-game-wasm/', import.meta.url);
  const sourceBytes = readFileSync(new URL('pkg/hegemony_wasm_bg.wasm', abiRoot));
  assert.deepEqual(currentBytes, sourceBytes);
  // Recorded catalog is a definition regression oracle, not a runnable old core.
  const prior = JSON.parse(readFileSync(new URL('./fixtures/reviewed-catalog-v044.json', import.meta.url), 'utf8'));
  const priorIds = new Set(prior.cards.map(card => card.id));
  assert.deepEqual(latest.cards.filter(card => priorIds.has(card.id)), prior.cards.map(c=>{if(c.id!=='JC018')return c;const ruleTraits={...c.ruleTraits};delete ruleTraits.renown;return {...c,ruleTraits};}));
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
  assert.equal(historicalIdentities.length, 40);
  const reject = state => {
    for (const name of ['assertSupported', 'catalog', 'supportsPacing', ...stateOperations]) assert.throws(() => routed[name](state, 0, '{}', '1000'), isUnsupported);
  };
  for (const { identity } of historicalIdentities) {
    // The router reads only this envelope. No legacy game is reduced/projected.
    reject(JSON.stringify(identity));
  }
  const room = JSON.parse(current.newGame('current', 'invite', 'duel', 'P0', 'watchers', '1'));
  for (const [before, after] of [[latest.rulesVersion, 'unknown-rules'], [latest.cardPoolVersion, 'unknown-pool'], [latest.engineVersion, 'unknown-engine'], ['"state_schema":3', '"state_schema":2'], ['"state_schema":3', '"state_schema":4']]) reject(room.state.replace(before, after));
  assert.equal(interpreted, 0);
  t.diagnostic(JSON.stringify({ historicalTuplesRejected: historicalIdentities.length, alteredIdentityCases: 5, currentStateOperationsOnRejectedStates: interpreted }));
});

test('production generated modules and Worker contain only the reviewed current WASM', () => {
  assert.deepEqual(readdirSync('generated').sort(), ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']);
  const files = readdirSync('dist/server').filter(name => name.endsWith('.wasm'));
  assert.equal(files.length, 1);
  assert.deepEqual(readFileSync('dist/server/' + files[0]), currentBytes);
  assert(readFileSync('dist/server/index.js').length + currentBytes.length < 64 * 1024 * 1024);
});

test('engine60 rejects the preserved actual57 room and the withdrawn58 header before interpretation', () => {
  const original = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v057.json', import.meta.url), 'utf8')).accept.state;
  const prior59 = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v059.json', import.meta.url), 'utf8')).accept.state;
  const withdrawn = JSON.stringify({ state_schema: 3, versions: {
    rules: 'hegemony-pdf-v1', cardPool: 'limited-v2.53-four-faction-engine-candidate', engine: 'rust-v0.2.58-four-faction-engine-candidate',
  } }); // Header-only routing case; not an invented playable historical room.
  let interpreted = 0;
  const operations = ['view', 'apply', 'joinGame', 'joinGameWithDeck', 'applyRoom', 'pollRoom', 'quoteRoom'];
  const guarded = { ...current };
  for (const name of operations) guarded[name] = () => { interpreted++; throw Error('Historical room reached reducer'); };
  const routed = routeKernels(guarded);
  for (const state of [original, prior59, withdrawn]) {
    for (const name of ['assertSupported', 'catalog', 'supportsPacing', ...operations]) assert.throws(() => routed[name](state, 0, '{}', '0'), isUnsupported);
  }
  assert.equal(interpreted, 0);
});
