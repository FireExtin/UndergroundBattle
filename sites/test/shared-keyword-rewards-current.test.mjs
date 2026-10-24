// Actual local Workerd/HTTP/D1, using fresh current-only Native commands.
import test from 'node:test';
import assert from 'node:assert/strict';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
import {harness} from './xq23-http-harness.mjs';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current112/shared-rewards-http.json.gz');
assert.equal(fixture.catalog.engineVersion,'rust-v0.2.112-shared-keyword-rewards');
for(const scenario of fixture.cases)test(`actual current Worker shared reward: ${scenario.name}`,async t=>{
 const h=await harness(t,scenario);let views=0,duplicates=0;
 const allViews=async expected=>{const before=await h.snapshot();for(let seat=0;seat<4;seat++){
  const response=await h.api('state',seat);assert.equal(response.status,200,JSON.stringify(response.body));
  assertPublicView(response.body,expected[seat]);views++;
 }assert.deepEqual(await h.snapshot(),before);};
 await allViews(scenario.initial.views);
 const catalog=await h.api('catalog',0);assert.equal(catalog.status,200);
 const {transport,entryIdempotency,...core}=catalog.body;assert.deepEqual(core,fixture.catalog);
 assert.equal(transport,'polling');assert.equal(entryIdempotency,true);
 for(const row of scenario.steps){
  assert.equal((await h.snapshot()).room.state,row.state);
  const response=await h.api('commands',row.seat,row.command);assert.equal(response.status,200,JSON.stringify(response.body));
  assertPublicView(response.body,row.expected.view);
  const saved=await h.snapshot();assert.equal(saved.room.state,row.expected.state);
  assert(saved.room.state.includes(`"seed":${scenario.seedDecimal}`));
  const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,row.expected.version).first();
  assertSessionJournal(journal.entry,row.expected.journal);
  const duplicate=await h.api('commands',row.seat,row.command);assert.equal(duplicate.status,200);
  assert.deepEqual(duplicate.body,response.body);assert.deepEqual(await h.snapshot(),saved);duplicates++;
  await allViews(row.views);
 }
 const final=await h.snapshot();assert.equal(final.commands.length,scenario.steps.length);
 assert.equal(final.journal.length,scenario.steps.length);
 t.diagnostic(JSON.stringify({scenario:scenario.name,commands:scenario.steps.length,views,duplicates,opaqueStateAndU64Exact:true,
  scope:'actual local Workerd/HTTP/D1; no browser or public deployment'}));
});
