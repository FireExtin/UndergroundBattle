// Actual terminal checkpoint produced by the fresh-lobby Native match, then
// current WASM/RoomService/local D1 commands. Not a browser or online DB test.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
import * as abi from '../generated/hegemony_wasm.js';

test('fresh Native match terminal seal: D1 restart, reopen, next choice and original restart receipt', async t => {
  abi.initSync({ module: readFileSync('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm') });
  const trace = JSON.parse(readFileSync(new URL('./fixtures/sealed-restart-native-v059.json', import.meta.url), 'utf8'));
  assert.equal(trace.scope, 'natural-fresh-match-terminal-state-then-real-restart-persist-next-choice');
  const kernel = routeKernels(abi);
  const persist = mkdtempSync(join(tmpdir(), 'sealed-restart-d1-'));
  const options = convertV4MiniflareOptions({ name: 'sealed-restart-test', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'sealed-restart-db' } });
  let mf = new Miniflare(options);
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  let db = await mf.getD1Database('DB'), now = 0;
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const initial = JSON.parse(abi.view(trace.initialState, 0));
  assert.equal(initial.status, 'finished');
  assert.equal(initial.sealedCards.length, 1);
  const id = initial.roomId, tokens = ['1', '2', '3', '4'].map(x => x.repeat(64));
  let store = new RoomStore(db), service = new RoomService(db, kernel, () => now);
  await store.create({ id, invite: initial.inviteCode, state: trace.initialState, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'actual-fresh-native-terminal-checkpoint', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  for (let seat = 1; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  const other = await service.create({ name: 'Independent restart table', mode: 'duel', deckId: 'responders', requestId: crypto.randomUUID() });
  const snapshot = async roomId => {
    const result = {};
    for (const table of ['rooms','seats','commands','journal','entry_receipts']) result[table] = (await db.prepare(`SELECT * FROM ${table} WHERE ${table === 'rooms' ? 'id' : 'room_id'}=?`).bind(roomId).all()).results;
    return result;
  };
  const otherFrozen = await snapshot(other.roomId);
  for (const [index, step] of trace.steps.entries()) {
    now = Number(step.serverNowMs);
    const view = await service.command(id, 'Bearer ' + tokens[step.seat], step.command);
    assert.deepEqual(view, step.transition.view);
    assert.equal(view.status, 'playing');
    assert.deepEqual(view.sealedCards || [], []);
    assert.equal((await store.room(id)).state, step.transition.state);
    for (let seat = 0; seat < 4; seat++) assert.deepEqual(JSON.parse(abi.view(step.transition.state, seat)), step.views[seat]);
    const frozen = await snapshot(id);
    if (index === 0) {
      assert.equal(view.pendingChoice.kind, 'mulligan');
      await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
      store = new RoomStore(db); service = new RoomService(db, kernel, () => now);
      assert.deepEqual(await snapshot(id), frozen);
      assert.deepEqual(await service.state(id, 'Bearer ' + tokens[step.seat], null), view);
    }
    now += 86_400_000;
    assert.deepEqual(await service.command(id, 'Bearer ' + tokens[0], trace.steps[0].command), trace.steps[0].transition.view);
    await assert.rejects(service.command(id, 'Bearer ' + tokens[0], { ...trace.steps[0].command, expectedVersion: trace.steps[0].command.expectedVersion + 1 }), error => error.status === 409);
    await assert.rejects(service.command(id, 'Bearer ' + other.token, { commandId: 'foreign-restart', expectedVersion: view.version, action: { kind: 'game', action: { kind: 'restart' } } }), error => error.status === 401);
    assert.deepEqual(await snapshot(id), frozen);
    assert.deepEqual(await snapshot(other.roomId), otherFrozen);
  }
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n, 2);
});
