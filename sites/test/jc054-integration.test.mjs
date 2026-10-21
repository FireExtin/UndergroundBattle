// Prepared Native layouts through real HTTP/workerd/WASM/D1; never live user data.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { assertPublicView, assertSessionJournal } from './assert-session-journal.mjs';
const project = fileURLToPath(new URL('../', import.meta.url));
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
const { loadFixture } = await import(pathToFileURL(root + 'tools/jc054-oracle.mjs').href);
const fixture = loadFixture();
const worker = process.env.HEGEMONY_CURRENT_WORKER_DIR || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'jc054-worker/server');
async function harness(t, scenario) {
  const persist = mkdtempSync(path.join(tmpdir(), 'jc054-http-'));
  const options = convertV4MiniflareOptions({ name: 'jc054-http', resourcePersistencePath: persist, modulesRoot: worker, modules: [
    { type: 'ESModule', path: path.join(worker, 'worker.js') },
    ...readdirSync(worker).filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: path.join(worker, name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'jc054-http' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  for (const file of readdirSync('drizzle').filter(name => name.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(sql => sql.trim())) await db.prepare(sql).run();
  }
  const initial = scenario.initial.views[0], id = initial.roomId;
  const tokens = initial.players.map((_, seat) => String(seat + 1).repeat(64));
  await new RoomStore(db).create({ id, invite: initial.inviteCode, state: scenario.initial.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]), requestHash: crypto.randomUUID(), intentHash: 'offline-native-jc054', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  for (let seat = 1; seat < tokens.length; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  return { id, tokens, db: () => db, store: () => new RoomStore(db), reopen: async () => {
    await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
  }, api: async (operation, seat, command) => {
    const response = await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${operation}`, {
      method: command ? 'POST' : 'GET', headers: { Authorization: 'Bearer ' + tokens[seat], ...(command ? { 'Content-Type': 'application/json' } : {}) },
      ...(command ? { body: JSON.stringify(command) } : {}),
    });
    return { status: response.status, body: await response.json() };
  }, globalCatalog: async () => (await mf.dispatchFetch('http://localhost/api/catalog')).json() };
}
async function runScenario(t, scenario, expectedCards = 147) {
  const h = await harness(t, scenario);
  const roomCatalog = await h.api('catalog', 0);
  assert.equal(roomCatalog.status, 200);
  assert.equal(roomCatalog.body.engineVersion, scenario.initial.views[0].versions.engine);
  assert.equal(roomCatalog.body.cards.length, expectedCards);
  assert.equal((await h.globalCatalog()).engineVersion, 'rust-v0.2.92-society-zone-fix');
  let accepted = 0, rejected = 0, reopened = 0, projections = 0;
  const checkpoints = new Set();
  let firstAccepted, lastAccepted, firstReceipt, lastReceipt;
  for (const step of scenario.steps) {
    assert.equal((await h.store().room(h.id)).state, step.state);
    const result = await h.api('commands', step.seat, step.command);
    if (step.expected.errorCode) {
      assert(result.status >= 400); assert.equal(result.body.error, step.expected.errorCode);
      assert.equal(await h.store().receipt(h.id, step.command.commandId), null); rejected++;
    } else {
      assert.equal(result.status, 200, JSON.stringify(result.body));
      assertPublicView(result.body, step.expected.view, scenario.name);
      const journal = await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id, step.expected.version).first();
      assertSessionJournal(journal.entry, step.expected.journal, scenario.name);
      if (!firstAccepted) { firstAccepted = step; firstReceipt = result.body; }
      lastAccepted = step; lastReceipt = result.body; accepted++;
    }
    assert.equal((await h.store().room(h.id)).state, step.expected.state);
    for (let seat = 0; seat < h.tokens.length; seat++) {
      const response = await h.api('state', seat); assert.equal(response.status, 200);
      assertPublicView(response.body, step.views[seat], scenario.name); projections++;
    }
    const view = step.views.find(v => v.pendingChoice) || step.expected.view;
    const key = view.pause ? 'paused' : view.pendingChoice ? 'choice' : view.stack.length ? 'stack' : view.turn > 1 ? 'next-turn' : null;
    if (key && !checkpoints.has(key)) {
      checkpoints.add(key); const saved = await h.store().room(h.id);
      await h.reopen(); reopened++; assert.deepEqual(await h.store().room(h.id), saved);
    }
  }
  const saved = await h.store().room(h.id);
  const receipts = await h.db().prepare('SELECT count(*) AS n FROM commands WHERE room_id=?').bind(h.id).first();
  assert.equal(receipts.n, accepted);
  await h.reopen(); reopened++; assert.deepEqual(await h.store().room(h.id), saved);
  for (const [step, receipt] of [[firstAccepted, firstReceipt], [lastAccepted, lastReceipt]]) if (step) {
    const duplicate = await h.api('commands', step.seat, step.command);
    assert.equal(duplicate.status, 200); assert.deepEqual(duplicate.body, receipt);
    assert.deepEqual(await h.store().room(h.id), saved);
    const conflict = await h.api('commands', step.seat, { ...step.command, expectedVersion: step.command.expectedVersion + 1 });
    assert.equal(conflict.status, 409); assert.deepEqual(await h.store().room(h.id), saved);
  }
  t.diagnostic(JSON.stringify({ scenarioName: scenario.name, reference: scenario.reference || 'compiler-bound Native81', expectedRoomCards: expectedCards, accepted, rejected, projections, reopened,
    completeHttpCoreViewsAndReferenceJournalsEqual: true, opaqueD1StatesEqual: true, originalReceiptsAfterReopen: true }));
}
for (const scenario of fixture.scenarios) test(`JC054 ABI81 HTTP ${scenario.name}`, t => runScenario(t, scenario));


for (const name of ['frame-00','order-00',...fixture.invalid.map(row=>row.name).filter(name=>name.startsWith('paid-stack-')||name.startsWith('successor-'))]) test(`JC054 corrupted ${name} fails before local D1 writes`, async t=>{
  const invalid=fixture.invalid.find(row=>row.name===name);assert(invalid);
  const reference=fixture.scenarios.find(s=>s.name==='paid-deploy'),outer=JSON.parse(invalid.state);
  const scenario={name:'invalid-'+name,initial:{state:invalid.state,views:reference.initial.views.map(v=>({...v,roomId:reference.initial.views[0].roomId,inviteCode:outer.game.invite_code,version:outer.revision}))}};
  const h=await harness(t,scenario),before=await h.store().room(h.id);
  for(let seat=0;seat<4;seat++){const response=await h.api('state',seat);assert.equal(response.status,400);assert.equal(response.body.message,invalid.roomError);}
  const command={commandId:'invalid',expectedVersion:outer.revision,action:{kind:'pauseRoom'}};
  assert.equal((await h.api('commands',0,command)).status,400);
  assert.deepEqual(await h.store().room(h.id),before);assert.equal(await h.store().receipt(h.id,'invalid'),null);
  const journal=await h.db().prepare('SELECT count(*) AS n FROM journal WHERE room_id=?').bind(h.id).first();assert.equal(journal.n,0);
});
