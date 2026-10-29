// Current closed Native scenarios, local HTTP/Worker/D1; not browser play.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {harness} from './xq23-http-harness.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const cases=['bq052','tentacle'].flatMap(id=>loadCurrentFixture(`rust-game-wasm/tests/current113/${id}.json.gz`).cases);
for(const scenario of cases)
test(`${scenario.name}: actual Native generation/lifecycle, receipts and cold reopening`,async t=>{
 const h=await harness(t,scenario);let reopened=0,rejectedPaidReveal=0;
 for(const step of scenario.steps){
  assert.equal((await h.store().room(h.id)).state,step.state);
  const negative=scenario.paidRevealRejection;
  if(negative&&negative.state===step.state){
   const before=await h.snapshot();
   for(let retry=0;retry<2;retry++){
    const result=await h.api('commands',negative.seat,negative.command);
    assert.equal(result.status,400);assert.equal(result.body.error,negative.expected.errorCode);
    assertPublicView(result.body.view,negative.expected.view);assert.deepEqual(await h.snapshot(),before);
   }
   rejectedPaidReveal++;
  }
  const response=await h.api('commands',step.seat,step.command);
  assert.equal(response.status,200);assertPublicView(response.body,step.expected.view);
  assert.equal((await h.store().room(h.id)).state,step.expected.state);
  const row=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,step.expected.version).first();assertSessionJournal(row.entry,step.expected.journal);
  for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,200);assertPublicView(r.body,step.views[seat]);}
  const saved=await h.snapshot();const duplicate=await h.api('commands',step.seat,step.command);assert.equal(duplicate.status,200);assertPublicView(duplicate.body,step.expected.view);assert.deepEqual(await h.snapshot(),saved);
  await h.reopen();reopened++;
 }
 const first=scenario.steps[0],saved=await h.snapshot();
 const duplicate=await h.api('commands',first.seat,first.command);assert.equal(duplicate.status,200);assertPublicView(duplicate.body,first.expected.view);assert.deepEqual(await h.snapshot(),saved);
 assert.equal(rejectedPaidReveal,scenario.paidRevealRejection?1:0);
 t.diagnostic(JSON.stringify({rejectedPaidReveal,preparedNativeLayout:!scenario.construction,naturalClosedRegistry:!!scenario.construction,commands:scenario.steps.length,coldReopens:reopened,originalReceiptAfterCompletion:true}));
});
