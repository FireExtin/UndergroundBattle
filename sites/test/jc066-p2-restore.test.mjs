// Exact review counterexamples. Rust alone changes two typed snapshot fields;
// JS transports opaque Room strings without parsing or rewriting them.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,mkdtempSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {assertPublicView} from './assert-session-journal.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
const reject=process.env.JC066_P2_EXPECT!=='accept';
assert(process.env.JC066_P2_CASES_FILE);const repro=JSON.parse(readFileSync(process.env.JC066_P2_CASES_FILE,'utf8'));
assert.equal(repro.expectedReject,reject);assert.equal(repro.cases.length,2);
const pkg=path.resolve(process.env.JC066_P2_PKG||'../rust-game-wasm/pkg');
const abi=await import(pathToFileURL(path.join(pkg,'hegemony_wasm.js')));
const wasm=readFileSync(path.join(pkg,'hegemony_wasm_bg.wasm'));abi.initSync({module:wasm});
const wasmSha256=hash(wasm);
if(!reject)assert.equal(wasmSha256,'a389ee48edb55ef5089a641c781352f8890ea1a6238efd1bde2664fb6f78d30e');
else assert.notEqual(wasmSha256,'a389ee48edb55ef5089a641c781352f8890ea1a6238efd1bde2664fb6f78d30e');
const worker=process.env.HEGEMONY_CURRENT_WORKER_DIR;assert(worker);
for(const c of repro.cases){
 const command={commandId:'p2-'+c.name,expectedVersion:c.revision,action:{kind:'pauseRoom'}};
 test(`JC066 P2 ${reject?'after rejection':'before acceptance'} WASM ${c.name}`,t=>{
  assert.equal(c.gameAccepted,!reject);assert.equal(c.roomAccepted,!reject);
  for(let seat=0;seat<4;seat++)assert.equal(JSON.parse(abi.view(c.originalState,seat)).version,c.revision);
  const operations=[...Array.from({length:4},(_,s)=>()=>abi.view(c.state,s)),()=>abi.pollRoom(c.state,0,'0'),()=>abi.applyRoom(c.state,0,JSON.stringify(command),'0')];
  for(const op of operations){
   if(reject)assert.throws(op,e=>String(e)===c.roomError);
   else assert.equal(typeof op(),'string');
  }
  t.diagnostic(JSON.stringify({case:c.name,reject,wasmSha256,stateSha256:hash(c.state),nativeGameAccepted:c.gameAccepted,nativeRoomAccepted:c.roomAccepted,originalViewsAccepted:4,mutatedEntrypoints:6}));
 });
 test(`JC066 P2 ${reject?'after rejection':'before acceptance'} actual HTTP/D1 ${c.name}`,async t=>{
  const persist=mkdtempSync(path.join(tmpdir(),'jc066-p2-'));
  const options=convertV4MiniflareOptions({name:'jc066-p2',resourcePersistencePath:persist,modulesRoot:worker,modules:[{type:'ESModule',path:path.join(worker,'worker.js')},...readdirSync(worker).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:path.join(worker,f)}))],compatibilityDate:'2026-10-02',cf:false,bindings:{LOCAL_TEST_CLOCK_MS:'0'},d1Databases:{DB:'jc066-p2'}});
  let mf=new Miniflare(options),db=await mf.getD1Database('DB');t.after(()=>mf.dispose());
  for(const file of readdirSync('drizzle').filter(f=>f.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64));
  await new RoomStore(db).create({id:c.roomId,invite:c.inviteCode,state:c.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-offline-p2-repro',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(c.revision,c.roomId).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(c.roomId,seat,await digest(tokens[seat])).run();
  const snapshot=async()=>({room:await new RoomStore(db).room(c.roomId),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(c.roomId).all()).results,journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(c.roomId).all()).results});
  const api=async(op,seat,body)=>{const r=await mf.dispatchFetch(`http://localhost/api/rooms/${c.roomId}/${op}`,{method:body?'POST':'GET',headers:{Authorization:'Bearer '+tokens[seat],...(body?{'Content-Type':'application/json'}:{})},...(body?{body:JSON.stringify(body)}:{})});return {status:r.status,body:await r.json()};};
  const before=await snapshot();let acceptedSnapshot,receipt;
  for(let reopen=0;reopen<2;reopen++){
   if(reopen){const saved=await snapshot();await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');assert.deepEqual(await snapshot(),saved);}
   for(let seat=0;seat<4;seat++){const r=await api('state',seat);assert.equal(r.status,reject?400:200);if(reject)assert.equal(r.body.message,c.roomError);}
   const r=await api('commands',0,command);assert.equal(r.status,reject?400:200);
   if(reject){assert.equal(r.body.message,c.roomError);assert.deepEqual(await snapshot(),before);}
   else if(!reopen){const expected=JSON.parse(abi.applyRoom(c.state,0,JSON.stringify(command),'0'));assert.equal(expected.outcome,'accepted');assertPublicView(r.body,expected.view);acceptedSnapshot=await snapshot();assert.equal(acceptedSnapshot.room.state,expected.state);assert.equal(acceptedSnapshot.commands.length,1);assert.equal(acceptedSnapshot.journal.length,1);receipt=r.body;}
   else{assert.deepEqual(r.body,receipt);assert.deepEqual(await snapshot(),acceptedSnapshot);}
  }
  t.diagnostic(JSON.stringify({case:c.name,reject,wasmSha256,stateSha256:hash(c.state),httpReads:8,httpCommands:2,reopened:true,zeroWrites:reject,originalReceiptAfterReopen:!reject,roomCommandsJournalUnchanged:reject,syntheticDatabaseRetained:true}));
 });
}
