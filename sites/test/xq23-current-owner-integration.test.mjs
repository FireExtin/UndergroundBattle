// Fresh Native93/current93 owner elimination in real local HTTP/workerd/D1.
import test from 'node:test';
import assert from 'node:assert/strict';
import {loadOwnerFixture} from '../../tools/xq23-current-owner-oracle.mjs';
import {loadCurrentFixture} from '../../tools/xq23-current-oracle.mjs';
import {harness} from './xq23-http-harness.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const current=loadCurrentFixture(),summaryCases=current.invalid.filter(c=>c.name.startsWith('target-summary-'));
assert.equal(summaryCases.length,4);
for(const c of summaryCases)test('current93 fresh paid state '+c.name+' rejects reads and commands across D1 reopen',async t=>{
 const reference=current.scenarios[0].initial.views.map(v=>({...v,roomId:c.roomId,inviteCode:c.inviteCode,version:c.revision}));
 const h=await harness(t,{initial:{state:c.state,views:reference}}),before=await h.snapshot();let rejected=0;
 for(let reopened=0;reopened<2;reopened++){
  if(reopened)await h.reopen();
  for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,400);assert.equal(r.body.message,c.roomError);rejected++;}
  const r=await h.api('commands',0,{commandId:'current-'+c.name,expectedVersion:c.revision,action:{kind:'pauseRoom'}});assert.equal(r.status,400);assert.equal(r.body.message,c.roomError);rejected++;
  assert.deepEqual(await h.snapshot(),before);
 }
 t.diagnostic(JSON.stringify({source:'fresh Native93 current93 paid frame',http400:rejected,original88Untouched:true,noD1Writes:true}));
});
test('current93 retains borrowed-owner cleanup, score and original receipts after real D1 reopen',async t=>{
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
 t.diagnostic(JSON.stringify({source:'fresh current-source Native93',accepted,rejected,projections,reopened,allOriginalReceiptsUnchanged:true,syntheticDatabaseRetained:true}));
});
