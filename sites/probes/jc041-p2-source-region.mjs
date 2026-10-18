// Explicit P2 probe only, outside default test/*.test.mjs discovery.
// Rust creates every opaque mutation; JS transports Room strings unchanged.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,mkdtempSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {assertPublicView,assertSessionJournal} from '../test/assert-session-journal.mjs';
import {assertSources} from '../../tools/jc041-oracle.mjs';
const file=process.env.JC041_P2_CASES_FILE,wasmDirectory=process.env.JC041_P2_WASM_DIR,worker=process.env.HEGEMONY_CURRENT_WORKER_DIR;
assert(file&&wasmDirectory&&worker,'Explicit Native P2 file, exact WASM directory and local Worker are required');
const f=JSON.parse(readFileSync(file,'utf8'));assert.equal(f.format,'jc041-p2-source-region-v1');
assert.equal(f.cases.length,4);assert.equal(f.legal.length,4);
const sha=b=>createHash('sha256').update(b).digest('hex');
if(f.expectedAccepted){
 const repo=path.resolve('..'),expected=Object.keys(JSON.parse(readFileSync(path.join(repo,'rust-game-wasm/frozen/abi86/native-source-binding.json'),'utf8')).files).concat(['rust-game/src/jc041.rs','rust-game/src/jc041_tests.rs']).sort();
 assert.deepEqual(Object.keys(f.sourceBindings.files).sort(),expected);assert.equal(expected.length,137);
 // Immutable before product plus the exact test-only repro source compiled then.
 for(const [name,hash]of Object.entries(f.sourceBindings.files)){
  const ref=name==='rust-game/src/jc041_tests.rs'?'6b5b824b1e3fd08edfb1b47f7d979ef1ffe69317':'f3dece927c733a761cf78d62514c6299fab280fa';
  assert.equal(sha(execFileSync('git',['show',ref+':'+name],{cwd:repo})),hash,'Immutable before compiler input '+name);
 }
}else assertSources(f.sourceBindings);
assert.equal(sha(f.originalState),'ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c');
const bytes=readFileSync(path.join(wasmDirectory,'hegemony_wasm_bg.wasm'));
if(f.expectedAccepted)assert.equal(sha(bytes),'980e4e727c92918de24a005ef63b2c361c7f9d2c765dc45bf304495efade2482');
else assert.notEqual(sha(bytes),'980e4e727c92918de24a005ef63b2c361c7f9d2c765dc45bf304495efade2482');
const abi=await import(pathToFileURL(path.join(wasmDirectory,'hegemony_wasm.js')).href);abi.initSync({module:bytes});
assert.equal(JSON.parse(abi.catalog()).engineVersion,'rust-v0.2.87-jc041-water-hidden-candidate');
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
for(const c of f.cases)test(`JC041 P2 ${c.name}: exact Native/WASM/HTTP restore boundary`,async t=>{
 f.originalViews.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(f.originalState,seat)),v));
 if(f.expectedAccepted){
  assert.equal(c.gameError,null);assert.equal(c.roomError,null);
  c.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(c.state,seat)),v));
  assert.deepEqual(JSON.parse(abi.pollRoom(c.state,0,'0')),c.expectedPoll);
  assert.deepEqual(JSON.parse(abi.applyRoom(c.state,0,JSON.stringify(c.command),'0')),c.expectedPause);
 }else{
  assert(c.gameError&&c.roomError);const same=e=>{assert.equal(String(e),c.roomError);return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(c.state,seat),same);
  assert.throws(()=>abi.pollRoom(c.state,0,'0'),same);assert.throws(()=>abi.applyRoom(c.state,0,JSON.stringify(c.command),'0'),same);
 }
 const h=await harness(t,c.state,f.originalViews[0]),initial=await h.snapshot();let http=0;
 if(f.expectedAccepted){
  for(let seat=0;seat<4;seat++){const r=await h.api(seat);assert.equal(r.status,200);assertPublicView(r.body,c.views[seat]);http++;}
  const r=await h.api(0,c.command);assert.equal(r.status,200);assertPublicView(r.body,c.expectedPause.view);http++;
  assert.equal((await h.room()).state,c.expectedPause.state);assertSessionJournal(await h.journal(c.expectedPause.version),c.expectedPause.journal);
  const saved=await h.snapshot();assert.equal(saved.commands.length,1);assert.equal(saved.journal.length,1);await h.reopen();
  for(let seat=0;seat<4;seat++){const p=await h.api(seat);assert.equal(p.status,200);assertPublicView(p.body,JSON.parse(abi.view(c.expectedPause.state,seat)));http++;}
  assert.deepEqual(await h.api(0,c.command),r);http++;assert.deepEqual(await h.snapshot(),saved);
 }else for(let reopen=0;reopen<2;reopen++){
  if(reopen)await h.reopen();
  for(let seat=0;seat<4;seat++){const r=await h.api(seat);assert.equal(r.status,400);assert.equal(r.body.message,c.roomError);http++;}
  const r=await h.api(0,c.command);assert.equal(r.status,400);assert.equal(r.body.message,c.roomError);http++;assert.deepEqual(await h.snapshot(),initial);
 }
 assert.equal(http,10);t.diagnostic(JSON.stringify({case:c.name,wasmSha256:sha(bytes),accepted:f.expectedAccepted,httpStatus:f.expectedAccepted?200:400,httpCalls:http,commandsWritten:f.expectedAccepted?1:0,journalWritten:f.expectedAccepted?1:0,realD1Reopen:true,originalReceiptOrZeroWritesConfirmed:true,opaqueStateSha256:sha(c.state)}));
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
 t.diagnostic(JSON.stringify({legal:s.name,commands:s.steps.length,allNativeOpaqueStatesJournalsViewsEqual:true,D1ReopenedAfterEveryCommand:true,originalReceiptsAfterReopen:true,initialLayoutBoundary:s.name==='p2-original'?'original real Room':'explicit Native layout; subsequent real Room commands only'}));
});
