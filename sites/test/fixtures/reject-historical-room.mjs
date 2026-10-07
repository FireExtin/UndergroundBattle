// Local synthetic fixtures only. Production rooms and credentials are never read.
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../../src/store.mjs';
import { digest } from '../../src/service.mjs';

export async function rejectHistoricalRoom(t, fixtureUrl) {
  const fixture = JSON.parse(readFileSync(fixtureUrl, 'utf8'));
  const persist = mkdtempSync(join(tmpdir(), 'current-only-old-fixture-'));
  const options = convertV4MiniflareOptions({ name: 'historical-rejection', resourcePersistencePath: persist,
    modules: [{ type: 'ESModule', path: resolve('dist/server/index.js') }, ...readdirSync('dist/server').filter(n => n.endsWith('.wasm')).map(n => ({ type: 'CompiledWasm', path: resolve('dist/server', n) }))],
    compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'historical-rejection-db' } });
  let mf = new Miniflare(options); t.after(() => mf.dispose()); let db = await mf.getD1Database('DB');
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId, invite = 'LOCALHISTORY', tokens = ['a', 'b', 'c', 'd'].map(x => x.repeat(64));
  await new RoomStore(db).create({ id, invite, state: fixture.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]), requestHash: crypto.randomUUID(), intentHash: 'explicit-offline-fixture', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(fixture.version, id).run();
  for (let seat = 1; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  const snapshot = async () => {
    const result = {};
    for (const table of ['rooms', 'seats', 'commands', 'entry_receipts', 'journal']) result[table] = (await db.prepare(`SELECT * FROM ${table} WHERE ${table === 'rooms' ? 'id' : 'room_id'}=?`).bind(id).all()).results;
    return result;
  };
  const before = await snapshot();
  const rejected = async (path, body, seat = 0) => {
    const response = await mf.dispatchFetch('http://localhost' + path, { method: body ? 'POST' : 'GET', headers: { Authorization: 'Bearer ' + tokens[seat], ...(body ? { 'Content-Type': 'application/json' } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    const value = await response.json(); assert.equal(response.status, 410); assert.equal(value.error, 'unsupported_room_version'); assert.match(value.message, /新建牌桌/); assert(!value.view);
  };
  for (let seat = 0; seat < 4; seat++) await rejected(`/api/rooms/${id}/state`, null, seat);
  await rejected(`/api/rooms/${id}/state?afterVersion=${fixture.version}`);
  await rejected(`/api/rooms/${id}/catalog`);
  await rejected(`/api/rooms/${id}/commands`, { commandId: crypto.randomUUID(), expectedVersion: fixture.version, action: { kind: 'game', action: { kind: 'pass' } } });
  await rejected(`/api/rooms/${id}/quote`, { windowId: 'old', intentId: 'old' });
  await rejected('/api/rooms/join', { inviteCode: invite, name: '新玩家', deckId: 'watchers', requestId: crypto.randomUUID() });
  assert.deepEqual(await snapshot(), before);
  await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
  await rejected(`/api/rooms/${id}/state`); assert.deepEqual(await snapshot(), before);
}
