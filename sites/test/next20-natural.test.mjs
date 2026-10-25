// Only candidate construction is admitted by the Native test scope. Production
// creation/start stays closed. Replay begins at the genuine Native started state.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {kernel as abi,moduleBytes} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
import {RoomStore} from '../src/store.mjs';
import {RoomService,digest} from '../src/service.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
abi.initSync({module:moduleBytes});

for(const name of ['bq034-natural-entry-counter','bq034-natural-shown-feed','cq25-xg25-natural-seal-move-search']){
  test(`natural candidate ${name}: full Native commands, every receipt and persisted restore`,async t=>{
    const s=loadCurrentFixture(`rust-game-wasm/tests/current112/${name}.json.gz`);
    assert.equal(s.publicAdmission,false);assert.deepEqual(s.candidateConstructionOnly,['BQ034','CQ25','XG25']);
    assert.deepEqual(JSON.parse(abi.catalog()),s.catalog);
    for(const id of ['BQ034','CQ25','XG25','BQ009','BQ033'])assert.equal(s.catalog.cards.find(c=>c.id===id).supported,false);
    // This is deliberately not public admission or a replay of overridden start.
    const start=s.nativeSetup[2];assert.equal(start.command.action.action.kind,'start');
    const rejected=JSON.parse(abi.applyRoom(start.state,start.seat,JSON.stringify(start.command),'0'));
    assert.equal(rejected.outcome,'rejected');assert.equal(rejected.state,start.state);
    assert.match(rejected.errorMessage,/不能加入当前玩家卡组/);
    const persist=mkdtempSync(join(tmpdir(),'next20-natural-d1-'));
    const options=convertV4MiniflareOptions({name:'next20-natural',resourcePersistencePath:persist,modules:[
      {type:'ESModule',path:resolve('dist/server/index.js')},
      ...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)})),
    ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'next20-natural'}});
    let mf=new Miniflare(options),db=await mf.getD1Database('DB');
    t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
    for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql'))){
      for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(v=>v.trim()))await db.prepare(sql).run();
    }
    const initial=s.startedState,views=JSON.parse(abi.view(initial,0)),id=views.roomId;
    const tokens=['1'.repeat(64),'2'.repeat(64)];
    let store=new RoomStore(db),service=new RoomService(db,routeKernels(abi),()=>0),reopened=0;
    await store.create({id,invite:views.inviteCode,state:initial,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),
      requestHash:crypto.randomUUID(),intentHash:'explicit-candidate-native-start',response:'{}'});
    await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(views.version,id).run();
    await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,1,await digest(tokens[1])).run();
    let pausedReopened=false;
    const reopen=async()=>{await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);service=new RoomService(db,routeKernels(abi),()=>0);reopened++;};
    for(let i=0;i<s.steps.length;i++){
      const row=s.steps[i];assert.equal((await store.room(id)).state,row.state);
      const actual=await service.command(id,'Bearer '+tokens[row.seat],row.command);
      assert.deepEqual(actual,row.expected.view);
      const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,row.expected.version).first();
      assertSessionJournal(journal.entry,row.expected.journal);
      assert.equal((await store.room(id)).state,row.expected.state);
      for(let seat=0;seat<2;seat++)assert.deepEqual(await service.state(id,'Bearer '+tokens[seat],null),row.views[seat]);
      const beforeDuplicate=await store.room(id);
      // HTTP receipt path uses the real Worker clock. An old command still
      // returns its original body before any new poll/tick or mutation.
      const duplicate=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/commands`,{method:'POST',headers:{'Content-Type':'application/json',Authorization:'Bearer '+tokens[row.seat]},body:JSON.stringify(row.command)});
      assert.equal(duplicate.status,200);assertPublicView(await duplicate.json(),row.expected.view);
      assert.deepEqual(await store.room(id),beforeDuplicate);
      assert.equal((await db.prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,i+1);
      if(i===0||i===Math.floor(s.steps.length/2)||row.expected.view.pause&&!pausedReopened){
        if(row.expected.view.pause)pausedReopened=true;await reopen();
        assert.equal((await store.room(id)).state,row.expected.state);
        for(let seat=0;seat<2;seat++)assert.deepEqual(await service.state(id,'Bearer '+tokens[seat],null),row.views[seat]);
      }
    }
    assert(pausedReopened);const saved=await store.room(id);await reopen();assert.deepEqual(await store.room(id),saved);
    const first=s.steps[0];const receipt=await service.command(id,'Bearer '+tokens[first.seat],first.command);
    assert.deepEqual(receipt,first.expected.view);assert.deepEqual(await store.room(id),saved);
    t.diagnostic(JSON.stringify({seed:s.seedDecimal,nativeSetupCommands:3,publicStartRejected:true,commands:s.steps.length,
      everyCommandOriginalHttpReceipt:true,everyCommandPersistedStateAndViews:true,journalsEqual:true,reopened,nativeDeckConstructionOnly:true,publicAdmission:false}));
  });
}
