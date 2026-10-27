// Current ABI93 ordinary entry regression; no persisted room rewriting.
// Real entry/ready/start requests, without initial state or room DB injection.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {RoomStore} from '../src/store.mjs';
const worker=path.dirname(process.env.CURRENT_CLOCK_WORKER||fileURLToPath(new URL('../dist/current-clock/worker.js',import.meta.url)));
for(const societyId of ['MSJC04','MSJC05'])test(`${societyId} enters with a named complete deck, exposes six-card start and preserves it after D1 reopen`,async t=>{
  const persist=mkdtempSync(path.join(tmpdir(),'msjc0405-entry-'));
  const options=convertV4MiniflareOptions({name:'msjc0405-entry',resourcePersistencePath:persist,modulesRoot:worker,modules:[
    {type:'ESModule',path:path.join(worker,'worker.js')},
    ...readdirSync(worker).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:path.join(worker,f)}))],
    compatibilityDate:'2026-10-02',cf:false,bindings:{LOCAL_TEST_CLOCK_MS:'0'},d1Databases:{DB:'msjc0405-entry'}});
  let mf=new Miniflare(options),db=await mf.getD1Database('DB');t.after(()=>mf.dispose());
  for(const file of readdirSync('drizzle').filter(f=>f.endsWith('.sql')))
    for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  const api=async(route,body,session)=>{
    const response=await mf.dispatchFetch('http://localhost'+route,{method:body?'POST':'GET',headers:{...(body?{'Content-Type':'application/json'}:{}),...(session?{Authorization:'Bearer '+session.token}:{})},...(body?{body:JSON.stringify(body)}:{})});
    return {status:response.status,body:await response.json()};
  };
  const catalog=(await api('/api/catalog')).body;assert.equal(catalog.engineVersion,'rust-v0.2.113-bq001-anchor-source-wip');
  assert.equal(catalog.cards.length,278);
  for(const id of ['BQ033','BQ009','BQ001'])assert.equal(catalog.cards.find(card=>card.id===id).supported,false);
  assert.equal(catalog.cards.filter(card=>card.supported).length,236);assert.equal(catalog.societies.length,12);
  const draft=(id,count=25)=>{
    const society=catalog.societies.find(c=>c.id===id);let left=count;const cards=[];
    for(const card of catalog.cards.filter(c=>c.supported&&c.color===society.color&&['character','spell','attachment'].includes(c.kind))) {
      const n=Math.min(left,card.deckCopyLimit??left);if(n)cards.push({cardId:card.id,count:n});left-=n;
    }
    assert.equal(left,0);cards.push({cardId:'JC125',count:50-count});
    return {id:`named-${id}`,name:`已命名的${society.name}`,description:'完整牌组入桌快照，本地测试',societyId:id,cards,
      rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:'2026-10-17T00:00:00.000Z'};
  };
  assert.equal((await api('/api/rooms',{name:'不合法',mode:'teams',deckDraft:draft(societyId,24),requestId:crypto.randomUUID()})).status,400);
  const create={name:'甲',mode:'teams',deckDraft:draft(societyId),requestId:crypto.randomUUID()};
  const initial=await api('/api/rooms',create);assert.equal(initial.status,200);assert.deepEqual(await api('/api/rooms',create),initial);
  const other=societyId==='MSJC04'?'MSJC05':'MSJC04';const expected=[societyId,other,societyId,other];const sessions=[initial.body];
  for(let seat=1;seat<4;seat++) {
    const body={name:`玩家${seat}`,inviteCode:initial.body.inviteCode,deckDraft:draft(expected[seat]),requestId:crypto.randomUUID()};
    const joined=await api('/api/rooms/join',body);assert.equal(joined.status,200);assert.deepEqual(await api('/api/rooms/join',body),joined);sessions.push(joined.body);
  }
  const route=(seat,op)=>`/api/rooms/${sessions[seat].roomId}/${op}`;
  const views=async()=>{const all=[];for(let seat=0;seat<4;seat++){const r=await api(route(seat,'state'),null,sessions[seat]);assert.equal(r.status,200);all.push(r.body);}return all;};
  let all=await views(),last;
  for(let seat=0;seat<4;seat++) {
    assert(all[seat].societyZones.every(z=>z.card===null));
    assert.equal(all[seat].yourDeck.name,draft(expected[seat]).name);assert.equal(all[seat].yourDeck.societyId,expected[seat]);
    const command={commandId:crypto.randomUUID(),expectedVersion:all[seat].version,action:{kind:'game',action:{kind:'ready'}}};
    const result=await api(route(seat,'commands'),command,sessions[seat]);assert.equal(result.status,200);last={seat,command,result};all=await views();
  }
  const start=await api(route(0,'commands'),{commandId:crypto.randomUUID(),expectedVersion:all[0].version,action:{kind:'game',action:{kind:'start'}}},sessions[0]);assert.equal(start.status,200);
  all=await views();
  assert.deepEqual(all[0].players.map(p=>p.handCount),[6,6,6,6]);
  for(let viewer=0;viewer<4;viewer++) {
    assert.equal(all[viewer].yourDeck.societyId,expected[viewer]);
    assert.equal(all[viewer].yourDeck.name,draft(expected[viewer]).name);
    assert.equal(all[viewer].societyZones.length,4);
    assert.deepEqual(all[viewer].societyZones.map(z=>z.card.cardId),expected);
    all[viewer].societyZones.forEach((z,seat)=>{assert.equal(z.card.owner,`p${seat}`);assert.equal(z.card.controller,`p${seat}`);});
  }
  const opaque=(await new RoomStore(db).room(initial.body.roomId)).state;
  await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');
  assert.equal((await new RoomStore(db).room(initial.body.roomId)).state,opaque);
  assert.deepEqual(await views(),all);
  assert.deepEqual(await api(route(last.seat,'commands'),last.command,sessions[last.seat]),last.result);
  assert.deepEqual(await api('/api/rooms',create),initial);
  assert.equal((await new RoomStore(db).room(initial.body.roomId)).state,opaque);
  t.diagnostic(JSON.stringify({societyId,ordinaryEntryOnly:true,players:4,startingHands:[6,6,6,6],namedSnapshotsAndOriginalReceiptsRestored:true,localSyntheticDatabaseRetained:true}));
});
