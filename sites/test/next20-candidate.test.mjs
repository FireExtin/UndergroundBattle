import { loadCurrentFixture } from '../../tools/current-native-source.mjs';
import { assertSessionJournal, assertPublicView } from './assert-session-journal.mjs';
// JC037 continuous prepared Native93 traces through the production service and real D1.
// Native fixture IDs use RoomService; the ordinary HTTP suite covers routing.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { kernel as abi, moduleBytes as currentBytes } from '../../rust-game-wasm/tests/current-kernel.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';

abi.initSync({ module: currentBytes });
const fixtures = process.env.HEGEMONY_CANDIDATE_NATIVE_FIXTURES ? loadCurrentFixture('rust-game-wasm/tests/current112/next20-candidate-room.json.gz') : {catalog:JSON.parse(abi.catalog()),cases:[]};
assert.deepEqual(JSON.parse(abi.catalog()),fixtures.catalog);

for (const scenario of fixtures.cases) test(`Internal closed next20 ${scenario.name}: continuous Native commands, journals, D1 reopening and receipts`, async t => {
  const persist = mkdtempSync(join(tmpdir(), 'next20-internal-d1-'));
  const options = convertV4MiniflareOptions({ name: 'next20-internal', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(n => n.endsWith('.wasm')).map(n => ({ type: 'CompiledWasm', path: resolve('dist/server', n) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'next20-internal' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const first = scenario.steps[0], initial = JSON.parse(abi.view(first.state, 0)), id = initial.roomId;
  const tokens = initial.players.map((_, seat) => String(seat + 1).repeat(64)); // Test-only seat secrets.
  let now = 0, store = new RoomStore(db), service = new RoomService(db, routeKernels(abi), () => now);
  await store.create({ id, invite: initial.inviteCode, state: first.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'explicit-next20-native-layout', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  for (let seat = 1; seat < tokens.length; seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, seat, await digest(tokens[seat])).run();
  let accepted = 0, rejected = 0, reopened = 0, lastAccepted, lastView;
  const reopenedStates = new Set();
  let firstAccepted;
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
      assertSessionJournal(journal.entry, step.expected.journal);
      firstAccepted ||= step; accepted++; lastAccepted = step; lastView = view;
    }
    assert.equal((await store.room(id)).state, step.expected.state, step.inputName);
    for (let seat = 0; seat < tokens.length; seat++) assert.deepEqual(await service.state(id, 'Bearer ' + tokens[seat], null), step.views[seat]);
    const keys = [step.expected.view.pause && 'paused', step.expected.view.regions.some(r => r.cardId === '') && 'vacant',
      step.expected.view.step.includes(':win') && 'win', step.expected.view.pendingChoice && 'choice',
      step.expected.view.stack.length && 'stack'].filter(Boolean);
    if (keys.some(key => !reopenedStates.has(key))) {
      keys.forEach(key => reopenedStates.add(key));
      await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
      store = new RoomStore(db); service = new RoomService(db, routeKernels(abi), () => now);
      assert.equal((await store.room(id)).state, step.expected.state);
      reopened++;
    }
  }
  if (scenario.steps.some(step => step.expected.view.pause)) assert(reopenedStates.has('paused'));

  const saved = await store.room(id);
  now = 600000;
  assert.deepEqual(await service.command(id, 'Bearer ' + tokens[lastAccepted.seat], lastAccepted.command), lastView);
  await assert.rejects(service.command(id, 'Bearer ' + tokens[lastAccepted.seat], { ...lastAccepted.command, expectedVersion: lastAccepted.command.expectedVersion + 1 }), e => e.status === 409);
  assert.deepEqual(await store.room(id), saved);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n, accepted);
  await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');
  store=new RoomStore(db);service=new RoomService(db,routeKernels(abi),()=>0);
  assert.deepEqual(await store.room(id),saved);reopened++;
  const duplicate=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/commands`,{method:'POST',headers:{'Content-Type':'application/json',Authorization:'Bearer '+tokens[firstAccepted.seat]},body:JSON.stringify(firstAccepted.command)});
  assert.equal(duplicate.status,200);assertPublicView(await duplicate.json(),firstAccepted.expected.view);
  assert.deepEqual(await store.room(id),saved);
  t.diagnostic(JSON.stringify({ preparedNativeLayout: true, accepted, rejected, reopened, fullJournalsEqual: true, opaqueStatesEqual: true, seatViewsEqual: true, duplicateOriginalReceipt: true }));
});

// Public entry is separate from the explicitly prepared internal cases above.
test('public closed next20: real HTTP create/join rejection, original receipts and D1 pause/reopen/resume',async t=>{
  const persist=mkdtempSync(join(tmpdir(),'next20-public-'));
  const worker=dirname(process.env.CURRENT_CLOCK_WORKER);
  const options=convertV4MiniflareOptions({name:'next20-public',resourcePersistencePath:persist,modulesRoot:worker,modules:[
    {type:'ESModule',path:join(worker,'worker.js')},...readdirSync(worker).filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:join(worker,n)}))],
    compatibilityDate:'2026-10-02',cf:false,bindings:{LOCAL_TEST_CLOCK_MS:'0'},d1Databases:{DB:'next20-public'}});
  let mf=new Miniflare(options),db=await mf.getD1Database('DB');
  t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
  for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const api=async(route,body,session)=>{
    const r=await mf.dispatchFetch('http://localhost'+route,{method:body?'POST':'GET',headers:{...(body?{'Content-Type':'application/json'}:{}),...(session?{Authorization:'Bearer '+session.token}:{})},...(body?{body:JSON.stringify(body)}:{})});
    return {status:r.status,body:await r.json()};
  };
  const catalog=(await api('/api/catalog')).body;
  assert.equal(catalog.cards.length,327);assert.equal(catalog.cards.filter(c=>c.supported).length,236);
  const draft=()=>({id:'next20-public',name:'Public control',description:'local synthetic entry',societyId:null,
    cards:structuredClone(catalog.decks.find(d=>d.id==='watchers').cards),rulesVersion:catalog.rulesVersion,
    cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:'2026-10-25T00:00:00Z'});
  const create={name:'Public P0',mode:'duel',deckDraft:draft(),requestId:crypto.randomUUID()};
  const created=await api('/api/rooms',create);assert.equal(created.status,200);assert.deepEqual(await api('/api/rooms',create),created);
  const counts=async()=>Promise.all(['rooms','seats','entry_receipts'].map(async table=>(await db.prepare(`SELECT COUNT(*) AS n FROM ${table}`).first()).n));
  const before=await counts(),original=(await new RoomStore(db).room(created.body.roomId)).state;
  for(const id of ['BQ033','BQ009','BQ001']){
    assert.equal(catalog.cards.find(c=>c.id===id).supported,false);
    const bad=draft();bad.cards.push({cardId:id,count:1});
    assert.equal((await api('/api/rooms',{...create,deckDraft:bad,requestId:crypto.randomUUID()})).status,400);
    assert.equal((await api('/api/rooms/join',{name:'Rejected',inviteCode:created.body.inviteCode,deckDraft:bad,requestId:crypto.randomUUID()})).status,400);
    assert.deepEqual(await counts(),before);assert.equal((await new RoomStore(db).room(created.body.roomId)).state,original);
  }
  const joinBody={name:'Public P1',inviteCode:created.body.inviteCode,deckDraft:draft(),requestId:crypto.randomUUID()};
  const joined=await api('/api/rooms/join',joinBody);assert.equal(joined.status,200);assert.deepEqual(await api('/api/rooms/join',joinBody),joined);
  const sessions=[created.body,joined.body];const route=(seat,op)=>`/api/rooms/${sessions[seat].roomId}/${op}`;
  const views=async()=>Promise.all(sessions.map(async(session,seat)=>{const r=await api(route(seat,'state'),null,session);assert.equal(r.status,200);return r.body;}));
  let all=await views(),first;
  const command=async(seat,action)=>{
    const cmd={commandId:crypto.randomUUID(),expectedVersion:all[seat].version,action};
    const result=await api(route(seat,'commands'),cmd,sessions[seat]);assert.equal(result.status,200);first||={seat,cmd,result};all=await views();return {seat,cmd,result};
  };
  for(const seat of [0,1])await command(seat,{kind:'game',action:{kind:'ready'}});
  await command(0,{kind:'game',action:{kind:'start'}});
  for(let n=0;n<4;n++){
    const seat=all.findIndex(v=>v.pendingChoice);if(seat<0)break;
    await command(seat,{kind:'game',action:{kind:'choose',choiceId:all[seat].pendingChoice.id,selected:[]}});
  }
  await command(0,{kind:'pauseRoom'});assert(all.every(v=>v.pause));
  const state=(await new RoomStore(db).room(created.body.roomId)).state,saved=all;
  await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');
  assert.equal((await new RoomStore(db).room(created.body.roomId)).state,state);assert.deepEqual(await views(),saved);
  assert.deepEqual(await api('/api/rooms',create),created);
  assert.deepEqual(await api(route(first.seat,'commands'),first.cmd,sessions[first.seat]),first.result);
  assert.equal((await new RoomStore(db).room(created.body.roomId)).state,state);
  await command(0,{kind:'resumeRoom'});assert(all.every(v=>!v.pause));
  t.diagnostic(JSON.stringify({publicAdmission:false,closedCardsRejectedAtCreateAndJoin:3,originalEntryAndCommandReceipts:true,opaquePausedStateRestored:true,players:2}));
});
