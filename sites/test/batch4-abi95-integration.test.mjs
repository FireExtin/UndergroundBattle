// Actual local workerd + exact production WASM + isolated synthetic D1. Native
// layouts are explicit; no browser, public admission or full50 acceptance claim.
import test from 'node:test';
import assert from 'node:assert/strict';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
import {harness} from './xq23-http-harness.mjs';
const fixture=loadCurrentFixture('docs/evidence/abi95-platform-20261023/jz56.current.json.gz');
for(const scenario of fixture.cases)test(`ABI95 JZ56 actual Worker ${scenario.name}`,async t=>{
 const h=await harness(t,scenario);let projections=0,reopened=0;
 const allViews=async views=>{for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,200,JSON.stringify(r.body));assertPublicView(r.body,views[seat]);projections++;}};
 const initial=await h.snapshot();await allViews(scenario.initial.views);assert.deepEqual(await h.snapshot(),initial);
 const catalogue=await h.api('catalog',0);assert.equal(catalogue.status,200);assert.deepEqual(catalogue.body,fixture.catalog);
 const unauthorized=await h.api('state',99);assert.equal(unauthorized.status,401);assert.deepEqual(await h.snapshot(),initial);
 for(let i=0;i<scenario.steps.length;i++){
  const row=scenario.steps[i],before=await h.snapshot();assert.equal(before.room.state,row.state);assert(row.state.includes(`"seed":${scenario.seedDecimal}`));
  const result=await h.api('commands',row.seat,row.command);assert.equal(result.status,200,JSON.stringify(result.body));assertPublicView(result.body,row.expected.view);
  const saved=await h.snapshot();assert.equal(saved.room.state,row.expected.state);assert(saved.room.state.includes(`"seed":${scenario.seedDecimal}`));
  const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,row.expected.version).first();assertSessionJournal(journal.entry,row.expected.journal);
  const retry=await h.api('commands',row.seat,row.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,result.body);assert.deepEqual(await h.snapshot(),saved);
  await allViews(row.views);
  if(i===Math.floor(scenario.steps.length/2)||i===scenario.steps.length-1){
   await h.reopen();reopened++;assert.deepEqual(await h.snapshot(),saved);await allViews(row.views);
   const afterReopen=await h.api('commands',row.seat,row.command);assert.equal(afterReopen.status,200);assert.deepEqual(afterReopen.body,result.body);assert.deepEqual(await h.snapshot(),saved);
  }
 }
 const final=await h.snapshot();assert.equal(final.commands.length,scenario.steps.length);assert.equal(final.journal.length,scenario.steps.length);
 t.diagnostic(JSON.stringify({scenario:scenario.name,nativeCommands:scenario.steps.length,projections,reopened,identicalReceipts:true,exactOpaqueStates:true,opaqueU64Preserved:true,publicAdmission:false}));
});
const invalid=new Map();for(const scenario of fixture.cases)for(const bad of scenario.invalid)if(!invalid.has(bad.label))invalid.set(bad.label,{bad,views:scenario.initial.views});
for(const [label,{bad,views}]of invalid)test(`ABI95 JZ56 corrupted ${label} is HTTP400 without D1 writes`,async t=>{
 const initial={state:bad.state,views:views.map(v=>({...v,roomId:bad.roomId,inviteCode:bad.inviteCode,version:bad.revision}))};
 const h=await harness(t,{initial}),before=await h.snapshot();let rejected=0;
 for(let reopened=0;reopened<2;reopened++){
  if(reopened)await h.reopen();
  for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,400);assert.equal(r.body.message,bad.nativeError);rejected++;}
  const r=await h.api('commands',0,{commandId:'invalid-'+label,expectedVersion:bad.revision,action:{kind:'pauseRoom'}});assert.equal(r.status,400);assert.equal(r.body.message,bad.nativeError);rejected++;
  assert.deepEqual(await h.snapshot(),before);
 }
 assert.equal(before.commands.length,0);assert.equal(before.journal.length,0);t.diagnostic(JSON.stringify({label,http400:rejected,realD1Reopen:true,noStateCommandJournalWrites:true}));
});
