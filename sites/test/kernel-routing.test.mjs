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

test('current ABI84 keeps the source bytes, reviewed prior definitions and exact new-room flow', () => {
  const abiRoot = existsSync(new URL('../../rust-game-wasm/Cargo.toml', import.meta.url))
    ? new URL('../../rust-game-wasm/', import.meta.url)
    : new URL('../rust-game-wasm/', import.meta.url);
  const sourceBytes = readFileSync(new URL('pkg/hegemony_wasm_bg.wasm', abiRoot));
  assert.deepEqual(currentBytes, sourceBytes);
  // Recorded catalog is a definition regression oracle, not a runnable old core.
  const prior = JSON.parse(readFileSync(new URL('./fixtures/reviewed-catalog-v044.json', import.meta.url), 'utf8'));
  const priorIds = new Set(prior.cards.map(card => card.id));
  assert.deepEqual(latest.cards.filter(card => priorIds.has(card.id)), prior.cards.map(c=>{if(c.id!=='JC018')return c;const ruleTraits={...c.ruleTraits};delete ruleTraits.renown;return {...c,ruleTraits};}));
  assert.deepEqual(latest.societies.filter(s => !['MSJC04', 'MSJC05'].includes(s.id)), prior.societies);
  assert.deepEqual(latest.societies.filter(s => ['MSJC04', 'MSJC05'].includes(s.id)).map(s => s.id), ['MSJC04', 'MSJC05']);
  for (const field of ['decks', 'world', 'deckBuildRules']) assert.deepEqual(latest[field], prior[field]);
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

test('duplicate exact kernel identities reject instead of choosing an arbitrary reducer', () => {
  const room = JSON.parse(current.newGame('ambiguous', 'LOCAL', 'duel', 'P0', 'watchers', '1'));
  const ambiguous = routeKernels(current, [current]);
  for (const operation of ['assertSupported', 'catalog', 'supportsPacing', 'view', 'applyRoom']) {
    assert.throws(() => ambiguous[operation](room.state, 0, '{}', '0'), /Ambiguous frozen kernel identity/);
  }
});

test('production Worker contains exactly current84 and retained83/82/81/80/79/78/77 WASM', () => {
  assert.deepEqual(readdirSync('generated').sort(), ['abi77', 'abi78', 'abi79', 'abi80', 'abi81', 'abi82', 'abi83', 'hegemony_wasm.js', 'hegemony_wasm_bg.wasm']);
  assert.deepEqual(readdirSync('generated/abi77').sort(), ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']);
  const files = readdirSync('dist/server').filter(name => name.endsWith('.wasm'));
  const retained = readFileSync('generated/abi77/hegemony_wasm_bg.wasm');
  const retained78 = readFileSync('generated/abi78/hegemony_wasm_bg.wasm');
  const retained79 = readFileSync('generated/abi79/hegemony_wasm_bg.wasm');
  const retained80 = readFileSync('generated/abi80/hegemony_wasm_bg.wasm');
  const retained81 = readFileSync('generated/abi81/hegemony_wasm_bg.wasm');
  const retained82 = readFileSync('generated/abi82/hegemony_wasm_bg.wasm');
  const retained83 = readFileSync('generated/abi83/hegemony_wasm_bg.wasm');
  assert.equal(files.length, 8);
  const emitted = files.map(name => readFileSync('dist/server/' + name));
  assert.equal(emitted.filter(bytes => bytes.equals(currentBytes)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained78)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained79)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained80)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained81)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained82)).length, 1);
  assert.equal(emitted.filter(bytes => bytes.equals(retained83)).length, 1);
  assert(readFileSync('dist/server/index.js').length + currentBytes.length + retained.length + retained78.length + retained79.length + retained80.length + retained81.length + retained82.length + retained83.length < 64 * 1024 * 1024);
});

test('engine83 rejects preserved unsupported57/59/60/61/62/63/64/72/73 rooms and separate candidate headers before interpretation', () => {
  const original = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v057.json', import.meta.url), 'utf8')).accept.state;
  const prior59 = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v059.json', import.meta.url), 'utf8')).accept.state;
  const prior60 = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v060.json', import.meta.url), 'utf8')).accept.state;
  const prior61 = readFileSync(new URL('./fixtures/original-engine61-room.json', import.meta.url), 'utf8');
  const prior62 = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v062.json', import.meta.url), 'utf8')).accept.state;
  const prior63 = JSON.parse(readFileSync(new URL('./fixtures/xq37-native-v063.json', import.meta.url), 'utf8')).accept.state;
  const prior64 = JSON.parse(readFileSync(new URL('./fixtures/xq24-container-invalid-v064.json', import.meta.url), 'utf8')).cases[0].original;
  const prior72 = JSON.parse(readFileSync(new URL('../../web/src/game/jz35Native72.fixture.json', import.meta.url), 'utf8')).cases[0].before;
  const prior73 = JSON.parse(readFileSync(new URL('./fixtures/source-built-native73-lobby.json',import.meta.url),'utf8')).state;
  const withdrawn = JSON.stringify({ state_schema: 3, versions: {
    rules: 'hegemony-pdf-v1', cardPool: 'limited-v2.53-four-faction-engine-candidate', engine: 'rust-v0.2.58-four-faction-engine-candidate',
  } }); // Header-only routing case; not an invented playable historical room.
  const separateBQ030 = JSON.stringify({ state_schema: 3, versions: {
    rules: 'hegemony-pdf-v1', cardPool: 'limited-v2.56-bq030-attachment-candidate', engine: 'rust-v0.2.61-bq030-attachment-candidate',
  } }); // Exact standalone BQ030 tuple, used only to exercise routing.
  let interpreted = 0;
  const operations = ['view', 'apply', 'joinGame', 'joinGameWithDeck', 'applyRoom', 'pollRoom', 'quoteRoom'];
  const guarded = { ...current };
  for (const name of operations) guarded[name] = () => { interpreted++; throw Error('Historical room reached reducer'); };
  const routed = routeKernels(guarded);
  for (const state of [original, prior59, prior60, prior61, prior62, prior63, prior64, prior72, prior73, withdrawn, separateBQ030]) {
    for (const name of ['assertSupported', 'catalog', 'supportsPacing', ...operations]) assert.throws(() => routed[name](state, 0, '{}', '0'), isUnsupported);
  }
  assert.equal(interpreted, 0);
});
