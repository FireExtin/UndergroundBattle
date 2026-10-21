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

test('current ABI93 keeps the source bytes, reviewed prior definitions and exact new-room flow', () => {
  const abiRoot = existsSync(new URL('../../rust-game-wasm/Cargo.toml', import.meta.url))
    ? new URL('../../rust-game-wasm/', import.meta.url)
    : new URL('../rust-game-wasm/', import.meta.url);
  const sourceBytes = readFileSync(new URL('pkg/hegemony_wasm_bg.wasm', abiRoot));
  assert.deepEqual(currentBytes, sourceBytes);
  // Recorded catalog is a definition regression oracle, not a runnable old core.
  const prior = JSON.parse(readFileSync(new URL('./fixtures/reviewed-catalog-v044.json', import.meta.url), 'utf8'));
  const priorIds = new Set(prior.cards.map(card => card.id));
  assert.deepEqual(latest.cards.filter(card => priorIds.has(card.id)), prior.cards.map(card => {
    // Corrections independently verified against the original printed cards.
    // The historical fixture stays unchanged; these are current-only exceptions.
    if (card.id === 'JC029') return { ...card,
      permanentIcons: { investigation: 0, combat: 0, influence: 0 },
      temporaryIcons: { investigation: 0, combat: 1, influence: 1 } };
    if (card.id === 'JZ49') return { ...card,
      permanentIcons: { investigation: 0, combat: 0, influence: 0 },
      temporaryIcons: { investigation: 0, combat: 2, influence: 0 } };
    if (card.id !== 'JC018') return card;
    const ruleTraits = { ...card.ruleTraits };
    delete ruleTraits.renown;
    return { ...card, ruleTraits };
  }));
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

test('production Worker contains exactly current93 and all fourteen retained77-89/91 WASM modules', () => {
  const retainedNames=[...Array.from({length:13},(_,i)=>'abi'+(77+i)), 'abi91'];
  assert.deepEqual(readdirSync('generated').sort(),[...retainedNames,'hegemony_wasm.js','hegemony_wasm_bg.wasm']);
  const originals=retainedNames.map(name=>{
    assert.deepEqual(readdirSync('generated/'+name).sort(),['hegemony_wasm.js','hegemony_wasm_bg.wasm']);
    return readFileSync('generated/'+name+'/hegemony_wasm_bg.wasm');
  });
  const emitted=readdirSync('dist/server').filter(name=>name.endsWith('.wasm')).map(name=>readFileSync('dist/server/'+name));
  assert.equal(emitted.length,15);
  for(const original of [currentBytes,...originals])assert.equal(emitted.filter(bytes=>bytes.equals(original)).length,1,'Each exact module is emitted once');
  assert(readFileSync('dist/server/index.js').length+emitted.reduce((n,b)=>n+b.length,0)<64*1024*1024);
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
