// Synthetic Native layouts; actual production WASM/RoomService/workerd/D1.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import * as abi from '../generated/hegemony_wasm.js';
import { routeKernels } from '../src/kernel-router.mjs';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';

abi.initSync({ module:readFileSync('generated/hegemony_wasm_bg.wasm') });
const fixtures = JSON.parse(readFileSync(new URL('./fixtures/xq24-native-v076.json', import.meta.url)));
for (const scenario of fixtures.scenarios) test(`XQ24 ${scenario.name}: Native journals, continuous D1 restore and original receipts`, async t => {
  const persist = mkdtempSync(join(tmpdir(), 'xq24-d1-'));
  const options = convertV4MiniflareOptions({ name:'xq24-test', resourcePersistencePath:persist, modules:[
    { type:'ESModule', path:resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({ type:'CompiledWasm', path:resolve('dist/server',n) }))
  ], compatibilityDate:'2026-10-02', cf:false, d1Databases:{DB:'xq24-test'} });
  let mf = new Miniflare(options), db = await mf.getD1Database('DB');
  t.after(async()=>{ await mf.dispose(); rmSync(persist,{recursive:true,force:true}); });
  for (const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim())) await db.prepare(sql).run();
  }
  const first=scenario.steps[0], initial=JSON.parse(abi.view(first.state,0)), id=initial.roomId;
  const tokens=initial.players.map((_,s)=>String(s+1).repeat(64)); // Synthetic test credentials only.
  let store=new RoomStore(db), service=new RoomService(db,routeKernels(abi),()=>0);
  await store.create({id,invite:initial.inviteCode,state:first.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),
    requestHash:crypto.randomUUID(),intentHash:'explicit-xq24-native-layout',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
  for (let s=1;s<tokens.length;s++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,s,await digest(tokens[s])).run();
  // Reopen before the first command, including already declared response fixtures.
  const reopen=async state=>{
    await mf.dispose(); mf=new Miniflare(options); db=await mf.getD1Database('DB');
    store=new RoomStore(db); service=new RoomService(db,routeKernels(abi),()=>0);
    assert.equal((await store.room(id)).state,state);
  };
  await reopen(first.state); let reopened=1;
  for (const step of scenario.steps) {
    assert.equal((await store.room(id)).state,step.state);
    const view=await service.command(id,'Bearer '+tokens[step.seat],step.command);
    assert.deepEqual(view,step.expected.view);
    assert.equal((await store.room(id)).state,step.expected.state);
    const row=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,step.expected.version).first();
    assert.deepEqual(JSON.parse(row.entry),{SessionEvents:{events:step.expected.journal}});
    for(let s=0;s<tokens.length;s++) assert.deepEqual(await service.state(id,'Bearer '+tokens[s],null),step.views[s]);
    // Every pause and resume crosses a fresh workerd + SQLite instance.
    if(['pauseRoom','resumeRoom'].includes(step.command.action.kind)) {await reopen(step.expected.state);reopened++;}
    assert.deepEqual(await service.command(id,'Bearer '+tokens[step.seat],step.command),view);
  }
  const last=scenario.steps.at(-1), saved=await store.room(id);
  await assert.rejects(service.command(id,'Bearer '+tokens[last.seat],{...last.command,expectedVersion:last.command.expectedVersion+1}),e=>e.status===409);
  assert.deepEqual(await store.room(id),saved);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,scenario.steps.length);
  t.diagnostic(JSON.stringify({preparedNativeLayout:true,commands:scenario.steps.length,reopened,fullJournals:true,opaqueStates:true,seatViews:tokens.length,originalReceipts:true}));
});
