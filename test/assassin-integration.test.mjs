import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';

test('real Worker/D1 persists printed-cost guard and pays self-hide once through reopen', async t => {
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/prepared-assassin-v028.json', import.meta.url)));
  assert.equal(fixture.syntheticInitialLayout, true);
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-assassin-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-assassin', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-assassin-db' } });
  let mf = new Miniflare(options); let db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const id = fixture.roomId; const tokens = ['a','b','c','d'].map(c => c.repeat(64));
  // Explicit local synthetic fixture. No production or browser game is inserted.
  await db.prepare('INSERT INTO rooms(id,invite,initial_state,state,version,attempt_nonce) VALUES(?,?,?,?,?,?)')
    .bind(id, 'ASSASSINTEST', fixture.state, fixture.state, fixture.version, 'fixture').run();
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
  const counts = async () => (await db.batch(['commands','journal'].map(table => db.prepare(`SELECT count(*) n FROM ${table} WHERE room_id=?`).bind(id)))).map(r => r.results[0].n);
  const exhausted = s => JSON.parse(s).game.players[0].assets.filter(c => c.exhausted).length;
  const settle = async () => {
    for (let i = 0; i < 8; i++) {
      const view = (await api(0, 'state')).body;
      if (!view.responseWindow) return view;
      const member = view.responseWindow.members.find(m => m.status === 'undecided');
      assert(member); const seat = Number(member.playerId.slice(1));
      const passed = await api(seat, 'commands', cmd(view.version, { kind: 'passResponse', windowId: view.responseWindow.id }));
      assert.equal(passed.status, 200);
    }
    assert.fail('Response window did not settle');
  };
  const reopen = async () => { await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB'); };
  const hiddenViews = async (instanceId, knownToController) => {
    for (let seat = 0; seat < 4; seat++) {
      const projected = (await api(seat, 'state')).body.regions[2].characters.find(c => c.instanceId === instanceId);
      assert(projected?.faceDown);
      if (seat === 0 && knownToController) assert.equal(projected.cardId, 'JC088');
      else {
        assert.equal(projected.name, '暗藏者');
        for (const field of ['cardId', 'text', 'cost', 'icons', 'defense', 'color', 'magic']) assert(!Object.hasOwn(projected, field));
      }
    }
  };
  await hiddenViews(fixture.oldSourceId, true);
  const revealCommand = cmd(fixture.version, fixture.firstAction);
  const revealed = await api(0, 'commands', revealCommand);
  assert.equal(revealed.status, 200); assert.equal(exhausted(await state()), 3);
  assert.deepEqual(await api(0, 'commands', revealCommand), revealed);
  const chooser = await settle(); assert.equal(chooser.pendingChoice.kind, 'trigger');
  assert.deepEqual(chooser.pendingChoice.options.map(o => o.id), [fixture.targetId]);
  for (const seat of [1, 2, 3]) assert.equal((await api(seat, 'state')).body.pendingChoice, null);
  const beforeBad = await state(); const beforeBadCounts = await counts();
  const bad = await api(0, 'commands', cmd(chooser.version, { kind: 'game', action: { kind: 'choose', choiceId: chooser.pendingChoice.id, selected: [fixture.expensiveTargetId] } }));
  assert.equal(bad.status, 400); assert.equal(await state(), beforeBad); assert.deepEqual(await counts(), beforeBadCounts);
  const selectCommand = cmd(chooser.version, { kind: 'game', action: { kind: 'choose', choiceId: chooser.pendingChoice.id, selected: [fixture.targetId] } });
  const selected = await api(0, 'commands', selectCommand); assert.equal(selected.status, 200);
  const selectedState = await state(); const selectedCounts = await counts();
  assert.equal(JSON.parse(selectedState).game.stack.at(-1).frame.targets[0].spec.printed_cost_max, 2);
  await reopen();
  assert.deepEqual(await api(0, 'commands', selectCommand), selected);
  const changed = structuredClone(selectCommand); changed.action.action.selected = [fixture.expensiveTargetId];
  assert.equal((await api(0, 'commands', changed)).status, 409);
  assert.equal(await state(), selectedState); assert.deepEqual(await counts(), selectedCounts);
  const resolved = await settle();
  assert(!resolved.regions[2].characters.some(c => c.instanceId === fixture.targetId));
  assert(resolved.regions[2].characters.some(c => c.instanceId === fixture.expensiveTargetId));
  const source = resolved.regions[2].characters.find(c => c.cardId === 'JC088');
  assert(source); assert.notEqual(source.instanceId, fixture.oldSourceId);
  const hideCommand = cmd(resolved.version, { kind: 'game', action: { kind: 'activate', cardId: source.instanceId, abilityId: 'hide-self' } });
  const hidden = await api(0, 'commands', hideCommand); assert.equal(hidden.status, 200);
  assert.equal(exhausted(await state()), 5); const paidState = await state(); const paidCounts = await counts();
  await reopen(); assert.deepEqual(await api(0, 'commands', hideCommand), hidden);
  const changedHide = structuredClone(hideCommand); changedHide.action.action.cardId = fixture.expensiveTargetId;
  assert.equal((await api(0, 'commands', changedHide)).status, 409);
  assert.equal(await state(), paidState); assert.deepEqual(await counts(), paidCounts);
  const final = await settle(); assert(!final.regions[2].characters.some(c => c.instanceId === source.instanceId));
  const ownHidden = final.regions[2].characters.find(c => c.controller === 'p0' && c.faceDown && c.cardId === 'JC088');
  assert(ownHidden); assert.equal(exhausted(await state()), 5);
  assert.notEqual(ownHidden.instanceId, source.instanceId);
  await hiddenViews(ownHidden.instanceId, true);
  const finalState = await state(); const finalCounts = await counts();
  await reopen();
  await hiddenViews(ownHidden.instanceId, true);
  assert.equal(await state(), finalState); assert.deepEqual(await counts(), finalCounts);
  assert.deepEqual(await api(0, 'commands', hideCommand), hidden);
  assert.equal(exhausted(await state()), 5);
});
