// Explicit Native layouts through real workerd/D1; no natural-browser claim.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import * as abi from '../generated/hegemony_wasm.js';
import {routeKernels} from '../src/kernel-router.mjs';
import {RoomStore} from '../src/store.mjs';
import {RoomService,digest} from '../src/service.mjs';
const traces=JSON.parse(readFileSync(new URL('./fixtures/deck-seal-native-v051.json',import.meta.url),'utf8'));
abi.initSync({module:readFileSync('generated/hegemony_wasm_bg.wasm')});
for(const [caseIndex,trace] of traces.entries()) test(`deck seal51 Native/D1 ${caseIndex}: private choice, pause/reopen, sealing/event and duplicate receipt`,async t=>{
 const persist=mkdtempSync(join(tmpdir(),'deck-seal51-d1-'));
 const options=convertV4MiniflareOptions({name:'deck-seal51-test',resourcePersistencePath:persist,modules:[
  {type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))
 ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'deck-seal51-test'}});
 let mf=new Miniflare(options);t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
 let db=await mf.getD1Database('DB');
 for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const initial=JSON.parse(abi.view(trace.initialState,0)),id=initial.roomId,tokens=['a','b','c','d'].map(x=>x.repeat(64));
 let store=new RoomStore(db),now=0,service=new RoomService(db,routeKernels(abi),()=>now);
 await store.create({id,invite:initial.inviteCode,state:trace.initialState,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-native-deck-seal-layout',response:'{}'});
 await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
 for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
 for(const [index,step] of trace.steps.entries()){
  now=Number(step.serverNowMs);const view=await service.command(id,'Bearer '+tokens[step.seat],step.command);
  assert.deepEqual(view,step.transition.view);assert.equal((await store.room(id)).state,step.transition.state);
  for(let seat=0;seat<4;seat++)assert.deepEqual(await service.state(id,'Bearer '+tokens[seat],null),step.views[seat]);
  if(index===0){
   const saved=await store.room(id);await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);service=new RoomService(db,routeKernels(abi),()=>now);
   assert.equal((await store.room(id)).state,saved.state);
   now+=600000;assert.deepEqual(await service.command(id,'Bearer '+tokens[step.seat],step.command),view);
  }
 }
 const last=trace.steps.at(-1),before=await store.room(id);now+=600000;
 assert.deepEqual(await service.command(id,'Bearer '+tokens[last.seat],last.command),last.transition.view);
 await assert.rejects(service.command(id,'Bearer '+tokens[last.seat],{...last.command,expectedVersion:last.command.expectedVersion+1}),e=>e.status===409&&/同一请求标识/.test(e.message));
 assert.deepEqual(await store.room(id),before);
 const receipts=(await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n;assert.equal(receipts,trace.steps.length);
 t.diagnostic(JSON.stringify({engine:JSON.parse(abi.catalog()).engineVersion,preparedNativeLayout:true,commands:trace.steps.length,privateOtherSeats:3,pauseReopen:true,duplicateOriginalReceipt:true,completeNativeWasmD1StateEqual:true}));
});
