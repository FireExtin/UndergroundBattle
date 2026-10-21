// Actual HTTP/workerd/single-WASM/D1 with synthetic Native93 layouts only.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { assertPublicView, assertSessionJournal } from './assert-session-journal.mjs';
import { loadFixture } from '../../tools/response-preview-oracle.mjs';
const fixture=loadFixture();
const worker=path.dirname(process.env.CURRENT_CLOCK_WORKER||fileURLToPath(new URL('../dist/current-clock/worker.js',import.meta.url)));
async function harness(t,scenario) {
  const persist=mkdtempSync(path.join(tmpdir(),'response-preview-http-'));
  const options={name:'response-preview-http',resourcePersistencePath:persist,modulesRoot:worker,modules:[
    {type:'ESModule',path:path.join(worker,'worker.js')},
    ...readdirSync(worker).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:path.join(worker,f)}))],
    compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'response-preview-http'}};
  let mf,db,now=0;
  const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions({...options,bindings:{LOCAL_TEST_CLOCK_MS:String(now)}}));db=await mf.getD1Database('DB');};
  await start();t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
  for(const f of readdirSync('drizzle').filter(f=>f.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+f,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const initial=scenario.initial.views[0],id=initial.roomId,tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64));
  const store=()=>new RoomStore(db);
  await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'offline-native-response-preview',response:'{}'});
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
for(const scenario of fixture.scenarios)test(`response preview ABI93 HTTP ${scenario.name}`,async t=>{
  const h=await harness(t,scenario);let accepted=0,rejected=0,projections=0,pureQueries=0,quotes=0,reopens=0;
  const receipts=[];
  const viewAll=async views=>{for(let seat=0;seat<4;seat++){const r=await h.api('state',seat);assert.equal(r.status,200);assertPublicView(r.body,views[seat]);projections++;}};
  const before=await h.snapshot();await viewAll(scenario.initial.views);await viewAll(scenario.initial.views);assert.deepEqual(await h.snapshot(),before);pureQueries+=8;
  const w=scenario.initial.views[0].responseWindow;
  assert(w.previewSources.length>0);assert.equal(w.myIntentId,undefined);
  assert(scenario.initial.views[0].legalActions.every(a=>a.kind==='pass'));
  await h.reopen();reopens++;await viewAll(scenario.initial.views);assert.deepEqual(await h.snapshot(),before);
  for(const row of scenario.steps) {
    assert.equal((await h.store().room(h.id)).state,row.state);
    if(row.kind==='view'){
      // Native direct views use the persisted clock. A no-op poll at 4999 did
      // not persist that clock; restore 0 before comparing those observations.
      const savedClock=JSON.parse(row.state).pacing.lastServerNowMs;
      if(h.clock()!==savedClock){await h.reopen(savedClock);reopens++;}
      const saved=await h.snapshot();await viewAll(row.views);assert.deepEqual(await h.snapshot(),saved);pureQueries+=4;continue;
    }
    if(row.kind==='quote'){
      const saved=await h.snapshot(),r=await h.api('quote',row.seat,row.request);quotes++;
      if(row.error){assert.equal(r.status,400);assert.equal(r.body.message,row.error);}
      else {assert.equal(r.status,200);assert.deepEqual(r.body,row.expected);}
      assert.deepEqual(await h.snapshot(),saved);continue;
    }
    const now=Number(row.serverNowMs);if(h.clock()!==now){await h.reopen(now);reopens++;}
    const saved=await h.snapshot();
    const r=await h.api(row.kind==='poll'?'state':'commands',row.seat,row.command||undefined);
    if(row.expected.errorCode){assert.equal(r.status,409);assert.equal(r.body.error,row.expected.errorCode);assertPublicView(r.body.view,row.expected.view);rejected++;assert.deepEqual(await h.snapshot(),saved);}
    else {
      assert.equal(r.status,200,JSON.stringify(r.body));assertPublicView(r.body,row.expected.view);
      if(row.kind==='command'){accepted++;receipts.push({row,body:r.body});}
    }
    assert.equal((await h.store().room(h.id)).state,row.expected.state);
    if(row.expected.changed){
      const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(h.id,row.expected.version).first();assertSessionJournal(journal.entry,row.expected.journal);
    }else assert.deepEqual(await h.snapshot(),saved);
    if(row.kind==='command')await viewAll(row.views);
  }
  const final=await h.snapshot();assert.equal(final.commands.length,accepted);
  await h.reopen();reopens++;
  for(const {row,body}of receipts){const repeated=await h.api('commands',row.seat,row.command);assert.equal(repeated.status,200);assert.deepEqual(repeated.body,body);assert.deepEqual(await h.snapshot(),final);}
  t.diagnostic(JSON.stringify({scenario:scenario.name,reference:'compiler-bound Native93',accepted,rejected,quotes,projections,pureQueries,reopens,originalReceiptsAfterReopen:true,opaqueStatesAndJournalsEqual:true}));
});
