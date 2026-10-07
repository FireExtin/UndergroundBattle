// Explicit offline native layout; real current WASM, RoomService and local D1.
// No production room, owner credential, capacity benchmark or UI draft recovery.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
import * as abi from '../generated/hegemony_wasm.js';

for (const [label,file] of [['saved paused table','pause-resume-native-v047.json'],['sealed choice/trigger/paid stack pause','sealing-room-native-v047.json']]) test(label+': exact native transitions, D1 reopen, receipts and isolated second table', async t => {
  abi.initSync({ module: readFileSync('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm') });
  const trace = JSON.parse(readFileSync(new URL('./fixtures/'+file, import.meta.url), 'utf8'));
  const kernel = routeKernels(abi);
  const savedIndex=trace.steps.findIndex(step=>step.command?.action.kind==='pauseRoom' && step.transition.outcome!=='rejected');
  const resumeIndex=trace.steps.findIndex(step=>step.command?.action.kind==='resumeRoom' && step.transition.outcome!=='rejected');
  assert(savedIndex>=0 && resumeIndex>savedIndex);
  const persist = mkdtempSync(join(tmpdir(), 'saved-table-d1-'));
  const options = convertV4MiniflareOptions({ name: 'saved-table-test', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'saved-table-db' } });
  let mf = new Miniflare(options); t.after(() => mf.dispose());
  let db = await mf.getD1Database('DB'), now = 0;
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const initial = JSON.parse(abi.view(trace.initialState, 0));
  const id = initial.roomId, tokens = ['a', 'b', 'c', 'd'].map(x => x.repeat(64));
  let store = new RoomStore(db), service = new RoomService(db, kernel, () => now);
  await store.create({ id, invite: initial.inviteCode, state: trace.initialState, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'explicit-offline-native-layout', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  for (let seat = 1; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  const b = await service.create({ name: 'Independent table', mode: 'duel', deckId: 'responders', requestId: crypto.randomUUID() });
  const snapshot = async roomId => {
    const result = {};
    for (const table of ['rooms','seats','commands','journal','entry_receipts']) result[table] = (await db.prepare(`SELECT * FROM ${table} WHERE ${table === 'rooms' ? 'id' : 'room_id'}=?`).bind(roomId).all()).results;
    return result;
  };
  let bFrozen;
  for (const [index, step] of trace.steps.entries()) {
    now = Number(step.serverNowMs);
    const authorization = 'Bearer ' + tokens[step.seat];
    let view;
    if (!step.command) view = await service.state(id, authorization, null);
    else if (step.transition.outcome === 'rejected') {
      await assert.rejects(service.command(id, authorization, step.command), error => {
        assert.equal(error.code, step.transition.errorCode); view = error.view; return true;
      });
    } else view = await service.command(id, authorization, step.command);
    assert.deepEqual(view, step.transition.view);
    assert.equal((await store.room(id)).state, step.transition.state, `opaque native state at ${index}`);
    for (let seat = 0; seat < 4; seat++) assert.deepEqual(JSON.parse(abi.view(step.transition.state, seat)), step.views[seat]);
    if (index === savedIndex) {
      const saved = await snapshot(id);
      now = 86_400_000;
      assert.deepEqual(await service.command(id, authorization, step.command), view);
      assert.equal(await service.state(id, authorization, String(view.version)), null);
      await assert.rejects(service.command(id, 'Bearer ' + b.token, { commandId: 'foreign', expectedVersion: view.version, action: { kind: 'resumeRoom' } }), error => error.status === 401);
      await assert.rejects(service.command(id, undefined, { commandId: 'spectator', expectedVersion: view.version, action: { kind: 'resumeRoom' } }), error => error.status === 401);
      // Same commandId is independently accepted in B while A stays paused.
      await service.command(b.roomId, 'Bearer ' + b.token, { commandId: step.command.commandId, expectedVersion: 0, action: { kind: 'game', action: { kind: 'ready' } } });
      assert.deepEqual(await snapshot(id), saved); bFrozen = await snapshot(b.roomId);
      await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
      store = new RoomStore(db); service = new RoomService(db, kernel, () => now);
      assert.deepEqual(await snapshot(id), saved);
    }
    if (index > savedIndex) assert.deepEqual(await snapshot(b.roomId), bFrozen);
    if (index === resumeIndex) {
      now = 172_800_000;
      assert.deepEqual(await service.command(id, authorization, step.command), view);
      await assert.rejects(service.command(id, authorization, { ...step.command, expectedVersion: step.command.expectedVersion + 1 }), error => error.status === 409);
    }
  }
});

test('engine47 rejects preserved actual engine46 paused room without relabelling its state',()=>{
  abi.initSync({module:readFileSync('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')});
  const old=JSON.parse(readFileSync(new URL('./fixtures/pause-resume-native-v046.json',import.meta.url),'utf8'));
  const identity=JSON.parse(abi.stateIdentity(old.initialState));
  assert.equal(identity.versions.engine,'rust-v0.2.46-pause-resume-candidate');
  assert.throws(()=>routeKernels(abi).view(old.initialState,0),error=>error.code==='unsupported_room_version' && error.status===410);
});
