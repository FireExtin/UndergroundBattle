import { loadCurrentFixture } from '../../tools/current-native-source.mjs';
import { assertSessionJournal, assertPublicView } from './assert-session-journal.mjs';
// Prepared actual Native93 cases; local workerd/D1, not browser or online play.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import { kernel as abi, moduleBytes as currentBytes } from '../../rust-game-wasm/tests/current-kernel.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
import {RoomStore} from '../src/store.mjs';
import {RoomService,digest} from '../src/service.mjs';
abi.initSync({module:currentBytes});
const fixture=loadCurrentFixture('rust-game-wasm/tests/current111/jc052-worker.json.gz');
assert.deepEqual(JSON.parse(abi.catalog()),fixture.catalog);

for(const row of fixture.cases) test(`JC052 ${row.name}: complete Native command, D1 reopen and original HTTP receipt`,async t=>{
  const persist=mkdtempSync(join(tmpdir(),'jc052-73-d1-'));
  const options=convertV4MiniflareOptions({name:'jc052-73',resourcePersistencePath:persist,modules:[
    {type:'ESModule',path:resolve('dist/server/index.js')},
    ...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))
  ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'jc052-test-db'}});
  let mf=new Miniflare(options),db=await mf.getD1Database('DB');
  t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
  for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))
    for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim())) await db.prepare(sql).run();
  const id=row.beforeViews[0].roomId,tokens=row.beforeViews.map((_,seat)=>String(seat+1).repeat(64));
  let store=new RoomStore(db),service=new RoomService(db,routeKernels(abi),()=>0);
  await store.create({id,invite:row.beforeViews[0].inviteCode,state:row.before,nonce:crypto.randomUUID(),
    tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-jc052-native-layout',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(row.beforeViews[0].version,id).run();
  for(let seat=1;seat<tokens.length;seat++) await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  const views=async expected=>{
    for(let seat=0;seat<tokens.length;seat++) {
      const response=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/state`,{headers:{Authorization:'Bearer '+tokens[seat]}});
      assert.equal(response.status,200);const view=await response.json();
      assert.deepEqual({...view,serverNowMs:expected[seat].serverNowMs},expected[seat]);
    }
  };
  await views(row.beforeViews);assert.equal((await store.room(id)).state,row.before);
  if(row.name==='leader-sacrifice') {
    const command={...row.command,commandId:'empty-forced-leader',action:{kind:'game',action:{...row.command.action.action,selected:[]}}};
    await assert.rejects(service.command(id,'Bearer '+tokens[row.seat],command),error=>error.status===400);
    assert.equal((await store.room(id)).state,row.before);
    for(const table of ['commands','journal']) assert.equal((await db.prepare(`SELECT COUNT(*) AS n FROM ${table} WHERE room_id=?`).bind(id).first()).n,0);
  }
  const view=await service.command(id,'Bearer '+tokens[row.seat],row.command);
  assert.deepEqual(view,row.expected.view);assert.equal((await store.room(id)).state,row.expected.state);
  // HTTP uses wall clock and would legitimately expire this prepared 0ms window.
  // The injected service clock verifies all exact Native after-projections instead.
  for(let seat=0;seat<tokens.length;seat++)
    assert.deepEqual(await service.state(id,'Bearer '+tokens[seat],null),row.afterViews[seat]);
  const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=?').bind(id).first();
  assertSessionJournal(journal.entry, row.expected.journal);
  await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);
  assert.equal((await store.room(id)).state,row.expected.state);
  const duplicate=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/commands`,{method:'POST',headers:{'Content-Type':'application/json',Authorization:'Bearer '+tokens[row.seat]},body:JSON.stringify(row.command)});
  assert.equal(duplicate.status,200);assertPublicView(await duplicate.json(),view);
  assert.equal((await store.room(id)).state,row.expected.state);
  for(const table of ['commands','journal']) assert.equal((await db.prepare(`SELECT COUNT(*) AS n FROM ${table} WHERE room_id=?`).bind(id).first()).n,1);
});
