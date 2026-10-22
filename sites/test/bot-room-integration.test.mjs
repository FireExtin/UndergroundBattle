import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { kernel as raw } from '../../rust-game-wasm/tests/current-kernel.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
import { createWorker } from '../src/http.mjs';
import { RoomStore } from '../src/store.mjs';
import { RoomService } from '../src/service.mjs';
import { restoreBotPolicy, botCandidateAlternatives } from '../src/bot-policy.mjs';
const kernel=routeKernels(raw),project=fileURLToPath(new URL('../',import.meta.url));
const hash=value=>createHash('sha256').update(typeof value==='string'?value:JSON.stringify(value)).digest('hex');
const seed='9007199254740993';
async function fixture(t,{enabled=true,defaultOptions=false}={}){
 const options=convertV4MiniflareOptions({name:'bot-room-d1',resourcePersistencePath:mkdtempSync(join(tmpdir(),'bot-room-d1-')),
   modules:[{type:'ESModule',path:fileURLToPath(new URL('./fixtures/bot-d1-worker.mjs',import.meta.url))}],
   compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'bot-room-d1'}});
 let mf=new Miniflare(options),db=await mf.getD1Database('DB'),now=0;
 t.after(()=>mf.dispose());
 for(const f of readdirSync(project+'drizzle').filter(f=>f.endsWith('.sql')))
  for(const sql of readFileSync(project+'drizzle/'+f,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const accesses=[];
 const settings={botEnabled:enabled,roomSeed:()=>seed,onBotView:event=>accesses.push(event)};
 const worker=defaultOptions?createWorker(kernel,()=>now):createWorker(kernel,()=>now,settings);
 const api=async(path,body,session)=>{
  const r=await worker.fetch(new Request('http://local'+path,{method:body?'POST':'GET',headers:{...(body?{'Content-Type':'application/json'}:{}),...(session?{Authorization:'Bearer '+session.token}:{})},...(body?{body:JSON.stringify(body)}:{})}),{DB:db});
  return {status:r.status,body:r.status===204?null:await r.json()};
 };
 const create=async(difficulty='normal',mode='duel',extra={})=>{
  const body={requestId:crypto.randomUUID(),name:'练习者',mode,deckId:'watchers',roomKind:'bot',difficulty,...extra};
  const response=await api('/api/rooms',body);assert.equal(response.status,200,response.body.message);return {session:response.body,body};
 };
 const count=async(table,id)=>{const r=await db.prepare(`SELECT count(*) AS n FROM ${table}${id?' WHERE '+(table==='rooms'?'id':'room_id')+'=?':''}`).bind(...(id?[id]:[])).first();return r.n;};
 const snapshot=async id=>hash(await Promise.all(['rooms','seats','commands','journal','entry_receipts'].map(async table=>(await db.prepare(`SELECT * FROM ${table} WHERE ${table==='rooms'?'id':'room_id'}=? ORDER BY rowid`).bind(id).all()).results)));
 return {api,create,count,snapshot,accesses,store:()=>new RoomStore(db),service:()=>new RoomService(db,kernel,()=>now,settings),
   command:(s,commandId,version,action)=>api(`/api/rooms/${s.roomId}/commands`,{commandId,expectedVersion:version,action},s),
   state:s=>api(`/api/rooms/${s.roomId}/state`,null,s),tick:n=>{now+=n;},
   reopen:async()=>{await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');},
   rows:async(id,table='journal')=>(await db.prepare(`SELECT * FROM ${table} WHERE room_id=? ORDER BY ${table==='journal'?'version':'rowid'}`).bind(id).all()).results};
}
function humanAction(view){
 const pick=action=>Object.fromEntries(Object.entries(action).filter(([k,v])=>['kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected','deckDraft'].includes(k)&&v!=null));
 if(view.pendingChoice){const c=view.pendingChoice,ids=c.options.map(o=>o.id);let a={kind:'choose',choiceId:c.id};
  if(['order','investigation'].includes(c.kind))a={...a,top:ids,bottom:[]};
  else if(c.kind==='damage')a={...a,allocations:{[ids[0]]:c.amount}};
  else a={...a,selected:ids.slice(0,c.min??0)};
  return {kind:'game',action:a};
 }
 if(view.responseWindow){const w=view.responseWindow,m=w.members.find(m=>m.playerId===view.you);if(!m||m.status==='passed')return null;
  return m.status==='composing'?{kind:'cancelAndPass',windowId:w.id,intentId:w.myIntentId}:{kind:'passResponse',windowId:w.id};}
 if(view.status==='lobby'){
  const you=view.players.find(p=>p.id===view.you),a=view.legalActions.find(a=>a.kind==='start')||(!you.ready&&view.legalActions.find(a=>a.kind==='ready'));
  return a?{kind:'game',action:pick(a)}:null;
 }
 const pass=view.legalActions.find(a=>a.kind==='pass');return pass?{kind:'game',action:pick(pass)}:null;
}
async function replay(f,session){
 const room=await f.store().room(session.roomId),rows=await f.rows(session.roomId);let state=room.initial_state,policy=null,botCommands=0;
 for(const row of rows){const e=JSON.parse(row.entry);if(row.version===0){policy=restoreBotPolicy(JSON.stringify(e.SessionEvents.botState));continue;}
  let next;
  if(e.Join)next=JSON.parse(kernel.joinGame(state,e.Join.name,e.Join.deck_id));
  else if(e.JoinWithDeck)next=JSON.parse(kernel.joinGameWithDeck(state,e.JoinWithDeck.name,JSON.stringify(e.JoinWithDeck.deck_draft)));
  else{assert(e.SessionEvents);const events=e.SessionEvents.events,command=events.find(e=>e.Command)?.Command,tick=events.find(e=>e.Tick)?.Tick;
   if(e.SessionEvents.botDecision){
    assert(command);const meta=e.SessionEvents.botDecision,authority={roomId:session.roomId,roomKind:'bot',difficulty:policy.difficulty,humanSeats:policy.humanSeats,botSeats:policy.botSeats};
    const candidates=botCandidateAlternatives(policy,authority,command.seat,seat=>JSON.parse(kernel.view(state,seat)));
    const chosen=candidates.find(c=>hash(c.action)===hash(command.command.action));assert(chosen,'recorded Bot action must be reproducible from persisted seed and legal views');
    assert.equal(hash(chosen.nextPolicy),hash(e.SessionEvents.botState));assert.equal(meta.beforeCounter,policy.counter);assert.equal(meta.afterCounter,chosen.nextPolicy.counter);
    policy=restoreBotPolicy(JSON.stringify(e.SessionEvents.botState));botCommands++;
   }
   next=command?JSON.parse(kernel.applyRoom(state,command.seat,JSON.stringify(command.command),String(command.server_now_ms))):JSON.parse(kernel.pollRoom(state,0,String(tick.server_now_ms)));
   assert.equal(hash(next.journal),hash(events));
  }
  assert.equal(next.version,row.version);state=next.state;
 }
 assert.equal(hash(state),hash(room.state));assert.equal((await f.service().botPolicy(session.roomId,room.version)).counter,String(botCommands));
 return {botCommands,version:room.version};
}

test('server gate is closed by default; fair creation cannot inherit Bot flags or privileges',async t=>{
 const f=await fixture(t,{enabled:false,defaultOptions:true});const catalog=await f.api('/api/catalog');assert(!catalog.body.botPractice);
 for(const extra of [{roomKind:'bot',difficulty:'hard'},{difficulty:'hard'},{roomKind:'human',difficulty:'hard'},{advantages:['peekHumanHands']},{botSeats:[1]},{seed:'1'}]){
  const r=await f.api('/api/rooms',{requestId:crypto.randomUUID(),name:'人类',mode:'duel',deckId:'watchers',...extra});assert.equal(r.status,400);
 }
 assert.equal(await f.count('rooms'),0);
 const human=await f.api('/api/rooms',{requestId:crypto.randomUUID(),name:'普通房',mode:'duel',deckId:'watchers'});assert.equal(human.status,200);assert(!human.body.view.botMatch);assert.equal(human.body.view.players.length,1);
});
test('Bot creation atomically fills and readies server seats, keeps one human credential and original receipt',async t=>{
 const f=await fixture(t);for(const mode of ['duel','teams']){
  const {session:s,body}=await f.create('hard',mode);assert.equal(s.view.you,'p0');assert.equal(s.view.players.length,mode==='teams'?4:2);
  assert(s.view.players.filter(p=>p.seat!==0).every(p=>p.ready));assert.equal(await f.count('seats',s.roomId),1);
  assert.equal(s.view.botMatch.difficulty,'hard');assert.deepEqual(s.view.botMatch.advantages,['peekHumanHands']);assert(!Object.hasOwn(s.view.botMatch,'seed'));
  const saved=await f.snapshot(s.roomId),again=await f.api('/api/rooms',body);assert.equal(again.status,200);assert.equal(hash(again.body),hash(s));assert.equal(await f.snapshot(s.roomId),saved);
  const room=await f.store().room(s.roomId),p=await f.service().botPolicy(s.roomId,room.version);assert.equal(p.seed,seed);assert(room.initial_state.includes('"seed":9007199254740993'));assert.equal(p.counter,'0');
  await replay(f,s);
 }
});
test('Bot invitations, forged server decision IDs and mode changes reject without writes',async t=>{
 const f=await fixture(t),{session:s}=await f.create('hard');const before=await f.snapshot(s.roomId);
 const joined=await f.api('/api/rooms/join',{requestId:crypto.randomUUID(),name:'外来玩家',inviteCode:s.inviteCode,deckId:'watchers'});assert.equal(joined.status,409);
 for(const extra of [{commandId:`bot:${s.roomId}:1:0`},{difficulty:'easy'},{botState:{counter:'1'}},{seed:'0'},{roomKind:'human'}]){
  const r=await f.api(`/api/rooms/${s.roomId}/commands`,{commandId:'human-forgery',expectedVersion:s.view.version,action:{kind:'game',action:{kind:'ready'}},...extra},s);assert.equal(r.status,400);
 }
 assert.equal(await f.snapshot(s.roomId),before);
});
test('a difficulty change under an original create ID cannot replace its durable Bot mode',async t=>{
 const f=await fixture(t),{session:s,body}=await f.create('normal'),before=await f.snapshot(s.roomId);
 const r=await f.api('/api/rooms',{...body,difficulty:'hard'});assert.equal(r.status,409);assert.equal(await f.snapshot(s.roomId),before);
 assert.equal((await f.service().botPolicy(s.roomId,s.view.version)).difficulty,'normal');
});
test('normal decision context never requests human views; hard hand-peek stays server-private',async t=>{
 const f=await fixture(t);for(const difficulty of ['normal','hard']){
  const {session:s}=await f.create(difficulty);let sequence=0,view=s.view;
  for(let n=0;n<35;n++){
   const state=await f.state(s);assert.equal(state.status,200);view=state.body;assert(!view.botAttention);
   const action=humanAction(view);if(action){const r=await f.command(s,`human-${difficulty}-${sequence++}`,view.version,action);assert.equal(r.status,200,r.body.message);}
   if(BigInt((await f.service().botPolicy(s.roomId,(await f.store().room(s.roomId)).version)).counter)>8n)break;
  }
  const calls=f.accesses.filter(c=>c.difficulty===difficulty);assert(calls.length>0);if(difficulty==='normal')assert(calls.every(c=>c.seat===c.botSeat));else assert(calls.some(c=>c.seat===0));
  assert(!Object.hasOwn(view,'humanHands'));assert(!JSON.stringify(view.botMatch).includes(seed));assert.equal(view.you,'p0');await replay(f,s);
 }
});
test('parallel state polls and restart of the host preserve one RNG step per accepted Bot command',async t=>{
 const f=await fixture(t),{session:s}=await f.create('normal');let view=s.view;
 for(let n=0;n<4;n++){const a=humanAction(view);if(a){const r=await f.command(s,`human-setup-${n}`,view.version,a);assert.equal(r.status,200,r.body.message);view=r.body;}}
 const states=await Promise.all(Array.from({length:4},()=>f.state(s)));assert(states.every(r=>r.status===200));
 const room=await f.store().room(s.roomId),policy=await f.service().botPolicy(s.roomId,room.version),records=await f.rows(s.roomId,'commands');
 assert.equal(Number(policy.counter),records.filter(r=>r.command_id.startsWith('bot:')).length);
 await f.reopen();const restored=await f.service().botPolicy(s.roomId,(await f.store().room(s.roomId)).version);assert.equal(hash(restored),hash(policy));assert.equal(restored.seed,seed);await replay(f,s);
});
test('pause stops Bot scheduling and preserves policy state across reopen',async t=>{
 const f=await fixture(t),{session:s}=await f.create('hard');let view=s.view;
 for(let n=0;n<4;n++){const a=humanAction(view);if(a){const r=await f.command(s,`human-pause-setup-${n}`,view.version,a);assert.equal(r.status,200,r.body.message);view=r.body;}}
 if(view.status==='lobby')view=(await f.state(s)).body;
 const paused=await f.command(s,'human-pause',view.version,{kind:'pauseRoom'});assert.equal(paused.status,200,paused.body.message);
 const room=await f.store().room(s.roomId),before=await f.service().botPolicy(s.roomId,room.version);await f.reopen();f.tick(20000);
 const polled=await f.state(s);assert.equal(polled.status,200);assert(polled.body.pause);assert.equal(hash(await f.service().botPolicy(s.roomId,(await f.store().room(s.roomId)).version)),hash(before));
});
test('real normal Bot match reaches a terminal game and its complete D1 journal replays exactly',async t=>{
 const f=await fixture(t),{session:s}=await f.create('normal');let commands=0,terminal=null;
 for(let n=0;n<1200;n++){
  const state=await f.state(s);assert.equal(state.status,200);const view=state.body;
  assert(!view.botAttention,`Bot blocked at ${view.phase}/${view.step}: ${view.pendingChoice?.kind||'none'}`);
  if(view.status==='finished'){terminal=view;break;}
  const action=humanAction(view);if(action){const r=await f.command(s,`human-natural-${commands++}`,view.version,action);assert.equal(r.status,200,r.body.message);}
 }
 assert(terminal,'bounded legal-view match must finish');assert(Number.isInteger(terminal.winnerTeam));const replayed=await replay(f,s);assert(replayed.botCommands>10);
 t.diagnostic(JSON.stringify({scope:'author local Bot smoke; human keeps/passses, not full-card or fair-mechanism acceptance',humanCommands:commands,botCommands:replayed.botCommands,turn:terminal.turn,winnerTeam:terminal.winnerTeam}));
});
