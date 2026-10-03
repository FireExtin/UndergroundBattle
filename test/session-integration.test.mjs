import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';

for (const [engineVersion, fixturePath] of [
  ['rust-v0.2.5', './fixtures/prepared-response-v025.json'],
  ['rust-v0.2.6', './fixtures/prepared-response-v026.json'],
  ['rust-v0.2.7', './fixtures/prepared-response-v027.json'],
  ['rust-v0.2.8', './fixtures/prepared-response-v028.json'],
]) test(`real Worker/D1 ${engineVersion} preserves composing, untimed choices, expiry and receipts through reopen`, async t => {
  const fixture = JSON.parse(readFileSync(new URL(fixturePath, import.meta.url), 'utf8'));
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-session-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-session', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-session-db' } });
  let mf = new Miniflare(options); let db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId;
  assert.match(id, /^[a-f0-9]{24}$/);
  const tokens = ['a','b','c','d'].map(c => c.repeat(64));
  // This local test imports an explicit synthetic layout. Production and browser
  // acceptance never insert/modify an active game's state this way.
  await db.prepare('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)')
    .bind(id, 'SESSIONTEST', fixture.state, fixture.state, fixture.version, 'fixture').run();
  for (let seat = 0; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  const api = async (seat, endpoint, body) => {
    const r = await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${endpoint}`, {
      method: body ? 'POST' : 'GET', headers: { Authorization: 'Bearer ' + tokens[seat], ...(body ? { 'Content-Type': 'application/json' } : {}) },
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    return { status: r.status, body: r.status === 204 ? null : await r.json() };
  };
  const cmd = (expectedVersion, action) => ({ commandId: crypto.randomUUID(), expectedVersion, action });
  const counts = async () => (await db.batch(['commands','journal'].map(table => db.prepare(`SELECT count(*) n FROM ${table} WHERE room_id=?`).bind(id)))).map(r => r.results[0].n);
  const state = async () => (await new RoomStore(db).room(id)).state;
  try {
    const paid = cmd(fixture.version, fixture.firstAction);
    const first = await api(0, 'commands', paid);
    assert.equal(first.status, 200); assert.equal(first.body.versions.engine, engineVersion);
    assert.equal(first.body.roomId, id);
    const window = first.body.responseWindow;
    assert.equal(window.members.length, 2); assert(window.members.every(member => member.status === 'undecided'));
    assert(window.members.every(member => member.deadlineMs - first.body.serverNowMs === 5000));
    const begin = [0, 1].map(seat => cmd(first.body.version, { kind: 'beginResponse', windowId: window.id, intentId: `intent-${seat}` }));
    const both = await Promise.all(begin.map((body, seat) => api(seat, 'commands', body)));
    assert(both.every(r => r.status === 200));
    assert.deepEqual(both.map(r => r.body.version).sort((a,b) => a-b), [first.body.version + 1, first.body.version + 2]);
    const composed = await api(0, 'state');
    assert(composed.body.responseWindow.members.every(member => member.status === 'composing' && member.deadlineMs === undefined));
    assert.equal(composed.body.responseWindow.myIntentId, 'intent-0');
    const opaque = await state(); const beforeCounts = await counts();
    const quote = await api(0, 'quote', { windowId: window.id, intentId: 'intent-0' });
    assert.equal(quote.status, 200); assert.equal(quote.body.version, composed.body.version);
    assert.equal(await state(), opaque); assert.deepEqual(await counts(), beforeCounts);
    const wrongIntent = await api(0, 'commands', cmd(composed.body.version,
      { kind: 'cancelAndPass', windowId: window.id, intentId: 'someone-else' }));
    assert.equal(wrongIntent.status, 409); assert.equal(await state(), opaque);
    const injected = await api(0, 'commands', cmd(composed.body.version,
      { kind: 'beginResponse', windowId: window.id, intentId: 'fake', serverNowMs: 0 }));
    assert.equal(injected.status, 400); assert.deepEqual(await counts(), beforeCounts);
    await new Promise(resolve => setTimeout(resolve, 5100));
    const stillComposing = await api(0, 'state');
    assert.equal(stillComposing.body.version, composed.body.version);
    assert(stillComposing.body.responseWindow.members.every(member => member.status === 'composing'));
    assert.equal(await state(), opaque);
    await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
    const recovered = await api(0, 'state');
    assert.equal(recovered.body.version, composed.body.version);
    assert.equal(recovered.body.responseWindow.myIntentId, 'intent-0');
    assert.deepEqual(await api(0, 'commands', begin[0]), both[0]);
    assert.equal(await state(), opaque); assert.deepEqual(await counts(), beforeCounts);
    const cancelled0 = await api(0, 'commands', cmd(composed.body.version, { kind: 'cancelAndPass', windowId: window.id, intentId: 'intent-0' }));
    assert.equal(cancelled0.status, 200);
    assert.equal(cancelled0.body.responseWindow.members.find(m => m.playerId === 'p0').status, 'passed');
    assert.equal(cancelled0.body.responseWindow.members.find(m => m.playerId === 'p1').status, 'composing');
    const cancelled1 = await api(1, 'commands', cmd(cancelled0.body.version, { kind: 'cancelAndPass', windowId: window.id, intentId: 'intent-1' }));
    assert.equal(cancelled1.status, 200); assert.notEqual(cancelled1.body.responseWindow.id, window.id);
    assert.equal(cancelled1.body.responseWindow.holderTeam, 1);
    const beforeExpiry = await counts();
    await new Promise(resolve => setTimeout(resolve, 5100));
    // Even afterVersion equal to the last revision must apply one durable expiry.
    const expiry = await api(2, 'state?afterVersion=' + cancelled1.body.version);
    assert.equal(expiry.status, 200); assert.equal(expiry.body.version, cancelled1.body.version + 1);
    assert.equal(expiry.body.responseWindow, null); assert.equal(expiry.body.stack.length, 0);
    assert.deepEqual(await counts(), [beforeExpiry[0], beforeExpiry[1] + 1]);
    const last = await db.prepare('SELECT entry FROM journal WHERE room_id=? ORDER BY version DESC LIMIT 1').bind(id).first();
    const events = JSON.parse(last.entry).SessionEvents.events;
    assert.equal(events.length, 1); assert('Tick' in events[0]);
    const finalState = await state();
    assert.deepEqual(await api(0, 'commands', paid), first); assert.equal(await state(), finalState);
    assert.equal((await api(2, 'state?afterVersion=' + expiry.body.version)).status, 204);
    assert.equal(await state(), finalState); assert.deepEqual(await counts(), [beforeExpiry[0], beforeExpiry[1] + 1]);
    t.diagnostic('Synthetic local fixture; actual Worker/D1 writes, timers and reopen. No remote natural-play claim.');
  } finally { await mf.dispose(); }
});
