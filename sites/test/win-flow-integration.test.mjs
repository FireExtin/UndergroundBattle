// Continuous prepared Native traces through the production service and real D1.
// Native fixture IDs use RoomService; the ordinary HTTP suite covers routing.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import * as abi from '../generated/hegemony_wasm.js';
import { routeKernels } from '../src/kernel-router.mjs';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';

abi.initSync({ module: readFileSync('generated/hegemony_wasm_bg.wasm') });
const fixtures = JSON.parse(readFileSync(new URL('./fixtures/win-flow-native-v062.json', import.meta.url), 'utf8'));

for (const scenario of fixtures.scenarios) test(`engine62 ${scenario.name}: continuous Native commands, journals, D1 reopening and receipts`, async t => {
  const persist = mkdtempSync(join(tmpdir(), 'win-flow-62-d1-'));
  const options = convertV4MiniflareOptions({ name: 'win-flow-62', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(n => n.endsWith('.wasm')).map(n => ({ type: 'CompiledWasm', path: resolve('dist/server', n) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'win-flow-62' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const first = scenario.steps[0], initial = JSON.parse(abi.view(first.state, 0)), id = initial.roomId;
  const tokens = initial.players.map((_, seat) => String(seat + 1).repeat(64)); // Test-only seat secrets.
  let now = 0, store = new RoomStore(db), service = new RoomService(db, routeKernels(abi), () => now);
  await store.create({ id, invite: initial.inviteCode, state: first.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'explicit-native-win-flow-layout', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  for (let seat = 1; seat < tokens.length; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  let accepted = 0, rejected = 0, reopened = 0, lastAccepted, lastView;
  const reopenedStates = new Set();
  for (const step of scenario.steps) {
    now = Number(step.serverNowMs);
    assert.equal((await store.room(id)).state, step.state, step.inputName);
    if (step.expected.errorCode) {
      await assert.rejects(service.command(id, 'Bearer ' + tokens[step.seat], step.command), e => e.code === step.expected.errorCode);
      assert.equal(await store.receipt(id, step.command.commandId), null);
      rejected++;
    } else {
      const view = await service.command(id, 'Bearer ' + tokens[step.seat], step.command);
      assert.deepEqual(view, step.expected.view, step.inputName);
      const journal = await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id, step.expected.version).first();
      assert.deepEqual(JSON.parse(journal.entry), { SessionEvents: { events: step.expected.journal } });
      accepted++; lastAccepted = step; lastView = view;
    }
    assert.equal((await store.room(id)).state, step.expected.state, step.inputName);
    for (let seat = 0; seat < tokens.length; seat++) assert.deepEqual(await service.state(id, 'Bearer ' + tokens[seat], null), step.views[seat]);
    const key = step.expected.view.paused ? 'paused' : step.expected.view.regions.some(r => r.cardId === '') ? 'vacant' : step.expected.view.step.includes(':win') ? 'win' : null;
    if (key && !reopenedStates.has(key)) {
      reopenedStates.add(key);
      await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
      store = new RoomStore(db); service = new RoomService(db, routeKernels(abi), () => now);
      assert.equal((await store.room(id)).state, step.expected.state);
      reopened++;
    }
  }
  const saved = await store.room(id);
  now = 600000;
  assert.deepEqual(await service.command(id, 'Bearer ' + tokens[lastAccepted.seat], lastAccepted.command), lastView);
  await assert.rejects(service.command(id, 'Bearer ' + tokens[lastAccepted.seat], { ...lastAccepted.command, expectedVersion: lastAccepted.command.expectedVersion + 1 }), e => e.status === 409);
  assert.deepEqual(await store.room(id), saved);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n, accepted);
  t.diagnostic(JSON.stringify({ preparedNativeLayout: true, accepted, rejected, reopened, fullJournalsEqual: true, opaqueStatesEqual: true, seatViewsEqual: true, duplicateOriginalReceipt: true }));
});
