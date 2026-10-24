import { loadCurrentFixture } from '../../tools/current-native-source.mjs';
import { assertSessionJournal, assertPublicView } from './assert-session-journal.mjs';
// JC046 continuous prepared Native93 traces through the production service and real D1.
// Native fixture IDs use RoomService; the ordinary HTTP suite covers routing.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { kernel as abi, moduleBytes as currentBytes } from '../../rust-game-wasm/tests/current-kernel.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';

abi.initSync({ module: currentBytes });
const fixtures = loadCurrentFixture('rust-game-wasm/tests/current111/jc046.json.gz');
const catalog=JSON.parse(abi.catalog());
assert.deepEqual(catalog,fixtures.catalog);
assert.equal(catalog.engineVersion,'rust-v0.2.111-import20-rulings-complete');
assert.equal(catalog.cardPoolVersion,'limited-v2.83-batch20-next');
assert.equal(catalog.cards.length,193);
assert.equal(fixtures.scenarios.length,37);
assert.equal(new Set(fixtures.scenarios.map(s=>s.name)).size,37);

function reopenCheckpoint(view) {
  return view.pause ? 'paused' : view.regions.some(r => r.cardId === '') ? 'vacant'
    : view.step.includes(':win') ? 'win' : view.pendingChoice ? 'choice'
    : view.stack.length ? 'stack' : null;
}

