// Reused local-only HTTP/D1 harness; fixture state and receipt assertions stay in callers.
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
const worker=process.env.HEGEMONY_CURRENT_WORKER_DIR||path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT||'/workspace/undergroundbattle-build','xq23-worker/server');
export async function harness(t,scenario) {
  const persist=mkdtempSync(path.join(tmpdir(),'xq23-http-'));
  const options={name:'xq23-http',resourcePersistencePath:persist,modulesRoot:worker,modules:[
    {type:'ESModule',path:path.join(worker,'worker.js')},
    ...readdirSync(worker).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:path.join(worker,f)}))],
    compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'xq23-http'}};
  let mf,db,now=0;
  const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions({...options,bindings:{LOCAL_TEST_CLOCK_MS:String(now)}}));db=await mf.getD1Database('DB');};
  await start();t.after(()=>mf.dispose()); // Preserve local synthetic D1 evidence; no cleanup.
  for(const f of readdirSync('drizzle').filter(f=>f.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+f,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const initial=scenario.initial.views[0],id=initial.roomId,tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64));
  const store=()=>new RoomStore(db);
  await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'offline-native-xq23',response:'{}'});
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
