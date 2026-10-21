// One current reducer in a real local workerd/D1 isolate. Retired identities only reject.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { kernel as current } from '../../rust-game-wasm/tests/current-kernel.mjs';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { assertPublicView, assertSessionJournal } from './assert-session-journal.mjs';
import { retiredIdentities } from './fixtures/historical-rooms.mjs';

const worker = process.env.HEGEMONY_CURRENT_WORKER_DIR || path.join(
  process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'current-worker/server');
test('current-only HTTP/D1 preserves u64, commands and receipts; retired versions return 410 without writes', async t => {
  const modules = readdirSync(worker).filter(f => f.endsWith('.wasm'));
  assert.equal(modules.length, 1);
  const options = convertV4MiniflareOptions({ name: 'current-kernel-http',
    resourcePersistencePath: mkdtempSync(path.join(tmpdir(), 'current-kernel-http-')), modulesRoot: worker,
    modules: [{ type: 'ESModule', path: path.join(worker, 'worker.js') },
      ...modules.map(f => ({ type: 'CompiledWasm', path: path.join(worker, f) }))],
    compatibilityDate: '2026-10-02', cf: false, bindings: { LOCAL_TEST_CLOCK_MS: '0' },
    d1Databases: { DB: 'current-kernel-http' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  for (const file of readdirSync('drizzle').filter(f => f.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const token = '1'.repeat(64), store = () => new RoomStore(db);
  const api = async (id, operation, body) => {
    const response = await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${operation}`, {
      method: body ? 'POST' : 'GET', headers: { Authorization: 'Bearer ' + token,
        ...(body ? { 'Content-Type': 'application/json' } : {}) },
      ...(body ? { body: JSON.stringify(body) } : {}) });
    return { status: response.status, body: await response.json() };
  };
  const snapshot = async id => Object.fromEntries(await Promise.all(
    ['rooms', 'seats', 'commands', 'entry_receipts', 'journal'].map(async table => [table,
      (await db.prepare(`SELECT * FROM ${table} WHERE ${table === 'rooms' ? 'id' : 'room_id'}=?`).bind(id).all()).results])));
  const id = 'f'.repeat(24), invite = 'CURRENTU64';
  const created = JSON.parse(current.newGame(id, invite, 'duel', 'P0', 'watchers', '18446744073709551615'));
  assert(created.state.includes('"seed":18446744073709551615'));
  await store().create({ id, invite, state: created.state, nonce: crypto.randomUUID(), tokenHash: await digest(token),
    requestHash: crypto.randomUUID(), intentHash: 'offline-current-http', response: '{}' });
  const before = await api(id, 'state');
  assert.equal(before.status, 200); assertPublicView(before.body, created.view);
  const command = { commandId: 'current-ready', expectedVersion: 0, action: { kind: 'game', action: { kind: 'ready' } } };
  const expected = JSON.parse(current.applyRoom(created.state, 0, JSON.stringify(command), '0'));
  const result = await api(id, 'commands', command);
  assert.equal(result.status, 200); assertPublicView(result.body, expected.view);
  assert.equal((await store().room(id)).state, expected.state);
  const journal = await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id, expected.version).first();
  assertSessionJournal(journal.entry, expected.journal);
  const saved = await snapshot(id);
  await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
  assert.deepEqual(await snapshot(id), saved);
  assert.deepEqual(await api(id, 'commands', command), result);
  assert.deepEqual(await snapshot(id), saved);
  assert.equal(saved.commands.length, 1);
  let rejected = 0;
  for (const { abi, identity } of retiredIdentities) {
    const retiredId = String(abi).padStart(2, '0').repeat(12);
    // Deliberately header-only rejection input, never a playable or migrated old room.
    const state = JSON.stringify(identity);
    await store().create({ id: retiredId, invite: 'RETIRED' + abi, state, nonce: crypto.randomUUID(),
      tokenHash: await digest(token), requestHash: crypto.randomUUID(), intentHash: 'retired-header-only', response: '{}' });
    const original = await snapshot(retiredId);
    for (const [operation, body] of [['state'], ['catalog'], ['commands', command],
      ['quote', { windowId: 'retired', intentId: 'retired' }]]) {
      const response = await api(retiredId, operation, body);
      assert.equal(response.status, 410); assert.equal(response.body.error, 'unsupported_room_version');
      assert.match(response.body.message, /新建牌桌/); assert.equal(response.body.view, undefined);
      rejected++;
    }
    assert.deepEqual(await snapshot(retiredId), original);
  }
  t.diagnostic(JSON.stringify({ currentEngine: JSON.parse(current.catalog()).engineVersion, wasmModules: 1,
    accepted: 1, receiptAfterReopen: true, retiredHeaderIdentities: retiredIdentities.length, rejected,
    opaqueStateExact: true, seedU64Exact: true, syntheticDatabaseRetained: true }));
});
