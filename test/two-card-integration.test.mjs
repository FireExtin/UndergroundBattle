import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';

test('real Worker/D1 restores controller-only owl identity and paid skeleton entry trigger', async t => {
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/prepared-two-cards-v028.json', import.meta.url)));
  assert.equal(fixture.syntheticInitialLayout, true);
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-two-card-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-two-card', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-two-card-db' } });
  let mf = new Miniflare(options); let db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId; const tokens = ['e','f','1','2'].map(c => c.repeat(64));
  // Isolated local fixture, never browser storage or a public game room.
  await db.prepare('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)')
    .bind(id, 'TWOCARDTEST', fixture.state, fixture.state, fixture.version, 'fixture').run();
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
  const exhausted = (s, seat) => JSON.parse(s).game.players[seat].assets.filter(c => c.exhausted).length;
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
  const hiddenViews = async () => {
    for (let seat = 0; seat < 4; seat++) {
      const c = (await api(seat, 'state')).body.regions[2].characters.find(c => c.instanceId === fixture.oldOwlId);
      assert(c?.faceDown); assert.equal(c.owner, 'p2'); assert.equal(c.controller, 'p1');
      if (seat === 1) assert.equal(c.cardId, 'JC001');
      else { assert.equal(c.name, '暗藏者'); for (const k of ['cardId','text','cost','icons','defense','color','magic']) assert(!Object.hasOwn(c, k)); }
    }
  };
  await hiddenViews(); await reopen(); await hiddenViews();
  const revealCommand = cmd(fixture.version, fixture.firstAction);
  const revealed = await api(1, 'commands', revealCommand); assert.equal(revealed.status, 200);
  assert.equal(exhausted(await state(), 1), 2); // One was already exhausted; reveal pays only the remaining one.
  assert.deepEqual(await api(1, 'commands', revealCommand), revealed);
  const afterReveal = await settle();
  const owl = afterReveal.regions[2].characters.find(c => c.cardId === 'JC001');
  assert(owl); assert.notEqual(owl.instanceId, fixture.oldOwlId);
  assert.equal(owl.icons.investigation, 1); assert.equal(owl.icons.influence, 1);
  for (let seat = 0; seat < 4; seat++) assert.equal((await api(seat, 'state')).body.regions[2].characters.find(c => c.instanceId === owl.instanceId).cardId, 'JC001');
  const deployCommand = cmd(afterReveal.version, { kind: 'game', action: { kind: 'deploy', cardId: fixture.skeletonId, region: 2 } });
  const deployed = await api(0, 'commands', deployCommand); assert.equal(deployed.status, 200);
  assert.equal(exhausted(await state(), 0), 4);
  const paidState = await state(); const paidCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', deployCommand), deployed);
  assert.equal(await state(), paidState); assert.deepEqual(await counts(), paidCounts);
  const chooser = await settle(); assert.equal(chooser.pendingChoice.kind, 'trigger');
  assert(chooser.pendingChoice.options.some(o => o.id === owl.instanceId));
  assert(!chooser.pendingChoice.options.some(o => o.id === fixture.oldOwlId || o.id === fixture.outsideId));
  for (const seat of [1,2,3]) assert.equal((await api(seat, 'state')).body.pendingChoice, null);
  const beforeBad = await state(); const beforeBadCounts = await counts();
  const invalid = await api(0, 'commands', cmd(chooser.version, { kind: 'game', action: { kind: 'choose', choiceId: chooser.pendingChoice.id, selected: [fixture.outsideId] } }));
  assert.equal(invalid.status, 400); assert.equal(await state(), beforeBad); assert.deepEqual(await counts(), beforeBadCounts);
  await reopen();
  const restored = (await api(0, 'state')).body;
  assert.deepEqual(restored.pendingChoice, chooser.pendingChoice);
  const chooseCommand = cmd(restored.version, { kind: 'game', action: { kind: 'choose', choiceId: restored.pendingChoice.id, selected: [owl.instanceId] } });
  const chosen = await api(0, 'commands', chooseCommand); assert.equal(chosen.status, 200);
  const chosenState = await state(); const chosenCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', chooseCommand), chosen);
  assert.equal(await state(), chosenState); assert.deepEqual(await counts(), chosenCounts);
  const final = await settle();
  assert(!final.regions[2].characters.some(c => c.instanceId === owl.instanceId));
  const skeleton = final.regions[2].characters.find(c => c.cardId === 'BQ083');
  assert(skeleton); assert.notEqual(skeleton.instanceId, fixture.skeletonId);
  assert(final.regions[3].characters.some(c => c.instanceId === fixture.outsideId));
  for (let seat = 0; seat < 4; seat++) {
    const v = (await api(seat, 'state')).body;
    assert(v.graveyard.some(c => c.cardId === 'JC001' && c.owner === 'p2'));
    assert.equal(v.pendingChoice, null);
  }
  assert.equal(exhausted(await state(), 0), 4); assert.equal(exhausted(await state(), 1), 2);
});
