import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest, RoomService } from '../src/service.mjs';
import { initSync, newGame } from '../generated/hegemony_wasm.js';
import { historicalLobbies } from './fixtures/historical-rooms.mjs';

test('real workerd/WASM with D1 SQLite: concurrency, receipts, rollback and reopen', async t => {
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-worker-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-integration', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-integration-db' } });
  let mf = new Miniflare(options);
  t.after(() => mf.dispose());
  let db = await mf.getD1Database('DB');
  const store = new RoomStore(db);
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const pacedRooms = new Set();
  const pacedVersions = new Set(['rust-v0.2.63-bq030-fixed-slots-candidate']);
  const durableView = value => pacedVersions.has(value?.versions?.engine) ? { ...value, serverNowMs: undefined } : value;
  const api = async (path, body, session) => {
    if (body?.action && pacedRooms.has(session?.roomId) && !['game','beginResponse','passResponse','cancelAndPass','submitResponse'].includes(body.action.kind)) {
      body = { ...body, action: { kind: 'game', action: body.action } };
    }
    const response = await mf.dispatchFetch('http://localhost' + path, {
      method: body ? 'POST' : 'GET', headers: { ...(body ? { 'Content-Type': 'application/json' } : {}),
        ...(session ? { Authorization: 'Bearer ' + session.token } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    const result = { status: response.status, body: response.status === 204 ? null : await response.json() };
    if (pacedVersions.has(result.body?.view?.versions?.engine)) pacedRooms.add(result.body.roomId);
    return result;
  };
  const counts = async id => {
    const rows = await db.batch(['seats', 'commands', 'journal'].map(table => db.prepare(`SELECT count(*) AS n FROM ${table} WHERE room_id=?`).bind(id)));
    return rows.map(r => r.results[0].n);
  };
  const key = () => crypto.randomUUID();
  const path = (session, action) => `/api/rooms/${session.roomId}/${action}`;
  try {
    const health = await api('/api/health'); assert.equal(health.body.transport, 'polling');
    const catalog = await api('/api/catalog'); assert.equal(catalog.body.cards.length, 129); for (const id of ['JC075','JC104','JZ31','JZ55','JZ49','JZ48','JC089','XQ40','XQ41','XQ45','JZ50','BQ104','XQ48','XQ37','JZ30','BQ028','BQ040','WM059','BQ078','JZ44','JZ45']) assert(catalog.body.cards.some(c => c.id === id)); assert.equal(catalog.body.entryIdempotency, true); assert(catalog.body.cards.some(c => c.id === 'XQ18'));
    assert(catalog.body.cards.some(card => card.id === 'JC047'));
    assert(catalog.body.cards.some(card => card.id === 'JC007'));
    assert.equal(catalog.body.engineVersion, 'rust-v0.2.63-bq030-fixed-slots-candidate');
    assert.equal(catalog.body.cardPoolVersion, 'limited-v2.56-bq030-attachment-candidate');
    assert.equal(catalog.body.deckBuildRules.minimumCards, 50);
    assert.equal(catalog.body.cards.find(card => card.id === 'JC125').deckCopyLimit, null);
    const create = { name: '甲', mode: 'teams', deckId: 'responders', requestId: key() };
    const hosts = await Promise.all([api('/api/rooms', create), api('/api/rooms', create)]);
    assert.equal(hosts[0].status, 200); assert.equal(hosts[1].status, 200); assert.deepEqual(hosts[0].body, hosts[1].body);
    const host = hosts[0].body;
    assert.equal((await api(path(host, 'catalog'))).status, 401);
    assert.deepEqual((await api(path(host, 'catalog'), null, host)).body, catalog.body);
    const currentKeepers = catalog.body.decks.find(deck => deck.id === 'keepers');
    assert.equal(currentKeepers.cards.find(entry => entry.cardId === 'JC058').count, 3);
    assert.equal(currentKeepers.cards.find(entry => entry.cardId === 'JC125').count, 14);
    assert.equal((await api('/api/rooms', { ...create, name: '异意图' })).status, 409);
    const joins = [1, 2, 3].map(i => ({ inviteCode: host.inviteCode, name: String(i), deckId: 'responders', requestId: key() }));
    const joined = await Promise.all(joins.map(body => api('/api/rooms/join', body)));
    assert(joined.every(r => r.status === 200));
    const players = [host, ...joined.map(r => r.body)];
    assert.equal(new Set(players.map(s => s.seat)).size, 4);
    assert.deepEqual(await counts(host.roomId), [4, 0, 3]);
    const retryJoin = await api('/api/rooms/join', joins[0]); assert.deepEqual(retryJoin.body, joined[0].body);
    assert.equal((await api('/api/rooms/join', { ...joins[0], name: '不同' })).status, 409);
    assert.deepEqual(await counts(host.roomId), [4, 0, 3]);
    const unauthenticated = await api(path(host, 'state'));
    assert.equal(unauthenticated.status, 401); assert.equal(unauthenticated.body.error, 'invalid_seat_token');
    assert.equal((await api(path(host, 'state') + '?afterVersion=3', null, host)).status, 204);
    const original = { commandId: key(), expectedVersion: 3, action: { kind: 'ready' } };
    const identical = await Promise.all([api(path(host, 'commands'), original, host), api(path(host, 'commands'), original, host)]);
    assert(identical.every(r => r.status === 200)); assert.deepEqual(identical[0].body, identical[1].body);
    assert.deepEqual(await counts(host.roomId), [4, 1, 4]);
    assert.equal((await api(path(host, 'commands'), original)).status, 401);
    assert.equal((await api(path(host, 'commands'), { ...original, action: { kind: 'pass' } }, host)).status, 409);
    assert.equal((await api(path(host, 'commands'), original, players[1])).status, 409);
    const concurrent = await Promise.all(players.slice(1, 3).map(s => api(path(s, 'commands'),
      { commandId: key(), expectedVersion: 4, action: { kind: 'ready' } }, s)));
    assert.deepEqual(concurrent.map(r => r.status).sort(), [200, 409]);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    const duelBody = { name: '末席测试', mode: 'duel', deckId: 'watchers', requestId: key() };
    const duel = (await api('/api/rooms', duelBody)).body;
    const lastSeatBody = { inviteCode: duel.inviteCode, name: '同一末席', deckId: 'responders', requestId: key() };
    const lastSeat = await Promise.all([api('/api/rooms/join', lastSeatBody), api('/api/rooms/join', lastSeatBody)]);
    assert(lastSeat.every(r => r.status === 200)); assert.deepEqual(lastSeat[0].body, lastSeat[1].body);
    assert.deepEqual(await counts(duel.roomId), [2, 0, 1]);
    // Test-only fresh lobby fixture; never mutate a live game's intermediate state.
    // The identical Rust WASM creates the maximum-u64 seed, kept opaque through D1 and Worker.
    initSync({ module: readFileSync((existsSync('rust-game-wasm') ? '' : '../') + 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm') });
    const preciseId = 'f'.repeat(24), preciseToken = 'a'.repeat(64);
    const precise = JSON.parse(newGame(preciseId, 'U64MAXTEST', 'duel', '精度测试', 'watchers', '18446744073709551615'));
    assert(precise.state.includes('"seed":18446744073709551615')); assert(precise.state.includes('"random":18446744073709551615'));
    await store.create({ id: preciseId, invite: 'U64MAXTEST', state: precise.state, nonce: key(), tokenHash: await digest(preciseToken),
      requestHash: await digest(key()), intentHash: 'fixture', response: '{}' });
    const preciseSession = { roomId: preciseId, token: preciseToken }; pacedRooms.add(preciseId);
    const preciseReady = await api(path(preciseSession, 'commands'), { commandId: key(), expectedVersion: 0, action: { kind: 'ready' } }, preciseSession);
    assert.equal(preciseReady.status, 200);
    const opaqueAfter = (await store.room(preciseId)).state;
    assert(opaqueAfter.includes('"seed":18446744073709551615')); assert(opaqueAfter.includes('"random":18446744073709551615'));
    // Local-only historical fixtures are retained byte-for-byte and fail closed.
    const oldRooms = [];
    for (const { minor, room: old } of historicalLobbies()) {
      const id = String(minor).padStart(2, '0').repeat(12), token = String(minor).padStart(2, '0').repeat(32), invite = 'OLDTEST' + minor;
      const createBody = { name: '旧核', mode: 'duel', deckId: 'watchers', requestId: key() };
      const receiptKeys = new RoomService(db, {});
      const entryKey = await receiptKeys.entryKey(createBody, { kind: 'create', name: '旧核', mode: 'duel', deckId: 'watchers' });
      const oldResponse = JSON.stringify({ roomId: id, inviteCode: invite, token, seat: 0, view: old.view });
      await store.create({ id, invite, state: old.state, nonce: key(), tokenHash: await digest(token),
        ...entryKey, response: oldResponse });
      const actor = { roomId: id, token };
      const duplicate = { commandId: key(), expectedVersion: 0, action: { kind: 'game', action: { kind: 'ready' } } };
      const commandHash = await digest(JSON.stringify({ action: { action: { kind: 'ready' }, kind: 'game' }, expectedVersion: 0, seat: 0 }));
      await db.prepare('INSERT INTO commands(room_id,seat,command_id,intent_hash,response,version) VALUES(?,0,?,?,?,0)').bind(id, duplicate.commandId, commandHash, JSON.stringify(old.view)).run();
      const joinBody = { inviteCode: invite, name: '旧核对手', deckId: 'hunters', requestId: key() };
      const joinKey = await receiptKeys.entryKey(joinBody, { kind: 'join', inviteCode: invite, name: '旧核对手', deckId: 'hunters' });
      await db.prepare('INSERT INTO entry_receipts(request_hash,intent_hash,response,room_id) VALUES(?,?,?,?)').bind(joinKey.requestHash, joinKey.intentHash, oldResponse, id).run();
      const snapshot = async () => {
        const tables = {};
        for (const table of ['rooms', 'seats', 'commands', 'entry_receipts', 'journal']) tables[table] = (await db.prepare(`SELECT * FROM ${table} WHERE ${table === 'rooms' ? 'id' : 'room_id'}=?`).bind(id).all()).results;
        return tables;
      };
      const before = await snapshot();
      const assertRejected = result => {
        assert.equal(result.status, 410); assert.equal(result.body.error, 'unsupported_room_version');
        assert.match(result.body.message, /新建牌桌/); assert(!result.body.view);
      };
      for (const operation of ['state', 'catalog']) assertRejected(await api(path(actor, operation), null, actor));
      assertRejected(await api(path(actor, 'state') + '?afterVersion=0', null, actor));
      assertRejected(await api(path(actor, 'commands'), { commandId: key(), expectedVersion: 0, action: { kind: 'game', action: { kind: 'ready' } } }, actor));
      assertRejected(await api(path(actor, 'quote'), { windowId: 'old', intentId: 'old' }, actor));
      assertRejected(await api('/api/rooms/join', { inviteCode: invite, name: '旧核对手', deckId: 'hunters', requestId: key() }));
      assertRejected(await api(path(actor, 'commands'), duplicate, actor));
      assertRejected(await api('/api/rooms', createBody));
      assertRejected(await api('/api/rooms/join', joinBody));
      assert.deepEqual(await snapshot(), before);
      assert.equal((await store.room(id)).state, old.state); assert.deepEqual(await counts(id), [1, 1, 0]);
      oldRooms.push({ id, actor, opaque: old.state, engine: old.view.versions.engine, assertRejected, before, snapshot });
    }
    const draft = { id: 'cloud-draft-a', name: '云端自组', description: '复制预组后编辑', societyId: null,
      cards: [{ cardId: 'JC005', count: 3 }, { cardId: 'JC125', count: 47 }],
      rulesVersion: catalog.body.rulesVersion, cardPoolVersion: catalog.body.cardPoolVersion,
      engineVersion: catalog.body.engineVersion, updatedAt: '2026-10-02T16:00:00Z' };
    const customBody = { name: '构筑甲', mode: 'duel', deckDraft: draft, requestId: key() };
    const customCreates = await Promise.all([api('/api/rooms', customBody), api('/api/rooms', customBody)]);
    assert(customCreates.every(result => result.status === 200));
    assert.deepEqual(customCreates[0], customCreates[1]);
    const customHost = customCreates[0].body;
    const entries = cards => [...cards].sort((left, right) => left.cardId.localeCompare(right.cardId));
    assert.deepEqual(entries(customHost.view.yourDeck.cards), entries(draft.cards));
    assert.equal(customHost.view.players[0].deckName, draft.name);
    assert.equal((await api('/api/rooms', { ...customBody, deckDraft: { ...draft, name: '改变同请求意图' } })).status, 409);
    for (const invalidDraft of [
      { ...draft, cards: [{ cardId: 'JC125', count: 49 }] },
      { ...draft, cards: [{ cardId: 'unknown-source', count: 50 }] },
      { ...draft, cards: [{ cardId: 'JC058', count: 50 }] },
      { ...draft, societyId: 'unknown-society-source' },
      { ...draft, engineVersion: 'rust-v0.2.3' },
    ]) assert.equal((await api('/api/rooms', { ...customBody, deckDraft: invalidDraft, requestId: key() })).status, 400);
    const guestDraft = { ...draft, id: 'cloud-draft-b', name: '构筑乙卡组', cards: [{ cardId: 'JC125', count: 50 }] };
    const customJoinBody = { name: '构筑乙', inviteCode: customHost.inviteCode, deckDraft: guestDraft, requestId: key() };
    const customJoins = await Promise.all([api('/api/rooms/join', customJoinBody), api('/api/rooms/join', customJoinBody)]);
    assert(customJoins.every(result => result.status === 200)); assert.deepEqual(customJoins[0], customJoins[1]);
    const customGuest = customJoins[0].body;
    assert.deepEqual(customGuest.view.yourDeck.cards, guestDraft.cards);
    assert(!JSON.stringify(customGuest.view.players).includes('cards'));
    const frozenHostView = (await api(path(customHost, 'state'), null, customHost)).body;
    draft.cards[0].count += 1; // Editing the original browser draft cannot mutate the persisted copy.
    assert.deepEqual(durableView((await api(path(customHost, 'state'), null, customHost)).body), durableView(frozenHostView));
    const stale = await api(path(host, 'commands'), { commandId: key(), expectedVersion: 4, action: { kind: 'ready' } }, host);
    assert.equal(stale.status, 409); assert.equal(stale.body.view.version, 5);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    const before = await db.prepare('SELECT * FROM rooms WHERE id=?').bind(host.roomId).first();
    const zero = await store.command({ id: host.roomId, seat: 0, commandId: key(), intentHash: 'fixture', expectedVersion: 4,
      state: before.state, version: 5, nonce: key(), response: '{}', entry: '{}' });
    assert.equal(zero, false); assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    assert.deepEqual(await db.prepare('SELECT * FROM rooms WHERE id=?').bind(host.roomId).first(), before);
    // A journal unique violation after a successful CAS must roll back that CAS.
    await db.prepare('INSERT INTO journal(room_id,version,entry) VALUES(?,6,?)').bind(host.roomId, '{}').run();
    await assert.rejects(store.command({ id: host.roomId, seat: 0, commandId: key(), intentHash: 'fixture', expectedVersion: 5,
      state: before.state, version: 6, nonce: key(), response: '{}', entry: '{}' }));
    assert.deepEqual(await db.prepare('SELECT * FROM rooms WHERE id=?').bind(host.roomId).first(), before);
    await db.prepare('DELETE FROM journal WHERE room_id=? AND version=6').bind(host.roomId).run();
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    const snapshot = (await api(path(host, 'state'), null, host)).body;
    assert.equal(snapshot.version, 5); assert(!('state' in snapshot)); assert(!('seed' in snapshot));
    await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
    assert.deepEqual(durableView((await api(path(customHost, 'state'), null, customHost)).body), durableView(frozenHostView));
    assert.deepEqual((await api('/api/rooms/join', customJoinBody)).body, customGuest);
    for (const old of oldRooms) {
      old.assertRejected(await api(path(old.actor, 'state'), null, old.actor));
      old.assertRejected(await api('/api/rooms/join', { ...customJoinBody, inviteCode: (await new RoomStore(db).room(old.id)).invite, requestId: key() }));
      assert.equal((await new RoomStore(db).room(old.id)).state, old.opaque);
      assert.deepEqual(await counts(old.id), [1, 1, 0]);
      assert.deepEqual(await old.snapshot(), old.before);
    }
    assert.deepEqual(durableView((await api(path(host, 'state'), null, host)).body), durableView(snapshot));
    assert.deepEqual((await api(path(host, 'commands'), original, host)).body, identical[0].body);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    t.diagnostic(JSON.stringify({ currentVersion: catalog.body.engineVersion, unsupportedFrozenVersions: oldRooms.map(r => r.engine), oldDataUntouchedBeforeAndAfterReopen: true, localWorkerdD1Only: true }));
    t.diagnostic('Provider emulation only; remote Cloudflare D1 acceptance is still required. Persisted at ' + persist);
  } finally { await mf.dispose(); }
});
