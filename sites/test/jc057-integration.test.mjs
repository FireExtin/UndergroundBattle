// Actual local HTTP/workerd/eight-WASM/D1; explicit Native84 layouts, not live rooms.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { assertPublicView, assertSessionJournal } from './assert-session-journal.mjs';
import { loadFixture } from '../../tools/jc057-oracle.mjs';
const fixture=loadFixture();
const worker=process.env.HEGEMONY_CURRENT_WORKER_DIR||path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT||'/workspace/undergroundbattle-build','jc054-worker/server');
async function harness(t,scenario) {
  const persist=mkdtempSync(path.join(tmpdir(),'jc057-http-'));
  const options={name:'jc057-http',resourcePersistencePath:persist,modulesRoot:worker,modules:[
    {type:'ESModule',path:path.join(worker,'worker.js')},
    ...readdirSync(worker).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:path.join(worker,f)}))],
    compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'jc057-http'}};
  let mf,db,now=0;
  const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions({...options,bindings:{LOCAL_TEST_CLOCK_MS:String(now)}}));db=await mf.getD1Database('DB');};
  await start();t.after(()=>mf.dispose()); // Preserve local synthetic D1 evidence; no cleanup.
  for(const f of readdirSync('drizzle').filter(f=>f.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+f,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const initial=scenario.initial.views[0],id=initial.roomId,tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64));
  const store=()=>new RoomStore(db);
  await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'offline-native-jc057',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  const snapshot=async()=>({room:await store().room(id),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results,
    journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
  return {id,store,db:()=>db,snapshot,clock:()=>now,
    reopen:async(clock=now)=>{const before=await snapshot();await mf.dispose();now=clock;await start();assert.deepEqual(await snapshot(),before);},
    api:async(operation,seat=0,body)=>{const response=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${operation}`,{method:body?'POST':'GET',
      headers:{Authorization:'Bearer '+tokens[seat],...(body?{'Content-Type':'application/json'}:{})},...(body?{body:JSON.stringify(body)}:{})});
      return {status:response.status,body:response.status===204?null:await response.json()};}};
}
for(const scenario of fixture.scenarios)test(`JC057 ABI84 HTTP ${scenario.name}`,async t=>{
  const h=await harness(t,scenario);let accepted=0,rejected=0,projections=0;
  const receipts=[];
  const viewAll=async views=>{for(let seat=0;seat<4;seat++){
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
    // Reopen every accepted command: paid response frames, private search
    // choices, pause/resume and post-resolution state all cross real D1 reopen.
    if(!row.expected.errorCode){await h.reopen();await viewAll(row.views);}
  }
  const final=await h.snapshot();assert.equal(final.commands.length,accepted);
  await h.reopen();
  for(const {row,body}of receipts) {
    const retry=await h.api('commands',row.seat,row.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,body);
    assert.deepEqual(await h.snapshot(),final);
  }
  t.diagnostic(JSON.stringify({scenario:scenario.name,reference:'compiler-bound Native84',accepted,rejected,projections,
    originalReceiptsAfterReopen:true,opaqueStatesAndJournalsEqual:true,localSyntheticDatabaseRetained:true}));
});

for(const invalid of fixture.invalid)test(`JC057 corrupted Declare ${invalid.name} restores as HTTP400 with no D1 writes`,async t=>{
  const reference=fixture.scenarios.find(s=>s.name==='reveal-p1').steps[5];
  const scenario={name:'invalid-'+invalid.name,initial:{state:invalid.state,views:reference.views}};
  const h=await harness(t,scenario),before=await h.snapshot();
  const command={commandId:'invalid-'+invalid.name,expectedVersion:invalid.revision,
    action:{kind:'game',action:{kind:'choose',choiceId:invalid.choiceId,selected:[]}}};
  let rejected=0;
  for(let reopen=0;reopen<2;reopen++) {
    if(reopen)await h.reopen();
    for(let seat=0;seat<4;seat++) {
      const result=await h.api('state',seat);assert.equal(result.status,400);assert.equal(result.body.message,invalid.roomError);rejected++;
    }
    const result=await h.api('commands',invalid.seat,command);assert.equal(result.status,400);assert.equal(result.body.message,invalid.roomError);rejected++;
    assert.deepEqual(await h.snapshot(),before);
  }
  assert.equal(before.commands.length,0);assert.equal(before.journal.length,0);
  t.diagnostic(JSON.stringify({invalid:invalid.name,http400:rejected,realD1Reopen:true,roomCommandsJournalUnchanged:true}));
});
