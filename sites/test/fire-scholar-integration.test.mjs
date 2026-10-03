import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';

test('real Worker/D1 restores paid region damage and private printed spell/book search', async t => {
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/prepared-fire-scholar-v028.json', import.meta.url)));
  assert.equal(fixture.syntheticInitialLayout, true);
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-fire-scholar-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-fire-scholar', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-fire-scholar-db' } });
  let mf = new Miniflare(options); let db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId; const tokens = ['7','8','9','a'].map(c => c.repeat(64));
  // Authorized isolated synthetic layout and dummy local tokens, never a public room.
  await db.prepare('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)')
    .bind(id, 'FIRESCHOLARTEST', fixture.state, fixture.state, fixture.version, 'fixture').run();
  for (let seat = 0; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  const api = async (seat, endpoint, body) => {
    const r = await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${endpoint}`, {
      method: body ? 'POST' : 'GET', headers: { Authorization: 'Bearer ' + tokens[seat], ...(body ? { 'Content-Type': 'application/json' } : {}) },
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    return { status: r.status, body: r.status === 204 ? null : await r.json() };
  };
  const cmd = (v, action) => ({ commandId: crypto.randomUUID(), expectedVersion: v, action });
  const state = async () => (await new RoomStore(db).room(id)).state;
  const spent = s => JSON.parse(s).game.players[0].assets.filter(c => c.exhausted).length;
  const counts = async () => (await db.batch(['commands','journal'].map(table => db.prepare(`SELECT count(*) n FROM ${table} WHERE room_id=?`).bind(id)))).map(r => r.results[0].n);
  const reopen = async () => { await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB'); };
  const settle = async () => {
    for (let i = 0; i < 8; i++) {
      const view = (await api(0, 'state')).body;
      if (!view.responseWindow) return view;
      const member = view.responseWindow.members.find(m => m.status === 'undecided'); assert(member);
      const result = await api(Number(member.playerId.slice(1)), 'commands', cmd(view.version, { kind: 'passResponse', windowId: view.responseWindow.id }));
      assert.equal(result.status, 200);
    }
    assert.fail('Response window did not settle');
  };
  const rejectUnchanged = async (seat, version, action) => {
    const before = await state(); const beforeCounts = await counts();
    assert.equal((await api(seat, 'commands', cmd(version, { kind: 'game', action }))).status, 400);
    assert.equal(await state(), before); assert.deepEqual(await counts(), beforeCounts);
  };
  await rejectUnchanged(0, fixture.version, { kind: 'play', cardId: fixture.fireId, region: 99 });
  await rejectUnchanged(1, fixture.version, { kind: 'play', cardId: fixture.fireId, region: 2 });
  const fireCommand = cmd(fixture.version, fixture.firstAction);
  const played = await api(0, 'commands', fireCommand); assert.equal(played.status, 200); assert.equal(spent(await state()), 3);
  const paidState = await state(); const paidCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', fireCommand), played);
  assert.equal(await state(), paidState); assert.deepEqual(await counts(), paidCounts);
  const afterFire = await settle();
  for (let seat = 0; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body;
    assert.equal(v.regions[2].characters.length, 1); const hidden = v.regions[2].characters[0];
    assert.equal(hidden.instanceId, fixture.hiddenId); assert.equal(hidden.faceDown, true);
    if (seat === 1) assert.equal(hidden.cardId, 'JC001'); else { assert.equal(hidden.name, '暗藏者'); for (const key of ['cardId','text','cost','icons','defense','color','magic']) assert(!Object.hasOwn(hidden, key)); }
    assert.equal(v.regions[3].characters[0].instanceId, fixture.outsideId);
    assert.equal(v.regions[3].characters[0].damage, 0); assert.equal(v.attachments.length, 0);
    assert(v.graveyard.some(c => c.owner === 'p' + seat && c.cardId === (seat === 2 ? 'JZ08' : 'JC125') && !fixture.deadIds.includes(c.instanceId)));
    if (seat === 1) assert(v.hand.some(c => c.cardId === 'BQ022'));
  }
  const deployCommand = cmd(afterFire.version, { kind: 'game', action: { kind: 'deploy', cardId: fixture.scholarId, region: 0 } });
  const deployed = await api(0, 'commands', deployCommand); assert.equal(deployed.status, 200); assert.equal(spent(await state()), 7);
  await reopen(); assert.deepEqual(await api(0, 'commands', deployCommand), deployed); assert.equal(spent(await state()), 7);
  const trigger = await settle(); assert.equal(trigger.pendingChoice.kind, 'trigger');
  const acceptCommand = cmd(trigger.version, { kind: 'game', action: { kind: 'choose', choiceId: trigger.pendingChoice.id, selected: ['accept'] } });
  assert.equal((await api(0, 'commands', acceptCommand)).status, 200);
  const search = await settle(); assert.equal(search.pendingChoice.kind, 'search');
  assert.deepEqual(search.pendingChoice.options.map(o => o.id), fixture.eligibleIds);
  assert.deepEqual(search.pendingChoice.options.map(o => o.card.cardId), ['JC006','XQ03','JC049']);
  for (let seat = 1; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body; assert.equal(v.pendingChoice, null);
    for (const privateId of fixture.eligibleIds) assert(!JSON.stringify(v).includes(privateId));
  }
  for (const invalid of fixture.excludedIds) await rejectUnchanged(0, search.version, { kind: 'choose', choiceId: search.pendingChoice.id, selected: [invalid] });
  await reopen(); const restored = (await api(0, 'state')).body; assert.deepEqual(restored.pendingChoice, search.pendingChoice);
  const searchCommand = cmd(restored.version, { kind: 'game', action: { kind: 'choose', choiceId: restored.pendingChoice.id, selected: [fixture.eligibleIds[1]] } });
  const searched = await api(0, 'commands', searchCommand); assert.equal(searched.status, 200);
  const doneState = await state(); const doneCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', searchCommand), searched);
  assert.equal(await state(), doneState); assert.deepEqual(await counts(), doneCounts);
  for (let seat = 0; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body; assert.equal(v.pendingChoice, null);
    assert(v.log.some(entry => entry.text.includes('展示检索的 力场束缚')));
    if (seat === 0) { const c = v.hand.find(c => c.cardId === 'XQ03'); assert(c); assert.notEqual(c.instanceId, fixture.eligibleIds[1]); }
    else assert(!v.hand.some(c => c.cardId === 'XQ03'));
  }
  assert.equal(spent(await state()), 7);
});
