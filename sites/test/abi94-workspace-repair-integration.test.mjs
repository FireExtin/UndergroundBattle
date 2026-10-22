// Explicit fresh Native layouts, real local HTTP/workerd/WASM/D1 commands.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, readdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {Miniflare, convertV4MiniflareOptions} from 'miniflare';
import {loadCurrentFixture, currentWasmSha256, sha256} from '../../tools/current-native-source.mjs';
import {catalog} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {assertPublicView, assertSessionJournal} from './assert-session-journal.mjs';

const native = loadCurrentFixture('docs/evidence/abi94-platform-verification-20261022/room.current.json.gz');
assert.equal(native.format, 'abi94-workspace-repair-room-v1');
assert.equal(native.scenarios.length, 3);
const worker = path.dirname(process.env.CURRENT_CLOCK_WORKER || path.resolve('dist/current-clock/worker.js'));
const wasm = readdirSync(worker).filter(n => n.endsWith('.wasm'));
assert.equal(wasm.length, 1);
assert.equal(sha256(readFileSync(path.join(worker, wasm[0]))), currentWasmSha256);

for (const scenario of native.scenarios) test(`ABI94 repaired Room: ${scenario.name}`, async t => {
  const persistence = mkdtempSync(path.join(tmpdir(), 'abi94-repair-d1-'));
  const options = convertV4MiniflareOptions({name: 'abi94-repair', resourcePersistencePath: persistence,
    modulesRoot: worker, modules: [{type: 'ESModule', path: path.join(worker, 'worker.js')},
      {type: 'CompiledWasm', path: path.join(worker, wasm[0])}], cf: false,
    compatibilityDate: '2026-10-02', bindings: {LOCAL_TEST_CLOCK_MS: '0'},
    d1Databases: {DB: 'abi94-repair'}});
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) {
      await db.prepare(sql).run();
    }
  }
  const health = await mf.dispatchFetch('http://local.test/api/health');
  assert.equal(health.status, 200);
  assert.equal((await health.json()).engineVersion, catalog.engineVersion);
  const catalogue = await mf.dispatchFetch('http://local.test/api/catalog');
  assert.equal(catalogue.status, 200);
  const {transport, entryIdempotency, ...actualCatalog} = await catalogue.json();
  assert.deepEqual(actualCatalog, catalog);
  const first = scenario.initial.views[0], id = first.roomId;
  const tokens = first.players.map((_, seat) => String(seat + 1).repeat(64));
  const store = () => new RoomStore(db);
  await store().create({id, invite: first.inviteCode, state: scenario.initial.state,
    nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]), requestHash: crypto.randomUUID(),
    intentHash: 'fresh-native94-workspace-repair', response: '{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(first.version, id).run();
  for (let seat = 1; seat < tokens.length; seat++) {
    await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  }
  const api = async (operation, seat, command) => {
    const response = await mf.dispatchFetch(`http://local.test/api/rooms/${id}/${operation}`, {
      method: command ? 'POST' : 'GET', headers: {Authorization: 'Bearer ' + tokens[seat],
        ...(command ? {'Content-Type': 'application/json'} : {})},
      ...(command ? {body: JSON.stringify(command)} : {}),
    });
    return {status: response.status, body: await response.json()};
  };
  const receipts = [];
  let accepted = 0, rejected = 0, projections = 0, reopened = 0;
  const boundaries = new Set();
  for (const step of scenario.steps) {
    assert.equal((await store().room(id)).state, step.state);
    const result = await api('commands', step.seat, step.command);
    if (step.expected.errorCode) {
      assert(result.status >= 400);
      assert.equal(result.body.error, step.expected.errorCode);
      assert.equal(await store().receipt(id, step.command.commandId), null);
      rejected++;
    } else {
      assert.equal(result.status, 200, JSON.stringify(result.body));
      assertPublicView(result.body, step.expected.view);
      const row = await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id, step.expected.version).first();
      assertSessionJournal(row.entry, step.expected.journal);
      receipts.push({step, result}); accepted++;
    }
    assert.equal((await store().room(id)).state, step.expected.state);
    for (let seat = 0; seat < tokens.length; seat++) {
      const view = await api('state', seat);
      assert.equal(view.status, 200); assertPublicView(view.body, step.views[seat]); projections++;
    }
    const kind = step.command.action.kind;
    if (['pauseRoom', 'resumeRoom', 'submitResponse'].includes(kind) && !boundaries.has(kind)) {
      boundaries.add(kind);
      const saved = await store().room(id);
      await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB'); reopened++;
      assert.deepEqual(await store().room(id), saved);
    }
  }
  const saved = await store().room(id);
  assert(saved.state.includes('"seed":18446744073709551488'));
  await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB'); reopened++;
  assert.deepEqual(await store().room(id), saved);
  for (const {step, result} of [receipts[0], receipts.at(-1)]) {
    assert.deepEqual(await api('commands', step.seat, step.command), result);
    assert.deepEqual(await store().room(id), saved);
  }
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n, accepted);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM journal WHERE room_id=?').bind(id).first()).n, accepted);
  t.diagnostic(JSON.stringify({scenario: scenario.name, accepted, rejected, projections, reopened,
    opaqueD1StateEqual: true, journalEqual: true, originalReceiptsAfterReopen: true,
    wasmSha256: currentWasmSha256, scope: 'Local HTTP/D1, not browser or deployment'}));
});
