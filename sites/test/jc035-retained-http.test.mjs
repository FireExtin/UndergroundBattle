// All thirteen actual reducers in one local workerd isolate; retained D1 rooms stay opaque.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync,readdirSync,mkdtempSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import * as frozen from '../../tools/frozen-kernels.mjs';
import {kernel as current} from '../../rust-game-wasm/tests/current89-kernel.mjs';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const worker=process.env.HEGEMONY_CURRENT_WORKER_DIR||path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT||'/workspace/undergroundbattle-build','jc035-worker/server');
test('one actual thirteen-kernel HTTP/D1 isolate preserves original77-88 and new89 state, commands, u64 and receipts',async t=>{
 const modules=readdirSync(worker).filter(f=>f.endsWith('.wasm'));assert.equal(modules.length,13);
 const options=convertV4MiniflareOptions({name:'jc035-retained-http',resourcePersistencePath:mkdtempSync(path.join(tmpdir(),'jc035-retained-http-')),modulesRoot:worker,
 modules:[{type:'ESModule',path:path.join(worker,'worker.js')},...modules.map(f=>({type:'CompiledWasm',path:path.join(worker,f)}))],compatibilityDate:'2026-10-02',cf:false,bindings:{LOCAL_TEST_CLOCK_MS:'0'},d1Databases:{DB:'jc035-retained-http'}});
 const mf=new Miniflare(options);t.after(()=>mf.dispose());const db=await mf.getD1Database('DB');
 for(const file of readdirSync('drizzle').filter(f=>f.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const kernels=Array.from({length:12},(_,i)=>{const s=frozen['frozenAbi'+(77+i)](),k=createRequire(import.meta.url)(s.jsPath);k.initSync({module:s.wasm});return k;}).concat(current);
 const token='1'.repeat(64),store=new RoomStore(db);
 for(const[index,k]of kernels.entries()){
  const id=(877700+index).toString(16).padStart(24,'0'),invite='JC'+String(7700+index),created=JSON.parse(k.newGame(id,invite,'duel','P0','watchers','18446744073709551615'));
  assert(created.state.includes('"seed":18446744073709551615'));
  await store.create({id,invite,state:created.state,nonce:crypto.randomUUID(),tokenHash:await digest(token),requestHash:crypto.randomUUID(),intentHash:'offline-thirteen-core-http',response:'{}'});
  const api=async(command)=>{const r=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${command?'commands':'state'}`,{method:command?'POST':'GET',headers:{Authorization:'Bearer '+token,...(command?{'Content-Type':'application/json'}:{})},...(command?{body:JSON.stringify(command)}:{})});return {status:r.status,body:await r.json()};};
  const before=await api();assert.equal(before.status,200);assertPublicView(before.body,created.view);
  const command={commandId:'retained-ready-'+index,expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}};
  const expected=JSON.parse(k.applyRoom(created.state,0,JSON.stringify(command),'0')),r=await api(command);assert.equal(r.status,200);assertPublicView(r.body,expected.view);
  assert.equal((await store.room(id)).state,expected.state);
  const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(journal.entry,expected.journal);
  const retry=await api(command);assert.deepEqual(retry,r);assert.equal((await store.room(id)).state,expected.state);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,1);
 }
 t.diagnostic(JSON.stringify({actualHttpKernelVersions:[77,78,79,80,81,82,83,84,85,86,87,88,89],sameIsolate:true,rooms:13,accepted:13,originalReceipts:13,opaqueStatesExact:true,seedU64Exact:true,syntheticDatabaseRetained:true}));
});
