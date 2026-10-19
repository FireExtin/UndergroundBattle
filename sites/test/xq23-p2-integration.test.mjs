// Focused P2 qualification: exact old paid-state mutation and new bound Native trace.
import test from 'node:test';
import assert from 'node:assert/strict';
import {loadFixture} from '../../tools/xq23-oracle.mjs';
import {loadP2Fixture,reviewSummaryCases} from '../../tools/xq23-p2-oracle.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
import {harness} from './xq23-http-harness.mjs';
const original=loadFixture(),p2=loadP2Fixture(),boundary=reviewSummaryCases();
const reference=original.scenarios.find(s=>s.name==='actor-0').steps[0].views;

for(const bad of boundary.invalid)test('XQ23 P2 exact paid-state '+bad.name+' rejects HTTP read/command/reopen without writes',async t=>{
 const expectedError=original.invalid.find(x=>x.name===bad.name).roomError;
 const h=await harness(t,{initial:{state:bad.state,views:reference}}),before=await h.snapshot();
 assert.equal(before.commands.length,0);assert.equal(before.journal.length,0);
 let rejected=0;
 for(let reopened=0;reopened<2;reopened++){
  if(reopened)await h.reopen();
  for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,400);assert.equal(r.body.message,expectedError);rejected++;}
  const r=await h.api('commands',0,{commandId:'p2-exact-'+bad.name,expectedVersion:9,action:{kind:'pauseRoom'}});
  assert.equal(r.status,400);assert.equal(r.body.message,expectedError);rejected++;
  assert.deepEqual(await h.snapshot(),before);
 }
 t.diagnostic(JSON.stringify({exactOriginalPaidStateSha256:'76a47a29fb9d3b1d772ffdb6087e9180cac93f91fdd0c3af0d989fdc36b48c65',singleField:bad.changedPath,http400:rejected,realD1Reopen:true,noRoomCommandOrJournalWrites:true}));
});

test('XQ23 P2 real borrowed-owner elimination keeps Native state, resources, score and live-owner receipts across HTTP/D1',async t=>{
 const h=await harness(t,p2),receipts=[];let accepted=0,rejected=0,reopened=0,projections=0;
 const views=async expected=>{for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,200);assertPublicView(r.body,expected[seat]);projections++;}};
 const initial=await h.snapshot();await views(p2.initial.views);assert.deepEqual(await h.snapshot(),initial);await h.reopen();reopened++;
 for(const[index,row]of p2.steps.entries()){
  assert.equal((await h.store().room(h.id)).state,row.state);
  const before=await h.snapshot(),result=await h.api('commands',row.seat,row.command);
  if(row.expected.errorCode){assert.equal(result.status,row.expected.errorCode==='invalid_action'?400:409);assert.equal(result.body.error,row.expected.errorCode);assertPublicView(result.body.view,row.expected.view);assert.deepEqual(await h.snapshot(),before);rejected++;}
  else{
   assert.equal(result.status,200,'command '+index+': '+JSON.stringify(result.body));assertPublicView(result.body,row.expected.view);receipts.push({row,body:result.body});accepted++;
   const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,row.expected.version).first();assertSessionJournal(journal.entry,row.expected.journal);
  }
  assert.equal((await h.store().room(h.id)).state,row.expected.state);await views(row.views);
  const prev=JSON.parse(row.state).game,next=JSON.parse(row.expected.state).game;
  const oldAssets=new Set(prev.players[0].assets.map(c=>c.id));
  const converted=next.players[0].assets.some(c=>c.definition==='XQ23'&&!oldAssets.has(c.id));
  const eliminated=!prev.players[2].eliminated&&next.players[2].eliminated;
  const paid=row.command.action.kind==='game'&&['play','activate'].includes(row.command.action.action.kind);
  if(!row.expected.errorCode&&(paid||converted||eliminated)){
   await h.reopen();reopened++;await views(row.views);
   if(eliminated){assert(next.players.flatMap(p=>p.assets).every(c=>c.owner!==2));assert.equal(next.players[0].assets.length,10);assert.equal(row.views[0].players[2].score,4);assert.equal(row.views[0].players[3].score,2);}
  }
 }
 const final=await h.snapshot();assert.equal(final.commands.length,accepted);assert.equal(final.journal.length,accepted);
 await h.reopen();reopened++;
 for(const {row,body}of receipts){const retry=await h.api('commands',row.seat,row.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,body);assert.deepEqual(await h.snapshot(),final);}
 const game=JSON.parse(final.room.state).game;
 assert.equal(game.players[2].hand.length,0);assert(game.players[3].hand.some(c=>c.definition==='XQ23'));assert(game.players.flatMap(p=>p.assets).every(c=>c.owner!==2));
 assert.equal(accepted,165);assert.equal(rejected,1);
 t.diagnostic(JSON.stringify({source:'actual139-input Native88P2',accepted,rejected,projections,reopened,completeOpaqueStatesAndJournals:true,scorePreserved:'4+2',deadOwnerAssetsAndHandAbsent:true,liveOwnerReturnSucceeded:true,allOriginalReceiptsRetriedWithoutWrites:true,localSyntheticDatabaseRetained:true}));
});
