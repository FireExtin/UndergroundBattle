// Prepared Native layouts; actual compiled Worker HTTP, workerd/D1 and current WASM.
// The injected service clock provides exact Native parity; HTTP uses its real clock.
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
const fixtures = JSON.parse(readFileSync(new URL('../../web/src/game/sevenCardUIV059.fixture.json', import.meta.url), 'utf8'));
for (const fixture of fixtures) test(`seven59 BQ028 attachment=${fixture.attachment} discard=${fixture.discard}: private HTTP, exact Native state and reopened receipt`, async t => {
  const persist = mkdtempSync(join(tmpdir(), 'seven59-d1-'));
  const options = convertV4MiniflareOptions({ name: 'seven59-test', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'seven59-test' } });
  let mf = new Miniflare(options);
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  let db = await mf.getD1Database('DB');
  for (const file of readdirSync('drizzle').filter(name => name.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const id = fixture.views[0].roomId, tokens = ['a', 'b', 'c', 'd'].map(x => x.repeat(64));
  let store = new RoomStore(db), now = 1000, service = new RoomService(db, routeKernels(abi), () => now);
  await store.create({ id, invite: fixture.views[0].inviteCode, state: fixture.before, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'prepared-native-seven59-layout', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(fixture.views[0].version, id).run();
  for (let seat = 1; seat < 4; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  const health = await (await mf.dispatchFetch('http://localhost/api/health')).json();
  assert.equal(health.engineVersion, 'rust-v0.2.59-seven-card-engine-candidate');
  const httpViews = async expected => {
    for (let seat = 0; seat < 4; seat++) {
      const response = await mf.dispatchFetch(`http://localhost/api/rooms/${id}/state`, { headers: { Authorization: 'Bearer ' + tokens[seat] } });
      assert.equal(response.status, 200);
      const view = await response.json();
      assert(Number.isSafeInteger(view.serverNowMs));
      assert.deepEqual({ ...view, serverNowMs: expected[seat].serverNowMs }, expected[seat]);
      if (seat !== 0) assert.equal(view.pendingChoice, null);
    }
  };
  await httpViews(fixture.views); assert.equal((await store.room(id)).state, fixture.before);
  assert.equal(fixture.views[0].pendingChoice.max, fixture.attachment ? 1 : 0);
  assert(fixture.views[0].pendingChoice.options.every(option => option.card.kind === 'attachment'));
  const view = await service.command(id, 'Bearer ' + tokens[0], fixture.command);
  assert.deepEqual(view, fixture.afterViews[0]); assert.equal((await store.room(id)).state, fixture.after);
  await httpViews(fixture.afterViews); assert.equal((await store.room(id)).state, fixture.after);
  await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
  store = new RoomStore(db); service = new RoomService(db, routeKernels(abi), () => now); now = 600000;
  assert.equal((await store.room(id)).state, fixture.after);
  const duplicate = await mf.dispatchFetch(`http://localhost/api/rooms/${id}/commands`, { method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: 'Bearer ' + tokens[0] }, body: JSON.stringify(fixture.command) });
  assert.equal(duplicate.status, 200); assert.deepEqual(await duplicate.json(), view);
  await assert.rejects(service.command(id, 'Bearer ' + tokens[0], { ...fixture.command, expectedVersion: fixture.command.expectedVersion + 1 }), e => e.status === 409);
  assert.equal((await store.room(id)).state, fixture.after);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n, 1);
  t.diagnostic(JSON.stringify({ nativeStateByteEqual: true, privateHttpViews: 4, duplicateHttpOriginalReceipt: true, persistReopen: true, browserAcceptance: false }));
});
