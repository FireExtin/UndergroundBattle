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
const { loadFixture } = await import(pathToFileURL(root + 'tools/wm062-oracle.mjs').href);
const fixture = loadFixture();
const worker = process.env.HEGEMONY_CURRENT_WORKER_DIR || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'jc054-worker/server');
async function harness(t, scenario) {
  const persist = mkdtempSync(path.join(tmpdir(), 'wm062-http-'));
  const options = convertV4MiniflareOptions({ name: 'wm062-http', resourcePersistencePath: persist, modulesRoot: worker, modules: [
    { type: 'ESModule', path: path.join(worker, 'worker.js') },
    ...readdirSync(worker).filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: path.join(worker, name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'wm062-http' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  for (const file of readdirSync('drizzle').filter(name => name.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(sql => sql.trim())) await db.prepare(sql).run();
  }
  const initial = scenario.initial.views[0], id = initial.roomId;
  const tokens = initial.players.map((_, seat) => String(seat + 1).repeat(64));
  await new RoomStore(db).create({ id, invite: initial.inviteCode, state: scenario.initial.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]), requestHash: crypto.randomUUID(), intentHash: 'offline-native-wm062', response: '{}' });
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
async function runScenario(t, scenario, expectedCards = 146) {
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
  t.diagnostic(JSON.stringify({ scenarioName: scenario.name, reference: scenario.reference || 'compiler-bound Native', expectedRoomCards: expectedCards, accepted, rejected, projections, reopened,
    completeHttpCoreViewsAndReferenceJournalsEqual: true, opaqueD1StatesEqual: true, originalReceiptsAfterReopen: true }));
}
for (const scenario of fixture.scenarios) test(`WM062 ABI80 HTTP ${scenario.name}`, t => runScenario(t, scenario));

for (const name of ['pending-04', 'response-00', 'pending-13']) test(`WM062 ABI80 corrupted ${name} rejects before D1 writes and after reopening`, async t => {
  const index=fixture.negative.findIndex(row=>row.file===name+'.json');
  const invalid=index<0?undefined:fixture.invalid[index];
  // The raw Native input has a known80 tuple and an intentionally invalid
  // continuation or paid play container. Keep the opaque bytes unchanged.
  assert(invalid, name);
  const outer=JSON.parse(invalid.state),reference=fixture.scenarios[0];
  const scenario={name:'invalid-'+name,initial:{state:invalid.state,views:reference.initial.views.map(v=>({...v,version:outer.revision}))}};
  const h=await harness(t,scenario);
  const catalog=await h.api('catalog',0);assert.equal(catalog.status,200);assert.equal(catalog.body.engineVersion,fixture.catalog.engineVersion);
  const snapshot=async()=>({room:await h.store().room(h.id),commands:(await h.db().prepare('SELECT * FROM commands WHERE room_id=?').bind(h.id).all()).results,
    journal:(await h.db().prepare('SELECT * FROM journal WHERE room_id=?').bind(h.id).all()).results});
  const before=await snapshot();
  const reject=async(operation,seat,command)=>{
    const r=await h.api(operation,seat,command);assert.equal(r.status,400);assert.equal(r.body.error,'request_failed');
    assert.equal(r.body.message,invalid.roomError);
    assert.deepEqual(Object.keys(r.body).sort(),['error','message']);assert.deepEqual(await snapshot(),before);
  };
  await reject('state',0);await reject('state',3);
  await reject('commands',0,{commandId:'invalid-wm062-'+name,expectedVersion:outer.revision,action:{kind:'pauseRoom'}});
  await h.reopen();await reject('state',0);
  t.diagnostic(JSON.stringify({invalidNativeName:name,known80Catalog:true,httpRestoreRejections:4,opaqueD1StateUnchanged:true,noCommandOrJournal:true,reopened:true}));
});

test('published v60 ABI78 keeps its exact reducer, private view, D1 state and original receipts through candidate80 Worker', async t => {
  const { loadBq035Fixture } = await import(pathToFileURL(root + 'tools/bq035-oracle.mjs').href);
  const scenario=loadBq035Fixture().scenarios.find(s=>s.name==='wm023-duel-paid-conversion-blood-knight-move');
  assert(scenario); await runScenario(t,scenario,144);
});

test('published v61 ABI79 retains exact JZ46 reducer, opaque state and old receipts through candidate80 Worker',async t=>{
 const {loadFixture:load79}=await import(pathToFileURL(root+'tools/jz46-oracle.mjs').href);
 const scenario=load79().scenarios.find(s=>s.name==='play-fresh-normalization');
 assert(scenario);await runScenario(t,scenario,145);
});
test('published ABI77 kernel-created room retains opaque state and original receipts through candidate80 Worker',async t=>{
 const {kernel:old77}=await import(pathToFileURL(root+'rust-game-wasm/tests/legacy77-kernel.mjs').href);
 const created=JSON.parse(old77.newGame('770000000000000000000001','LOCAL77','duel','P0','watchers','18446744073709551615'));
 const joined=JSON.parse(old77.joinGame(created.state,'P1','hunters'));
 let state=joined.state;
 const initial={state,views:[0,1].map(seat=>JSON.parse(old77.view(state,seat)))};
 const steps=[];
 // The actual retained reducer generates every state; no room-ID or version
 // rewrite of a recorded old opaque state is permitted.
 for(const [seat,kind] of [[0,'ready'],[1,'ready'],[0,'start']]){
  const command={commandId:'retained77-'+kind+'-'+seat,expectedVersion:JSON.parse(state).revision,action:{kind:'game',action:{kind}}};
  const expected=JSON.parse(old77.applyRoom(state,seat,JSON.stringify(command),'0'));
  assert.equal(expected.outcome,'accepted');
  steps.push({state,seat,command,expected,views:[0,1].map(s=>JSON.parse(old77.view(expected.state,s)))});
  state=expected.state;
 }
 await runScenario(t,{name:'retained77-actual-kernel-created-room',reference:'exact published77 WASM',initial,steps},143);
});