for (const scenario of fixtures.scenarios) test(`JC046 ABI93 ${scenario.name}: continuous Native commands, journals, D1 reopening and receipts`, async t => {
  const persist = mkdtempSync(join(tmpdir(), 'jc046-77-d1-'));
  const options = convertV4MiniflareOptions({ name: 'jc046-77', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(n => n.endsWith('.wasm')).map(n => ({ type: 'CompiledWasm', path: resolve('dist/server', n) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'jc046-77' } });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  for (const file of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const first = scenario.steps[0], initial = JSON.parse(abi.view(first.state, 0)), id = initial.roomId;
  const tokens = initial.players.map((_, seat) => String(seat + 1).repeat(64)); // Test-only seat secrets.
  let now = 0, store = new RoomStore(db), service = new RoomService(db, routeKernels(abi), () => now);
  await store.create({ id, invite: initial.inviteCode, state: first.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]),
    requestHash: crypto.randomUUID(), intentHash: 'explicit-jc046-native-layout', response: '{}' });
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
    const key = reopenCheckpoint(step.expected.view);
    if (key && !reopenedStates.has(key)) {
      reopenedStates.add(key);
      await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
      store = new RoomStore(db); service = new RoomService(db, routeKernels(abi), () => now);
      assert.equal((await store.room(id)).state, step.expected.state);
      reopened++;
    }
  }
  const saved = await store.room(id);
  if (lastAccepted) {
    now = 600000;
    assert.deepEqual(await service.command(id, 'Bearer ' + tokens[lastAccepted.seat], lastAccepted.command), lastView);
    await assert.rejects(service.command(id, 'Bearer ' + tokens[lastAccepted.seat], { ...lastAccepted.command, expectedVersion: lastAccepted.command.expectedVersion + 1 }), e => e.status === 409);
    assert.deepEqual(await store.room(id), saved);
  } else {
    // Payment/loyalty/payload rejection-only scenarios must create no receipt or journal.
    assert.equal(accepted,0);
    assert.equal(rejected,scenario.steps.length);
    assert.equal(saved.state,scenario.initial.state);
    assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM journal WHERE room_id=?').bind(id).first()).n,0);
  }
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n, accepted);
  await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');
  store=new RoomStore(db);service=new RoomService(db,routeKernels(abi),()=>0);
  assert.deepEqual(await store.room(id),saved);reopened++;
  let duplicateOriginalReceipt=false;
  if (firstAccepted) {
    const duplicate=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/commands`,{method:'POST',headers:{'Content-Type':'application/json',Authorization:'Bearer '+tokens[firstAccepted.seat]},body:JSON.stringify(firstAccepted.command)});
    assert.equal(duplicate.status,200);assertPublicView(await duplicate.json(),firstAccepted.expected.view);
    assert.deepEqual(await store.room(id),saved);
    duplicateOriginalReceipt=true;
  }
  t.diagnostic(JSON.stringify({ preparedNativeLayout: true, accepted, rejected, reopened, fullJournalsEqual: true, opaqueStatesEqual: true, seatViewsEqual: true, rejectedOnlyScenario: accepted===0, duplicateOriginalReceipt }));
});

test('JC046 mandatory sacrifice: pauseRoom, paused D1 reopen, resumeRoom and all four choices continue', async t => {
  const scenario=fixtures.scenarios.find(row=>row.name==='paid-seat0');
  const choiceIndex=scenario.steps.findIndex(step=>step.command.action.action?.kind==='choose'&&!step.expected.errorCode);
  assert(choiceIndex>0);
  const persist=mkdtempSync(join(tmpdir(),'jc046-pause-77-d1-'));
  const options=convertV4MiniflareOptions({name:'jc046-pause-77',resourcePersistencePath:persist,modules:[
    {type:'ESModule',path:resolve('dist/server/index.js')},
    ...readdirSync('dist/server').filter(name=>name.endsWith('.wasm')).map(name=>({type:'CompiledWasm',path:resolve('dist/server',name)})),
  ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'jc046-pause-77'}});
  let mf=new Miniflare(options),db=await mf.getD1Database('DB');
  t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
  for(const file of readdirSync('drizzle').filter(name=>name.endsWith('.sql'))) {
    for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(sql=>sql.trim()))await db.prepare(sql).run();
  }
  const initial=JSON.parse(abi.view(scenario.initial.state,0)),id=initial.roomId;
  const tokens=initial.players.map((_,seat)=>String(seat+1).repeat(64)); // Offline test seats only.
  const kernel=routeKernels(abi);
  let now=0,store=new RoomStore(db),service=new RoomService(db,kernel,()=>now),accepted=0,reopened=0;
  await store.create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),
    requestHash:crypto.randomUUID(),intentHash:'explicit-jc046-native-pause-layout',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
  for(let seat=1;seat<tokens.length;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  const snapshot=async()=>({room:await store.room(id),
    commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY command_id').bind(id).all()).results,
    journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
  const execute=async(seat,command)=>{
    const before=await store.room(id);
    const expected=JSON.parse(abi.applyRoom(before.state,seat,JSON.stringify(command),String(now)));
    assert.equal(expected.outcome,'accepted');
    const view=await service.command(id,'Bearer '+tokens[seat],command);
    assert.deepEqual(view,expected.view);
    assert.equal((await store.room(id)).state,expected.state);
    const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();
    assertSessionJournal(journal.entry, expected.journal);
    for(let viewer=0;viewer<tokens.length;viewer++)assert.deepEqual(await service.state(id,'Bearer '+tokens[viewer],null),JSON.parse(abi.view(expected.state,viewer)));
    accepted++;return expected;
  };
  const reopen=async()=>{
    const saved=await snapshot();await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');
    store=new RoomStore(db);service=new RoomService(db,kernel,()=>now);assert.deepEqual(await snapshot(),saved);reopened++;
  };
  for(const step of scenario.steps.slice(0,choiceIndex)) {
    now=Number(step.serverNowMs);assert.equal((await store.room(id)).state,step.state);
    assert.deepEqual(await execute(step.seat,step.command),step.expected);
  }
  const originalChoice=JSON.parse(abi.view((await store.room(id)).state,0)).pendingChoice;
  assert.deepEqual(originalChoice,scenario.steps[choiceIndex-1].views[0].pendingChoice);
  assert.equal(originalChoice.title,'选择一个自己操控的角色牺牲');
  assert.equal(originalChoice.min,1);assert.equal(originalChoice.max,1);assert.equal(originalChoice.allowDecline,false);
  const pauseCommand={commandId:'jc046-mandatory-pause',expectedVersion:(await store.room(id)).version,action:{kind:'pauseRoom'}};
  const paused=await execute(0,pauseCommand);
  assert.deepEqual(paused.view.pause,{pausedAtMs:now,pausedBy:0});
  assert.equal(reopenCheckpoint(paused.view),'paused'); // Regresses if the selector uses the nonexistent `paused` field.
  assert.deepEqual(paused.view.pendingChoice,originalChoice);
  const frozen=await snapshot();now=86_400_000;
  await reopen();
  const restored=await service.state(id,'Bearer '+tokens[0],null);
  assert.deepEqual(restored.pause,paused.view.pause);assert.deepEqual(restored.pendingChoice,originalChoice);
  assert.deepEqual(await service.command(id,'Bearer '+tokens[0],pauseCommand),paused.view);
  const rejected={...scenario.steps[choiceIndex].command,commandId:'jc046-choice-while-paused',expectedVersion:paused.version};
  await assert.rejects(service.command(id,'Bearer '+tokens[0],rejected),error=>error.code==='room_paused');
  assert.equal(await store.receipt(id,rejected.commandId),null);assert.deepEqual(await snapshot(),frozen);
  const resumeCommand={commandId:'jc046-mandatory-resume',expectedVersion:paused.version,action:{kind:'resumeRoom'}};
  const resumed=await execute(0,resumeCommand);
  assert.equal(resumed.view.pause,undefined);assert.deepEqual(resumed.view.pendingChoice,originalChoice);
  await reopen();
  const resumedSnapshot=await snapshot();
  assert.deepEqual(await service.command(id,'Bearer '+tokens[0],resumeCommand),resumed.view);
  assert.deepEqual(await snapshot(),resumedSnapshot);
  // Added session commands change revision, time and pause/resume log entries, not the Native game projection.
  const gameProjection=view=>Object.fromEntries(Object.entries(view).filter(([key])=>!['version','serverNowMs','log'].includes(key)));
  const choiceSeats=[];
  for(const step of scenario.steps.slice(choiceIndex)) {
    assert.equal(step.command.action.action.kind,'choose');choiceSeats.push(step.seat);
    const command={...step.command,commandId:'paused-'+step.command.commandId,expectedVersion:(await store.room(id)).version};
    const result=await execute(step.seat,command);
    for(let seat=0;seat<tokens.length;seat++)assert.deepEqual(gameProjection(JSON.parse(abi.view(result.state,seat))),gameProjection(step.views[seat]));
  }
  assert.deepEqual(choiceSeats,[0,1,2,3]);
  const completed=await service.state(id,'Bearer '+tokens[0],null);assert.equal(completed.pendingChoice,null);assert.equal(completed.pause,undefined);
  const final=await snapshot();assert.equal(final.commands.length,scenario.steps.length+2);assert.equal(final.journal.length,accepted);
  await reopen();
  for(const [command,view] of [[pauseCommand,paused.view],[resumeCommand,resumed.view]]) {
    assert.deepEqual(await service.command(id,'Bearer '+tokens[0],command),view);
  }
  assert.deepEqual(await snapshot(),final);
  t.diagnostic(JSON.stringify({preparedNativeLayout:true,nativePrefixCommands:choiceIndex,accepted,pausedD1Reopened:true,
    resumedD1Reopened:true,reopened,mandatoryChoicePreserved:true,choiceSeats,pausedChoiceRejectedWithoutWrite:true,
    completeKernelStatesAndJournalsEqual:true,nativeGameProjectionsEqual:true,duplicatePauseResumeReceipts:true}));
});
