// Explicit Native layouts through real WASM/RoomService/local workerd D1.
// No production data, live Site, or natural-browser claim.
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
const fixtures=JSON.parse(readFileSync(new URL('./fixtures/bounded-entry-native-v063.json',import.meta.url),'utf8'));
abi.initSync({module:readFileSync('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')});
for(const [label,count] of [['employee',1],['employeeEmpty',0],['passers',0],['passers',3],['passersEmpty',0]]) test(`bounded ${label}/${count}: private candidates, pause/reopen, exact result and one receipt`,async t=>{
 const persist=mkdtempSync(join(tmpdir(),'entry-search50-d1-'));
 const options=convertV4MiniflareOptions({name:'entry-search50',resourcePersistencePath:persist,modules:[
  {type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))
 ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'entry-search50'}});
 let mf=new Miniflare(options);t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
 let db=await mf.getD1Database('DB');for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const initial=fixtures.states[label],view=JSON.parse(abi.view(initial,0)),id=view.roomId,tokens=['a','b','c','d'].map(x=>x.repeat(64));
 let store=new RoomStore(db),now=0,service=new RoomService(db,routeKernels(abi),()=>now);
 await store.create({id,invite:view.inviteCode,state:initial,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-native-entry-search-layout',response:'{}'});
 await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(view.version,id).run();
 for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
 const command=(key,version,action)=>({commandId:key,expectedVersion:version,action});
 const pause=command('pause-entry',view.version,{kind:'pauseRoom'});let paused=await service.command(id,'Bearer '+tokens[0],pause);
 assert.deepEqual(paused.pendingChoice,view.pendingChoice);
 for(let seat=1;seat<4;seat++){const v=await service.state(id,'Bearer '+tokens[seat],null);assert.equal(v.pendingChoice,null);assert(!JSON.stringify(v).includes(view.pendingChoice.options[0]?.id||'no-such-id'));}
 const saved=await store.room(id);await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);service=new RoomService(db,routeKernels(abi),()=>now);
 assert.equal((await store.room(id)).state,saved.state);now=90000;assert.deepEqual(await service.command(id,'Bearer '+tokens[0],pause),paused);
 const resumed=await service.command(id,'Bearer '+tokens[0],command('resume-entry',paused.version,{kind:'resumeRoom'}));assert.deepEqual(resumed.pendingChoice,view.pendingChoice);
 const selected=view.pendingChoice.options.slice(0,count).map(o=>o.id);
 const choose=command('choose-entry',resumed.version,{kind:'game',action:{kind:'choose',choiceId:view.pendingChoice.id,selected}});
 const before=await store.room(id);const expected=JSON.parse(abi.applyRoom(before.state,0,JSON.stringify(choose),String(now)));
 assert.equal(expected.outcome,'accepted');const complete=await service.command(id,'Bearer '+tokens[0],choose);assert.deepEqual(complete,expected.view);assert.equal((await store.room(id)).state,expected.state);
 const g0=JSON.parse(before.state).game,g1=JSON.parse(expected.state).game;assert.equal(g1.players[0].deck.length,g0.players[0].deck.length-count);
 assert.equal(complete.pendingChoice,null);
 // The existing Fisher-Yates shuffle consumes no RNG for a zero/one-card deck.
 if(g0.players[0].deck.length>1)assert.notEqual(g1.random,g0.random);else assert.equal(g1.random,g0.random);
 now+=90000;assert.deepEqual(await service.command(id,'Bearer '+tokens[0],choose),complete);assert.equal((await store.room(id)).state,expected.state);
 await assert.rejects(service.command(id,'Bearer '+tokens[0],{...choose,expectedVersion:choose.expectedVersion+1}),e=>e.status===409);
 const receipts=(await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=? AND command_id=?').bind(id,choose.commandId).first()).n;assert.equal(receipts,1);
 t.diagnostic(JSON.stringify({engine:JSON.parse(abi.catalog()).engineVersion,preparedNativeLayout:true,privateOtherSeats:3,acceptedCount:count,pauseReopen:true,duplicateReceipt:true,exactNativeWasmD1State:true}));
});
