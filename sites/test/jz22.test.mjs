// Prepared native states through actual workerd/D1, persistence and duplicate receipts.
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
abi.initSync({module:readFileSync('generated/hegemony_wasm_bg.wasm')});
const cases=JSON.parse(readFileSync(new URL('./fixtures/jz22-native-v054.json',import.meta.url),'utf8'));
for(const c of cases)test(`jz22-blue54 ${c.name}: native transition, D1 reopening, privacy and duplicate receipt`,async t=>{
 const persist=mkdtempSync(join(tmpdir(),'jz22-blue54-d1-'));
 const options=convertV4MiniflareOptions({name:'jz22-blue54-test',resourcePersistencePath:persist,modules:[
  {type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))
 ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'jz22-blue54-test'}});
 let mf=new Miniflare(options);t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
 let db=await mf.getD1Database('DB');for(const f of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+f,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const initial=JSON.parse(abi.view(c.state,0)),id=initial.roomId,tokens=['a','b','c','d'].map(x=>x.repeat(64));
 let now=0,store=new RoomStore(db),service=new RoomService(db,routeKernels(abi),()=>now);
 await store.create({id,invite:initial.inviteCode,state:c.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-native-jz22-layout',response:'{}'});
 await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
 for(let s=1;s<4;s++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,s,await digest(tokens[s])).run();
 const view=await service.command(id,'Bearer '+tokens[c.seat],c.command);assert.deepEqual(view,c.expected.view);assert.equal((await store.room(id)).state,c.expected.state);
 for(let seat=0;seat<4;seat++)assert.deepEqual(await service.state(id,'Bearer '+tokens[seat],null),c.views[seat]);
 const saved=await store.room(id);await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);service=new RoomService(db,routeKernels(abi),()=>now);
 assert.equal((await store.room(id)).state,saved.state);now=600000;
 assert.deepEqual(await service.command(id,'Bearer '+tokens[c.seat],c.command),view);
 await assert.rejects(service.command(id,'Bearer '+tokens[c.seat],{...c.command,expectedVersion:c.command.expectedVersion+1}),e=>e.status===409&&/同一请求标识/.test(e.message));
 assert.deepEqual(await store.room(id),saved);
 assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,1);
 t.diagnostic(JSON.stringify({engine:JSON.parse(abi.catalog()).engineVersion,preparedNativeLayout:true,inputName:c.inputName,completeOpaqueStateEqual:true,fourSeatViewsEqual:true,persistReopen:true,duplicateOriginalReceipt:true}));
});
