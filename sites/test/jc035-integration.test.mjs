// Actual local HTTP/workerd/thirteen-WASM/D1; explicit Native89 layouts, not live rooms.
import test from 'node:test';
import assert from 'node:assert/strict';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
import {loadFixture} from '../../tools/jc035-oracle.mjs';
import {harness} from './xq23-http-harness.mjs';
const fixture=loadFixture();

for(const scenario of fixture.scenarios)test(`JC035 ABI89 HTTP ${scenario.name}`,async t=>{
  const h=await harness(t,scenario);let accepted=0,rejected=0,projections=0;
  const receipts=[];
  const viewAll=async views=>{for(let seat=0;seat<views.length;seat++){
    const result=await h.api('state',seat);assert.equal(result.status,200);assertPublicView(result.body,views[seat]);projections++;
  }};
  const before=await h.snapshot();await viewAll(scenario.initial.views);assert.deepEqual(await h.snapshot(),before);
  await h.reopen();await viewAll(scenario.initial.views);assert.deepEqual(await h.snapshot(),before);
  for(const row of scenario.steps) {
    assert.equal((await h.store().room(h.id)).state,row.state);
    const saved=await h.snapshot(),result=await h.api('commands',row.seat,row.command);
    if(row.expected.errorCode) {
      assert.equal(result.status,row.expected.errorCode==='invalid_action'?400:409);assert.equal(result.body.error,row.expected.errorCode);
      assertPublicView(result.body.view,row.expected.view);assert.deepEqual(await h.snapshot(),saved);rejected++;
    } else {
      assert.equal(result.status,200,JSON.stringify(result.body));assertPublicView(result.body,row.expected.view);
      receipts.push({row,body:result.body});accepted++;
    }
    assert.equal((await h.store().room(h.id)).state,row.expected.state);
    if(row.expected.changed) {
      const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,row.expected.version).first();
      assertSessionJournal(journal.entry,row.expected.journal);
    }
    await viewAll(row.views);
    // Reopen every accepted command: paid response frames,
    // pause/resume and post-resolution state all cross real D1 reopen.
    if(!row.expected.errorCode){await h.reopen();await viewAll(row.views);}
  }
  const final=await h.snapshot();assert.equal(final.commands.length,accepted);
  await h.reopen();
  for(const {row,body}of receipts) {
    const retry=await h.api('commands',row.seat,row.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,body);
    assert.deepEqual(await h.snapshot(),final);
  }
  t.diagnostic(JSON.stringify({scenario:scenario.name,reference:'compiler-bound Native89',accepted,rejected,projections,
    originalReceiptsAfterReopen:true,opaqueStatesAndJournalsEqual:true,localSyntheticDatabaseRetained:true}));
});

for(const invalid of fixture.invalid)test(`JC035 corrupted ${invalid.name} restores HTTP400 with no D1 writes`,async t=>{
 const reference=fixture.scenarios[0].initial.views;
 const scenario={name:'invalid-'+invalid.name,initial:{state:invalid.state,views:reference.map(v=>({...v,roomId:invalid.roomId,inviteCode:invalid.inviteCode,version:invalid.revision}))}};
 const h=await harness(t,scenario),before=await h.snapshot();
 const command={commandId:'invalid-'+invalid.name,expectedVersion:invalid.revision,action:{kind:'pauseRoom'}};
 let rejected=0;
 for(let reopen=0;reopen<2;reopen++){
  if(reopen)await h.reopen();
  for(let seat=0;seat<reference.length;seat++){const r=await h.api('state',seat);assert.equal(r.status,400);assert.equal(r.body.message,invalid.roomError);rejected++;}
  const r=await h.api('commands',0,command);assert.equal(r.status,400);assert.equal(r.body.message,invalid.roomError);rejected++;
  assert.deepEqual(await h.snapshot(),before);
 }
 assert.equal(before.commands.length,0);assert.equal(before.journal.length,0);
 t.diagnostic(JSON.stringify({invalid:invalid.name,http400:rejected,realD1Reopen:true,roomCommandsJournalUnchanged:true}));
});

for(const restore of fixture.legalRestores)test(`JC035 P2 legal restore ${restore.name}`,async t=>{
 const h=await harness(t,{name:restore.name,initial:{state:restore.state,views:restore.views}}),before=await h.snapshot();
 for(let reopen=0;reopen<2;reopen++){
  if(reopen)await h.reopen();
  for(let seat=0;seat<restore.views.length;seat++){
   const result=await h.api('state',seat);assert.equal(result.status,200,restore.name);
   assertPublicView(result.body,restore.views[seat]);
  }
  assert.deepEqual(await h.snapshot(),before,restore.name+' restore is read-only and does not pump');
 }
 if(restore.continuation){
  const r=restore.continuation,result=await h.api('commands',r.seat,r.command);
  assert.equal(result.status,200,JSON.stringify(result.body));assertPublicView(result.body,r.expected.view);
  assert.equal((await h.store().room(h.id)).state,r.expected.state);
  const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,r.expected.version).first();
  assertSessionJournal(journal.entry,r.expected.journal);
  const after=await h.snapshot();await h.reopen();
  const retry=await h.api('commands',r.seat,r.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,result.body);
  assert.deepEqual(await h.snapshot(),after);
 }
 t.diagnostic(JSON.stringify({restore:restore.name,validBeforeAndAfterD1Reopen:true,readOnlyLoadNoPump:true,
  realPendingContinuation:!!restore.continuation,localSyntheticDatabaseRetained:true}));
});
