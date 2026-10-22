// Fresh offline Native layouts in local D1; actual current Room commands.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {loadCurrentFixture,currentWasmSha256,sha256} from '../../tools/current-native-source.mjs';
import {catalog} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {kernel} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {RoomStore} from '../src/store.mjs';
import {RoomService,digest} from '../src/service.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const native=loadCurrentFixture('docs/evidence/jc060-attached-character-2026-10-22/abi94-society.current.json.gz');
const modules=[{type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))];
assert.equal(modules.filter(m=>m.type==='CompiledWasm').length,1,'Package only the current kernel');
assert.equal(sha256(readFileSync(modules[1].path)),currentWasmSha256,'Worker embeds corrected current bytes');
for(const [index,c] of native.cases.entries())test(`JC005 Society case ${index}: D1 receipts, rejected fees and response reopen`,async t=>{
  const persist=mkdtempSync(join(tmpdir(),'jc005-society-d1-'));
  const options=convertV4MiniflareOptions({name:'jc005-society',resourcePersistencePath:persist,cf:false,modules,compatibilityDate:'2026-10-02',d1Databases:{DB:'jc005-society'}});
  let mf=new Miniflare(options);t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
  const health=await mf.dispatchFetch('http://local.test/api/health');assert.equal(health.status,200);assert.equal((await health.json()).engineVersion,catalog.engineVersion);
  const serverCatalog=await mf.dispatchFetch('http://local.test/api/catalog');assert.equal(serverCatalog.status,200);
  const {transport,entryIdempotency,...actualCatalog}=await serverCatalog.json();assert.deepEqual(actualCatalog,catalog);
  let db=await mf.getD1Database('DB');
  for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const first=c.initial.views[0],id=first.roomId,credentials=['a','b','c','d'].map(v=>v.repeat(64));
  let store=new RoomStore(db),service=new RoomService(db,routeKernels(kernel),()=>0);
  await store.create({id,invite:first.inviteCode,state:c.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(credentials[0]),requestHash:crypto.randomUUID(),intentHash:'fresh-native94-society-offline-layout',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(first.version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(credentials[seat])).run();
  const receipts=new Map();let reopened=0;
  for(const step of c.steps){
    const auth='Bearer '+credentials[step.seat];assert.equal((await store.room(id)).state,step.state);
    if(step.expected.outcome==='accepted'){
      const result=await service.command(id,auth,step.command);assertPublicView(result,step.expected.view);
      receipts.set(step.command.commandId,result);
      assert.deepEqual(await service.command(id,auth,step.command),result);
      const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,step.expected.version).first();assertSessionJournal(journal.entry,step.expected.journal);
    }else if(receipts.has(step.command.commandId)){
      // Reducer reports stale version; the server must return its original
      // accepted receipt, with neither a second payment nor journal event.
      assert.deepEqual(await service.command(id,auth,step.command),receipts.get(step.command.commandId));
    }else await assert.rejects(service.command(id,auth,step.command),error=>error.status===400&&error.code===step.expected.errorCode);
    assert.equal((await store.room(id)).state,step.expected.state);
    for(let seat=0;seat<4;seat++)assertPublicView(await service.state(id,'Bearer '+credentials[seat],null),step.views[seat]);
    if(['beginResponse','submitResponse'].includes(step.command.action.kind)&&step.expected.outcome==='accepted'){
      await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);service=new RoomService(db,routeKernels(kernel),()=>0);reopened++;
      assert.equal((await store.room(id)).state,step.expected.state);
      assert.deepEqual(await service.command(id,auth,step.command),receipts.get(step.command.commandId));
      for(let seat=0;seat<4;seat++)assertPublicView(await service.state(id,'Bearer '+credentials[seat],null),step.views[seat]);
    }
  }
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,receipts.size);
  assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM journal WHERE room_id=?').bind(id).first()).n,receipts.size);
  assert.equal(reopened,c.ownExhaustedMind?2:0);
  assert((await store.room(id)).state.includes(`"seed":${c.seedDecimal}`));
  t.diagnostic(JSON.stringify({case:index,commands:c.steps.length,acceptedReceipts:receipts.size,reopened,workerHealth:200,workerCatalog:200,workerWasmSha256:currentWasmSha256,opaqueHighU64Preserved:true,scope:'Author targeted local Worker/D1 integration; not public or browser acceptance'}));
});
