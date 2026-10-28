// Fresh Native prepared and optional closed-registry natural cases; local HTTP/Worker/D1, not browser play.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {harness} from './xq23-http-harness.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/mscq02.json.gz');
for(const scenario of fixture.cases)
test(`MSCQ02 ${scenario.name}: HTTP opening destruction, time threshold and destroyed-region win draw, receipts and cold D1 reopening`,async t=>{
 const h=await harness(t,scenario);let reopened=0;
 for(const step of scenario.steps){
  assert.equal((await h.store().room(h.id)).state,step.state);
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
 t.diagnostic(JSON.stringify({preparedNativeLayout:!scenario.construction,naturalClosedRegistry:!!scenario.construction,commands:scenario.steps.length,coldReopens:reopened,originalReceiptAfterCompletion:true}));
});
