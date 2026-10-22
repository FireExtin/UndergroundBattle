// One fresh current Native witness in local D1. This neither publishes nor
// substitutes for the consolidated complete-batch acceptance run.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {kernel} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {RoomStore} from '../src/store.mjs';
import {RoomService,digest} from '../src/service.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
for(const [label,fixture] of [['attached adviser domains and real Hide','abi94-adviser.current.json.gz']]){
const native=loadCurrentFixture('docs/evidence/jc060-attached-character-2026-10-22/'+fixture);
test(`ABI94 ${label} and four private views survive D1 reopen and identical receipts`,async t=>{
  const persist=mkdtempSync(join(tmpdir(),'batch3-abi94-d1-'));
  const options=convertV4MiniflareOptions({name:'batch3-abi94',resourcePersistencePath:persist,cf:false,
    modules:[{type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))],compatibilityDate:'2026-10-02',d1Databases:{DB:'batch3-abi94'}});
  let mf=new Miniflare(options);t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
  let db=await mf.getD1Database('DB');
  for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const first=native.initial.views[0],id=first.roomId,credentials=['a','b','c','d'].map(c=>c.repeat(64));
  let store=new RoomStore(db),service=new RoomService(db,routeKernels(kernel),()=>0);
  await store.create({id,invite:first.inviteCode,state:native.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(credentials[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-native94-smoke',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(first.version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(credentials[seat])).run();
  assert(native.initial.state.includes(`"seed":${native.seedDecimal}`));
  for(let i=0;i<native.steps.length;i++){
    const step=native.steps[i],auth='Bearer '+credentials[step.seat];
    assert.equal((await store.room(id)).state,step.state);
    const result=await service.command(id,auth,step.command);assertPublicView(result,step.expected.view);
    assert.equal((await store.room(id)).state,step.expected.state);
    const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,step.expected.version).first();assertSessionJournal(journal.entry,step.expected.journal);
    const retry=await service.command(id,auth,step.command);assert.deepEqual(retry,result);assert.equal((await store.room(id)).state,step.expected.state);
    for(let seat=0;seat<4;seat++)assertPublicView(await service.state(id,'Bearer '+credentials[seat],null),step.views[seat]);
    if(i===Math.floor(native.steps.length/2)){
      await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);service=new RoomService(db,routeKernels(kernel),()=>0);assert.equal((await store.room(id)).state,step.expected.state);
      for(let seat=0;seat<4;seat++)assertPublicView(await service.state(id,'Bearer '+credentials[seat],null),step.views[seat]);
      assert.deepEqual(await service.command(id,auth,step.command),result);
      assert.equal((await store.room(id)).state,step.expected.state);
    }
  }
  const receipts=await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first();assert.equal(receipts.n,native.steps.length);
  const journalCount=await db.prepare('SELECT COUNT(*) AS n FROM journal WHERE room_id=?').bind(id).first();assert.equal(journalCount.n,native.steps.length);
  t.diagnostic(JSON.stringify({engine:native.catalog.engineVersion,nativeCommands:native.steps.length,privateViews:native.steps.length*4,reopened:true,identicalReceipts:true,opaqueU64Preserved:true,publicAdmissions:0}));
});
}
