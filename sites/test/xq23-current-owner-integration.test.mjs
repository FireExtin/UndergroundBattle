// Fresh Native141/current89 owner elimination in real local HTTP/workerd/D1.
import test from 'node:test';
import assert from 'node:assert/strict';
import {loadOwnerFixture} from '../../tools/xq23-current-owner-oracle.mjs';
import {harness} from './xq23-http-harness.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
test('current89 retains borrowed-owner cleanup, score and original receipts after real D1 reopen',async t=>{
 const f=loadOwnerFixture(),h=await harness(t,f),receipts=[];let accepted=0,rejected=0,projections=0,reopened=0;
 const views=async expected=>{for(const[seat,v]of expected.entries()){const r=await h.api('state',seat);assert.equal(r.status,200);assertPublicView(r.body,v);projections++;}};
 await views(f.initial.views);
 for(const row of f.steps){
  assert.equal((await h.store().room(h.id)).state,row.state);const before=await h.snapshot(),r=await h.api('commands',row.seat,row.command);
  if(row.expected.errorCode){assert.equal(r.status,400);assertPublicView(r.body.view,row.expected.view);assert.deepEqual(await h.snapshot(),before);rejected++;}
  else{assert.equal(r.status,200,JSON.stringify(r.body));assertPublicView(r.body,row.expected.view);receipts.push({row,body:r.body});accepted++;
   const j=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,row.expected.version).first();assertSessionJournal(j.entry,row.expected.journal);
  }
  assert.equal((await h.store().room(h.id)).state,row.expected.state);await views(row.views);
  const old=JSON.parse(row.state).game,next=JSON.parse(row.expected.state).game;
  if(!row.expected.errorCode&&(!old.players[2].eliminated&&next.players[2].eliminated||row.command.action.kind==='game'&&['play','activate'].includes(row.command.action.action.kind))){await h.reopen();reopened++;await views(row.views);}
 }
 const final=await h.snapshot();assert.equal(final.commands.length,165);assert.equal(final.journal.length,165);
 await h.reopen();reopened++;
 for(const{row,body}of receipts){const r=await h.api('commands',row.seat,row.command);assert.equal(r.status,200);assert.deepEqual(r.body,body);assert.deepEqual(await h.snapshot(),final);}
 const game=JSON.parse(final.room.state).game;assert(game.players.flatMap(p=>p.assets).every(c=>c.owner!==2));assert.equal(game.players[2].hand.length,0);assert(game.players[3].hand.some(c=>c.definition==='XQ23'));
 assert.equal(accepted,165);assert.equal(rejected,1);assert.equal(f.steps.at(-1).views[0].players[2].score,4);assert.equal(f.steps.at(-1).views[0].players[3].score,2);
 t.diagnostic(JSON.stringify({source:'fresh actual141-input Native89',accepted,rejected,projections,reopened,allOriginalReceiptsUnchanged:true,syntheticDatabaseRetained:true}));
});
