// Explicit Native prepared layouts through current ABI and real local D1.
// These IDs are Native test IDs, so commands use RoomService rather than HTTP.
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
const native = JSON.parse(readFileSync(new URL('../../web/src/game/xq18Native63Test.fixture.json', import.meta.url), 'utf8'));

async function setup(t, state) {
  const persist = mkdtempSync(join(tmpdir(), 'xq18-60-d1-'));
  const options = convertV4MiniflareOptions({ name: 'xq18-60-test', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'xq18-60-test' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  for (const file of readdirSync('drizzle').filter(name => name.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const initial = JSON.parse(abi.view(state, 0)), id = initial.roomId, tokens = ['a','b','c','d'].map(x => x.repeat(64));
  let now = 0, store = new RoomStore(db), service = new RoomService(db, routeKernels(abi), () => now);
  await store.create({ id, invite: initial.inviteCode, state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'explicit-native-xq18-layout', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  for (let seat = 1; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  const verify = async fixture => {
    now = fixture.serverNow ?? 0;
    assert.equal((await store.room(id)).state, fixture.state);
    const view = await service.command(id, 'Bearer ' + tokens[fixture.seat], fixture.command);
    assert.deepEqual(view, fixture.expected.view);
    assert.equal((await store.room(id)).state, fixture.expected.state);
    for (let seat = 0; seat < 4; seat++) {
      const current = await service.state(id, 'Bearer ' + tokens[seat], null);
      assert.deepEqual({ ...current, serverNowMs: fixture.views[seat].serverNowMs }, fixture.views[seat]);
      if (seat !== 0) assert.equal(current.pendingChoice, null);
    }
    return view;
  };
  const reopen = async () => {
    await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
    store = new RoomStore(db); service = new RoomService(db, routeKernels(abi), () => now);
  };
  const duplicate = async (fixture, original) => {
    const saved = await store.room(id);
    now = 600000;
    assert.deepEqual(await service.command(id, 'Bearer ' + tokens[fixture.seat], fixture.command), original);
    await assert.rejects(service.command(id, 'Bearer ' + tokens[fixture.seat], { ...fixture.command, expectedVersion: fixture.command.expectedVersion + 1 }), e => e.status === 409);
    assert.deepEqual(await store.room(id), saved);
  };
  const rejectPaused = async (paused, choice) => {
    const saved = await store.room(id);
    now = paused.serverNow;
    const command = { ...choice.command, commandId: 'xq18-paused-worker', expectedVersion: paused.expected.view.version };
    await assert.rejects(service.command(id, 'Bearer ' + tokens[choice.seat], command), e => e.status === 409 && e.code === 'room_paused');
    assert.deepEqual(await store.room(id), saved);
  };
  return { verify, reopen, duplicate, rejectPaused };
}

for (const fixture of native.carriers) test(`XQ18 same-name carrier ${fixture.selected}: exact Native choice and D1 reopened receipt`, async t => {
  const room = await setup(t, fixture.state);
  const original = await room.verify(fixture);
  await room.reopen(); await room.duplicate(fixture, original);
});

test('XQ18 pause, reopen, resume and exact carrier choice preserve all Native state strings', async t => {
  const room = await setup(t, native.pauseResume[0].state);
  let original;
  for (const fixture of native.pauseResume) {
    original = await room.verify(fixture);
    if (fixture === native.pauseResume[0]) await room.rejectPaused(fixture, native.pauseResume.at(-1));
    await room.reopen();
  }
  await room.duplicate(native.pauseResume.at(-1), original);
});
