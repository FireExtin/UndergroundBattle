// Closed source-bound real Native Room inputs, local HTTP/Worker/D1 only.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {harness} from './xq23-http-harness.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const scenarios=loadCurrentFixture('rust-game-wasm/tests/current113/xq13.json.gz').cases.map(c=>{if(!c.construction)return c;const index=c.steps.findIndex(s=>s.command.action.action?.kind==='play');assert(index>0);return{...c,name:c.name+'-native-earned-attachment-tail',nativePreparationCommands:index,initial:{state:c.steps[index].state,views:c.steps[index-1].views},steps:c.steps.slice(index)};});
for(const scenario of scenarios)
test(`${scenario.name}: actual path attachment and icon conversion survive real Worker persistence`,async t=>{
 const h=await harness(t,scenario);let reopens=0,commands=0;
 for(const step of scenario.steps){assert.equal((await h.store().room(h.id)).state,step.state);const result=await h.api('commands',step.seat,step.command);assert.equal(result.status,200);assertPublicView(result.body,step.expected.view);assert.equal((await h.store().room(h.id)).state,step.expected.state);
  const row=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,step.expected.version).first();assertSessionJournal(row.entry,step.expected.journal);
  for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,200);assertPublicView(r.body,step.views[seat]);}
  const saved=await h.snapshot();const dup=await h.api('commands',step.seat,step.command);assert.equal(dup.status,200);assertPublicView(dup.body,step.expected.view);assert.deepEqual(await h.snapshot(),saved);
  commands++;await h.reopen();reopens++;
 }
 const first=scenario.steps[0],saved=await h.snapshot();const again=await h.api('commands',first.seat,first.command);assert.equal(again.status,200);assertPublicView(again.body,first.expected.view);assert.deepEqual(await h.snapshot(),saved);
 t.diagnostic(JSON.stringify({commands,reopens,views:commands*4,nativePreparationCommands:scenario.nativePreparationCommands||0,naturalClosedRegistry:!!scenario.construction,preparedNativeLayout:!scenario.construction,originalReceiptAfterCompletion:true}));
});
