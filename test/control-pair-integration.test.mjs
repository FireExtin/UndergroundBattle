import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';

test('real Worker/D1 restores paid magic return and another actor-controlled hide, rejecting a teammate', async t => {
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/prepared-control-pair-v028.json', import.meta.url)));
  assert.equal(fixture.syntheticInitialLayout, true);
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-control-pair-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-control-pair', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-control-pair-db' } });
  let mf = new Miniflare(options); let db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId; const tokens = ['3','4','5','6'].map(c => c.repeat(64));
  // Explicit isolated local fixture. No public room or browser state is written.
  await db.prepare('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)')
    .bind(id, 'CONTROLPAIRTEST', fixture.state, fixture.state, fixture.version, 'fixture').run();
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
  const exhausted = s => JSON.parse(s).game.players[0].assets.filter(c => c.exhausted).length;
  const counts = async () => (await db.batch(['commands','journal'].map(table => db.prepare(`SELECT count(*) n FROM ${table} WHERE room_id=?`).bind(id)))).map(r => r.results[0].n);
  const settle = async () => {
    for (let i = 0; i < 8; i++) {
      const view = (await api(0, 'state')).body;
      if (!view.responseWindow) return view;
      const member = view.responseWindow.members.find(m => m.status === 'undecided');
      assert(member); const seat = Number(member.playerId.slice(1));
      assert.equal((await api(seat, 'commands', cmd(view.version, { kind: 'passResponse', windowId: view.responseWindow.id }))).status, 200);
    }
    assert.fail('Response window did not settle');
  };
  const reopen = async () => { await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB'); };
  const rejectUnchanged = async (seat, version, action) => {
    const before = await state(); const beforeCounts = await counts();
    assert.equal((await api(seat, 'commands', cmd(version, { kind: 'game', action }))).status, 400);
    assert.equal(await state(), before); assert.deepEqual(await counts(), beforeCounts);
  };
  await rejectUnchanged(0, fixture.version, { kind: 'play', cardId: fixture.gateId, targetId: fixture.publicFriendId });
  const returnCommand = cmd(fixture.version, fixture.firstAction);
  const played = await api(0, 'commands', returnCommand); assert.equal(played.status, 200);
  assert.equal(exhausted(await state()), 2);
  const paidState = await state(); const paidCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', returnCommand), played);
  assert.equal(await state(), paidState); assert.deepEqual(await counts(), paidCounts);
  const afterReturn = await settle();
  for (let seat = 0; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body;
    assert(!v.regions[2].characters.some(c => c.instanceId === fixture.returnId));
    assert.equal(v.attachments.length, 0);
    if (seat === 2) {
      const returned = v.hand.find(c => c.cardId === 'JC002'); assert(returned);
      assert.notEqual(returned.instanceId, fixture.returnId); assert.equal(returned.controller, 'p2');
    } else assert(!v.hand.some(c => c.cardId === 'JC002'));
    if (seat === 3) assert(v.hand.some(c => c.cardId === 'BQ022'));
  }
  await rejectUnchanged(0, afterReturn.version, { kind: 'conceal', cardId: fixture.lawyerId, region: 0 });
  const deployCommand = cmd(afterReturn.version, { kind: 'game', action: { kind: 'deploy', cardId: fixture.lawyerId, region: 0 } });
  const deployed = await api(0, 'commands', deployCommand); assert.equal(deployed.status, 200);
  assert.equal(exhausted(await state()), 4);
  await reopen(); assert.deepEqual(await api(0, 'commands', deployCommand), deployed);
  assert.equal(exhausted(await state()), 4);
  const chooser = await settle(); assert.equal(chooser.pendingChoice.kind, 'trigger');
  assert.deepEqual(chooser.pendingChoice.options.map(o => o.id).sort(), [fixture.allyId, fixture.publicFriendId].sort());
  const lawyer = chooser.regions[0].characters.find(c => c.cardId === 'XQ16');
  assert(lawyer); assert.notEqual(lawyer.instanceId, fixture.lawyerId);
  for (const seat of [1,2,3]) assert.equal((await api(seat, 'state')).body.pendingChoice, null);
  for (const target of [lawyer.instanceId, fixture.enemyId, fixture.teammateId]) await rejectUnchanged(0, chooser.version, { kind: 'choose', choiceId: chooser.pendingChoice.id, selected: [target] });
  await reopen();
  const restored = (await api(0, 'state')).body;
  assert.deepEqual(restored.pendingChoice, chooser.pendingChoice);
  const chooseCommand = cmd(restored.version, { kind: 'game', action: { kind: 'choose', choiceId: restored.pendingChoice.id, selected: [fixture.allyId] } });
  const chosen = await api(0, 'commands', chooseCommand); assert.equal(chosen.status, 200);
  const chosenState = await state(); const chosenCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', chooseCommand), chosen);
  assert.equal(await state(), chosenState); assert.deepEqual(await counts(), chosenCounts);
  await settle(); await reopen();
  for (let seat = 0; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body;
    const hidden = v.regions[4].characters.find(c => c.faceDown);
    assert(hidden); assert.notEqual(hidden.instanceId, fixture.allyId);
    assert.equal(hidden.owner, 'p2'); assert.equal(hidden.controller, 'p0');
    if (seat === 0) assert.equal(hidden.cardId, 'JC001');
    else { assert.equal(hidden.name, '暗藏者'); for (const k of ['cardId','text','cost','icons','defense','color','magic']) assert(!Object.hasOwn(hidden, k)); }
    assert.equal(v.pendingChoice, null);
    const teammate = v.regions[4].characters.find(c => c.instanceId === fixture.teammateId);
    assert(teammate); assert.equal(teammate.faceDown, false); assert.equal(teammate.controller, 'p1');
  }
  assert.equal(exhausted(await state()), 4);
});
