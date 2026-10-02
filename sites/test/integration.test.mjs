import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { initSync, newGame } from '../generated/hegemony_wasm.js';

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
  const api = async (path, body, session) => {
    const response = await mf.dispatchFetch('http://localhost' + path, {
      method: body ? 'POST' : 'GET', headers: { ...(body ? { 'Content-Type': 'application/json' } : {}),
        ...(session ? { Authorization: 'Bearer ' + session.token } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    return { status: response.status, body: response.status === 204 ? null : await response.json() };
  };
  const counts = async id => {
    const rows = await db.batch(['seats', 'commands', 'journal'].map(table => db.prepare(`SELECT count(*) AS n FROM ${table} WHERE room_id=?`).bind(id)));
    return rows.map(r => r.results[0].n);
  };
  const key = () => crypto.randomUUID();
  const path = (session, action) => `/api/rooms/${session.roomId}/${action}`;
  try {
    const health = await api('/api/health'); assert.equal(health.body.transport, 'polling');
    const catalog = await api('/api/catalog'); assert.equal(catalog.body.cards.length, 29); assert.equal(catalog.body.entryIdempotency, true);
    const create = { name: '甲', mode: 'teams', deckId: 'responders', requestId: key() };
    const hosts = await Promise.all([api('/api/rooms', create), api('/api/rooms', create)]);
    assert.equal(hosts[0].status, 200); assert.equal(hosts[1].status, 200); assert.deepEqual(hosts[0].body, hosts[1].body);
    const host = hosts[0].body;
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
    assert.equal((await api(path(host, 'state'))).status, 401);
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
    const preciseSession = { roomId: preciseId, token: preciseToken };
    const preciseReady = await api(path(preciseSession, 'commands'), { commandId: key(), expectedVersion: 0, action: { kind: 'ready' } }, preciseSession);
    assert.equal(preciseReady.status, 200);
    const opaqueAfter = (await store.room(preciseId)).state;
    assert(opaqueAfter.includes('"seed":18446744073709551615')); assert(opaqueAfter.includes('"random":18446744073709551615'));
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
    assert.deepEqual((await api(path(host, 'state'), null, host)).body, snapshot);
    assert.deepEqual((await api(path(host, 'commands'), original, host)).body, identical[0].body);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    t.diagnostic('Provider emulation only; remote Cloudflare D1 acceptance is still required. Persisted at ' + persist);
  } finally { await mf.dispose(); }
});
