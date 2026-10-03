import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';

test('real Worker/D1 restores controller-only paid rescue and private forecast-three ordering', async t => {
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/prepared-rescue-forecast-v028.json', import.meta.url)));
  assert.equal(fixture.syntheticInitialLayout, true);
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-rescue-forecast-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-rescue-forecast', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-rescue-forecast-db' } });
  let mf = new Miniflare(options); let db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId; const tokens = ['7','8','9','a'].map(c => c.repeat(64));
  // Authorized isolated synthetic layout and dummy local tokens, never a public room.
  await db.prepare('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)')
    .bind(id, 'RESCUEFORECASTTEST', fixture.state, fixture.state, fixture.version, 'fixture').run();
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
  await rejectUnchanged(1, fixture.version, { kind: 'deploy', cardId: fixture.rescueId, region: 0 });
  const deploy = cmd(fixture.version, fixture.firstAction);
  const deployed = await api(0, 'commands', deploy); assert.equal(deployed.status, 200); assert.equal(spent(await state()), 1);
  const paid = await state(); const paidCounts = await counts(); await reopen();
  assert.deepEqual(await api(0, 'commands', deploy), deployed); assert.equal(await state(), paid); assert.deepEqual(await counts(), paidCounts);
  const onBoard = await settle(); const source = onBoard.regions[0].characters.find(c => c.cardId === 'JC075');
  assert(source); assert.notEqual(source.instanceId, fixture.rescueId); assert.equal(source.exhausted, false);
  const rescue = targetId => ({ kind: 'activate', cardId: source.instanceId, abilityId: 'rescue', targetId });
  for (const invalid of [source.instanceId, fixture.teammateId, fixture.hiddenId]) await rejectUnchanged(0, onBoard.version, rescue(invalid));
  await rejectUnchanged(1, onBoard.version, rescue(fixture.targetId));
  const rescueCommand = cmd(onBoard.version, { kind: 'game', action: rescue(fixture.targetId) });
  const rescuing = await api(0, 'commands', rescueCommand); assert.equal(rescuing.status, 200); assert.equal(spent(await state()), 3);
  const inFlight = await state(); await reopen(); assert.deepEqual(await api(0, 'commands', rescueCommand), rescuing); assert.equal(await state(), inFlight);
  const returned = await settle();
  for (let seat = 0; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body;
    assert.equal(v.regions[4].characters.length, 0); assert.equal(v.attachments.length, 0);
    assert.equal(v.regions[0].characters.find(c => c.instanceId === source.instanceId).exhausted, true);
    const hidden = v.regions[1].characters[0]; assert.equal(hidden.instanceId, fixture.hiddenId);
    if (seat === 0) assert.equal(hidden.cardId, 'JC125'); else { assert.equal(hidden.name, '暗藏者'); assert(!Object.hasOwn(hidden, 'cardId')); }
    if (seat === 2) { const c = v.hand.find(c => c.cardId === 'LC21'); assert(c); assert.notEqual(c.instanceId, fixture.targetId); assert.equal(c.owner, 'p2'); assert.equal(c.controller, 'p2'); }
    else assert(!v.hand.some(c => c.cardId === 'LC21'));
    if (seat === 3) assert(v.hand.some(c => c.cardId === 'BQ022')); else assert(!v.hand.some(c => c.cardId === 'BQ022'));
  }
  await rejectUnchanged(0, returned.version, rescue(fixture.teammateId));
  const forecastDeploy = cmd(returned.version, { kind: 'game', action: { kind: 'deploy', cardId: fixture.forecastId, region: 0 } });
  const forecasting = await api(0, 'commands', forecastDeploy); assert.equal(forecasting.status, 200); assert.equal(spent(await state()), 5);
  await reopen(); assert.deepEqual(await api(0, 'commands', forecastDeploy), forecasting); assert.equal(spent(await state()), 5);
  const trigger = await settle(); assert.equal(trigger.pendingChoice.kind, 'trigger');
  assert.equal((await api(0, 'commands', cmd(trigger.version, { kind: 'game', action: { kind: 'choose', choiceId: trigger.pendingChoice.id, selected: ['accept'] } }))).status, 200);
  const forecast = await settle(); assert.equal(forecast.pendingChoice.kind, 'investigation');
  assert.deepEqual(forecast.pendingChoice.options.map(o => o.id), fixture.originalDeckIds.slice(0,3));
  assert.deepEqual(forecast.pendingChoice.options.map(o => o.card.cardId), ['JC006','XQ03','JC125']);
  for (let seat = 1; seat < 4; seat++) { const v = (await api(seat, 'state')).body; assert.equal(v.pendingChoice, null); for (const id of fixture.originalDeckIds) assert(!JSON.stringify(v).includes(id)); }
  for (const [top,bottom] of [ [[fixture.originalDeckIds[0]],[]], [fixture.originalDeckIds.slice(0,3),[fixture.originalDeckIds[0]]], [fixture.originalDeckIds.slice(0,3),['foreign']] ]) {
    await rejectUnchanged(0, forecast.version, { kind: 'choose', choiceId: forecast.pendingChoice.id, top, bottom });
  }
  await reopen(); const restored = (await api(0, 'state')).body; assert.deepEqual(restored.pendingChoice, forecast.pendingChoice);
  const orderCommand = cmd(restored.version, { kind: 'game', action: { kind: 'choose', choiceId: restored.pendingChoice.id, top: [fixture.originalDeckIds[2]], bottom: [fixture.originalDeckIds[1],fixture.originalDeckIds[0]] } });
  const ordered = await api(0, 'commands', orderCommand); assert.equal(ordered.status, 200);
  const done = await state(); const doneCounts = await counts(); await reopen();
  assert.deepEqual(await api(0, 'commands', orderCommand), ordered); assert.equal(await state(), done); assert.deepEqual(await counts(), doneCounts);
  const p = JSON.parse(done).game.players[0]; assert.equal(p.hand.length, 0);
  assert.deepEqual(p.deck.map(c => c.id), [2,3,4,1,0].map(i => fixture.originalDeckIds[i]));
  assert.equal(spent(done), 5);
  for (let seat = 0; seat < 4; seat++) assert.equal((await api(seat, 'state')).body.pendingChoice, null);
});
