// Public HTTP admission and legal trajectories. No candidate registry or D1 room injection.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve,dirname} from 'node:path';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {kernel as abi,moduleBytes} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {RoomStore} from '../src/store.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
abi.initSync({module:moduleBytes});
const names=['bq034-natural-entry-counter','bq034-natural-shown-feed','cq25-xg25-natural-seal-move-search','bq034-natural-two-unique-sacrifice'];
const key=()=>crypto.randomUUID();
async function harness(t,worker,seed){
 const persist=mkdtempSync(join(tmpdir(),'next20-public-d1-')),directory=dirname(worker);
 const options=convertV4MiniflareOptions({name:'next20-public',resourcePersistencePath:persist,modules:[
  {type:'ESModule',path:worker},...readdirSync(directory).filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:join(directory,n)}))],
  compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'next20-public'},bindings:{LOCAL_TEST_CLOCK_MS:'0',LOCAL_TEST_SEED:String(seed)}});
 let mf=new Miniflare(options),db=await mf.getD1Database('DB');
 t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
 for(const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql'))){for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(v=>v.trim()))await db.prepare(sql).run();}
 return {db:()=>db,store:()=>new RoomStore(db),reopen:async()=>{await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');},
  api:async(path,body,session)=>{const r=await mf.dispatchFetch('http://localhost'+path,{method:body?'POST':'GET',headers:{...(body?{'Content-Type':'application/json'}:{}),...(session?{Authorization:'Bearer '+session.token}:{})},...(body?{body:JSON.stringify(body)}:{})});return{status:r.status,body:r.status===204?null:await r.json()};}};
}
for(const name of names){test(`public next20 ${name}: HTTP create/join/start, every command receipt and cold restore`,async t=>{
 const s=loadCurrentFixture(`rust-game-wasm/tests/current112/${name}.json.gz`);assert.equal(s.publicAdmission,true);assert.deepEqual(s.candidateConstructionOnly,[]);
 assert.deepEqual(JSON.parse(abi.catalog()),s.catalog);
 for(const id of ['BQ034','CQ25','XG25'])assert.equal(s.catalog.cards.find(c=>c.id===id).supported,true);
 for(const id of ['BQ009','BQ033'])assert.equal(s.catalog.cards.find(c=>c.id===id).supported,false);
 assert(process.env.NEXT20_PUBLIC_WORKER,'Build the local fixed-seed/clock HTTP wrapper using the exact production WASM');
 const h=await harness(t,resolve(process.env.NEXT20_PUBLIC_WORKER),s.seedDecimal),api=h.api;
 const counts=async()=>{const r=await h.db().batch(['rooms','seats','entry_receipts'].map(table=>h.db().prepare(`SELECT COUNT(*) AS n FROM ${table}`)));return r.map(x=>x.results[0].n);};
 const empty=await counts();
 for(const id of ['BQ009','BQ033']){const draft=structuredClone(s.decks[0]);draft.cards.push({cardId:id,count:1});assert.equal((await api('/api/rooms',{requestId:key(),name:'P0',mode:'duel',deckDraft:draft})).status,400);assert.deepEqual(await counts(),empty);}
 const tooMany=structuredClone(s.decks[0]);tooMany.cards=tooMany.cards.filter(e=>e.cardId!=='BQ034');tooMany.cards.push({cardId:'BQ034',count:4});
 assert.equal((await api('/api/rooms',{requestId:key(),name:'P0',mode:'duel',deckDraft:tooMany})).status,400);assert.deepEqual(await counts(),empty);
 const createBody={requestId:key(),name:'P0',mode:'duel',deckDraft:s.decks[0]},created=await api('/api/rooms',createBody);assert.equal(created.status,200);const host=created.body;
 assert.deepEqual(await api('/api/rooms',createBody),created);assert.deepEqual(await counts(),[1,1,1]);
 const badJoin=structuredClone(s.decks[1]);badJoin.cards.push({cardId:'BQ009',count:1});
 assert.equal((await api('/api/rooms/join',{requestId:key(),name:'P1',inviteCode:host.inviteCode,deckDraft:badJoin})).status,400);assert.deepEqual(await counts(),[1,1,1]);
 const joinBody={requestId:key(),name:'P1',inviteCode:host.inviteCode,deckDraft:s.decks[1]},joined=await api('/api/rooms/join',joinBody);assert.equal(joined.status,200);const sessions=[host,joined.body];
 assert.deepEqual(await api('/api/rooms/join',joinBody),joined);assert.deepEqual(await counts(),[1,2,2]);
 // Only the unpredictable public room ID/invite differ from Native's deterministic labels.
 // Literal replacements preserve opaque u64 text and every gameplay field unchanged.
 const remap=x=>typeof x==='string'?x.replaceAll(s.nativeRoomId,host.roomId).replaceAll(s.nativeInviteCode,host.inviteCode):Array.isArray(x)?x.map(remap):x&&typeof x==='object'?Object.fromEntries(Object.entries(x).map(([k,v])=>[k,remap(v)])):x;
 assert.equal((await h.store().room(host.roomId)).state,remap(s.initialState));
 let reopens=0,paused=false,firstReceipt;
 for(let i=0;i<s.steps.length;i++){
  const row=s.steps[i],command=remap(row.command),expected=remap(row.expected),path=`/api/rooms/${host.roomId}/commands`;
  assert.equal((await h.store().room(host.roomId)).state,remap(row.state));
  const actual=await api(path,command,sessions[row.seat]);assert.equal(actual.status,200,`${name} command ${i}`);assertPublicView(actual.body,expected.view);if(i===0)firstReceipt=actual;
  const saved=await h.store().room(host.roomId);assert.equal(saved.state,expected.state);
  const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(host.roomId,expected.version).first();assertSessionJournal(journal.entry,expected.journal);
  for(let seat=0;seat<2;seat++){const refresh=await api(`/api/rooms/${host.roomId}/state`,null,sessions[seat]);assert.equal(refresh.status,200);assertPublicView(refresh.body,remap(row.views[seat]));}
  assert.deepEqual(await api(path,command,sessions[row.seat]),actual);assert.deepEqual(await h.store().room(host.roomId),saved);
  assert.equal((await h.db().prepare('SELECT COUNT(*) AS n FROM commands WHERE room_id=?').bind(host.roomId).first()).n,i+1);
  if(i===0||i===Math.floor(s.steps.length/2)||expected.view.pause&&!paused){if(expected.view.pause)paused=true;await h.reopen();reopens++;assert.equal((await h.store().room(host.roomId)).state,expected.state);
   for(let seat=0;seat<2;seat++){const v=await api(`/api/rooms/${host.roomId}/state`,null,sessions[seat]);assert.equal(v.status,200);assertPublicView(v.body,remap(row.views[seat]));}}
 }
 assert(paused);const saved=await h.store().room(host.roomId);await h.reopen();reopens++;assert.deepEqual(await h.store().room(host.roomId),saved);
 assert.deepEqual(await api(`/api/rooms/${host.roomId}/commands`,remap(s.steps[0].command),sessions[s.steps[0].seat]),firstReceipt);assert.deepEqual(await h.store().room(host.roomId),saved);
 t.diagnostic(JSON.stringify({seed:s.seedDecimal,publicAdmission:true,candidateOverride:false,D1StateInjection:false,HTTPCreateJoinStart:true,commands:s.steps.length,everyReceiptAndTwoSeatRefresh:true,reopens,identityNormalizationOnly:['roomId','inviteCode']}));
});}
test('unwrapped production Worker admits all three cards through public create join start and resumes after cold reopen',async t=>{
 const a=loadCurrentFixture('rust-game-wasm/tests/current112/cq25-xg25-natural-seal-move-search.json.gz'),b=loadCurrentFixture('rust-game-wasm/tests/current112/bq034-natural-two-unique-sacrifice.json.gz');
 const h=await harness(t,resolve('dist/server/index.js'),0),api=h.api;
 const host=await api('/api/rooms',{requestId:key(),name:'Production P0',mode:'duel',deckDraft:a.decks[0]});assert.equal(host.status,200);
 const guest=await api('/api/rooms/join',{requestId:key(),name:'Production P1',inviteCode:host.body.inviteCode,deckDraft:b.decks[0]});assert.equal(guest.status,200);
 const sessions=[host.body,guest.body];let version=guest.body.view.version,n=0;
 const send=async(seat,action)=>{const command={commandId:key(),expectedVersion:version,action},response=await api(`/api/rooms/${host.body.roomId}/commands`,command,sessions[seat]);assert.equal(response.status,200);version=response.body.version;
  assert.deepEqual(await api(`/api/rooms/${host.body.roomId}/commands`,command,sessions[seat]),response);n++;return response.body;};
 for(let seat=0;seat<2;seat++)await send(seat,{kind:'game',action:{kind:'ready'}});
 const start=await send(0,{kind:'game',action:{kind:'start'}});assert.equal(start.status,'playing');
 const pause=await send(0,{kind:'pauseRoom'});assert(pause.pause);const stored=await h.store().room(host.body.roomId);await h.reopen();assert.deepEqual(await h.store().room(host.body.roomId),stored);
 const resumed=await send(0,{kind:'resumeRoom'});assert.equal(resumed.pause,undefined);assert.equal(resumed.status,'playing');
 for(let seat=0;seat<2;seat++)assert.equal((await api(`/api/rooms/${host.body.roomId}/state`,null,sessions[seat])).status,200);
 t.diagnostic(JSON.stringify({productionBundleUnwrapped:true,publicCreateJoinStart:true,cards:['BQ034','CQ25','XG25'],realRandomSeed:true,commandReceipts:n,coldReopens:1}));
});
