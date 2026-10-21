// Explicit P2 probe only, outside default test/*.test.mjs discovery.
// Rust creates every opaque mutation; JS transports Room strings unchanged.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,mkdtempSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {assertPublicView,assertSessionJournal} from '../test/assert-session-journal.mjs';
import {loadP2Fixture,sha256 as sha} from '../../tools/jc041-oracle.mjs';
import {kernel as abi,moduleBytes as bytes} from '../../rust-game-wasm/tests/current-kernel.mjs';
const f=loadP2Fixture();assert.equal(sha(bytes),f.wasmSha256);
const worker=path.dirname(process.env.CURRENT_CLOCK_WORKER||fileURLToPath(new URL('../dist/current-clock/worker.js',import.meta.url)));
async function harness(t,state,view){
 const options={name:'jc041-p2-http',resourcePersistencePath:mkdtempSync(path.join(tmpdir(),'jc041-p2-region-')),modulesRoot:worker,
  modules:[{type:'ESModule',path:path.join(worker,'worker.js')},...readdirSync(worker).filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:path.join(worker,n)}))],
  compatibilityDate:'2026-10-02',cf:false,bindings:{LOCAL_TEST_CLOCK_MS:'0'},d1Databases:{DB:'jc041-p2-http'}};
 let mf,db;const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions(options));db=await mf.getD1Database('DB');};
 await start();t.after(()=>mf.dispose());
 for(const n of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+n,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const id=view.roomId,tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64)),store=()=>new RoomStore(db);
 await store().create({id,invite:view.inviteCode,state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'local-native-jc041-p2',response:'{}'});
 await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(view.version,id).run();
 for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
 const snapshot=async()=>({room:await store().room(id),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results,journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
 return {db:()=>db,room:()=>store().room(id),snapshot,reopen:async()=>{const before=await snapshot();await mf.dispose();await start();assert.deepEqual(await snapshot(),before);},
  api:async(seat,command)=>{const r=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${command?'commands':'state'}`,{method:command?'POST':'GET',headers:{Authorization:'Bearer '+tokens[seat],...(command?{'Content-Type':'application/json'}:{})},...(command?{body:JSON.stringify(command)}:{})});return{status:r.status,body:await r.json()};},
  journal:async version=>(await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,version).first()).entry};
}
for(const c of f.cases)test(`JC041 P2 ${c.name}: exact Native93/WASM93/HTTP restore boundary`,async t=>{
 f.originalViews.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(f.originalState,seat)),v));
 assert(c.gameError&&c.roomError);const same=e=>{assert.equal(String(e),c.roomError);return true;};
 for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(c.state,seat),same);
 assert.throws(()=>abi.pollRoom(c.state,0,'0'),same);assert.throws(()=>abi.applyRoom(c.state,0,JSON.stringify(c.command),'0'),same);
 const h=await harness(t,c.state,f.originalViews[0]),initial=await h.snapshot();let http=0;
 for(let reopen=0;reopen<2;reopen++){
  if(reopen)await h.reopen();
  for(let seat=0;seat<4;seat++){const r=await h.api(seat);assert.equal(r.status,400);assert.equal(r.body.message,c.roomError);http++;}
  const r=await h.api(0,c.command);assert.equal(r.status,400);assert.equal(r.body.message,c.roomError);http++;assert.deepEqual(await h.snapshot(),initial);
 }
 assert.equal(initial.commands.length,0);assert.equal(initial.journal.length,0);assert.equal(http,10);
 t.diagnostic(JSON.stringify({case:c.name,wasmSha256:sha(bytes),accepted:false,httpStatus:400,httpCalls:http,commandsWritten:0,journalWritten:0,realD1Reopen:true,zeroWritesConfirmed:true,opaqueStateSha256:sha(c.state)}));
});
for(const s of f.legal)test(`JC041 P2 legal ${s.name}: original/changed-region continuation remains exact`,async t=>{
 const h=await harness(t,s.initial.state,s.initial.views[0]);let state=s.initial.state;const receipts=[];
 for(const [seat,v]of s.initial.views.entries()){assert.deepEqual(JSON.parse(abi.view(state,seat)),v);const r=await h.api(seat);assert.equal(r.status,200);assertPublicView(r.body,v);}
 for(const row of s.steps){
  assert.equal(row.state,state);const expected=JSON.parse(abi.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs));assert.deepEqual(expected,row.expected);
  const r=await h.api(row.seat,row.command);assert.equal(r.status,200);assertPublicView(r.body,row.expected.view);assert.equal((await h.room()).state,row.expected.state);
  assertSessionJournal(await h.journal(row.expected.version),row.expected.journal);receipts.push({row,result:r});state=row.expected.state;
  await h.reopen();for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(abi.view(state,seat)),row.views[seat]);const p=await h.api(seat);assert.equal(p.status,200);assertPublicView(p.body,row.views[seat]);}
 }
 const saved=await h.snapshot();await h.reopen();for(const r of receipts){assert.deepEqual(await h.api(r.row.seat,r.row.command),r.result);assert.deepEqual(await h.snapshot(),saved);}
 t.diagnostic(JSON.stringify({legal:s.name,commands:s.steps.length,allNativeOpaqueStatesJournalsViewsEqual:true,D1ReopenedAfterEveryCommand:true,originalReceiptsAfterReopen:true,initialLayoutBoundary:s.name==='p2-original'?'fresh Native layout with real Room declaration':'explicit Native layout; subsequent real Room commands only'}));
});
