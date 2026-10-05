// Explicit local offline fixtures: production Worker/D1 protocol, never a live-state injection.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,mkdtempSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {tmpdir} from 'node:os';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
import {lazyKernel} from '../src/lazy-kernel.mjs';
const source=readFileSync('src/kernel.mjs','utf8');
async function load(folder){const abi=await import(pathToFileURL(resolve('rust-game-wasm',folder,'hegemony_wasm.js')));const bytes=readFileSync(resolve('rust-game-wasm',folder,'hegemony_wasm_bg.wasm'));abi.initSync({module:bytes});return{abi,bytes,catalog:JSON.parse(abi.catalog())};}
const current=await load('pkg');
const withoutClock=v=>{const {serverNowMs,...stable}=v;return stable;};

test('all 36 actual identities preserve old rooms and new rooms use accepted34',async()=>{
 const imports=new Map([...source.matchAll(/import \* as (\w+) from '..\/generated\/([^']+)\/hegemony_wasm.js';/g)].map(m=>[m[1],m[2]]));
 const rows=[...source.matchAll(/const (\w+) = lazyKernel\((\w+), (\w+), (\{[^\n]+\})\);/g)];
 assert.equal(rows.length,36);
 const kernels=[];
 for(const row of rows){const folder=row[2]==='current'?'pkg':imports.get(row[2]);assert(folder,row[2]);const loaded=folder==='pkg'?current:await load(folder);const identity=JSON.parse(row[4]);for(const field of ['rulesVersion','cardPoolVersion','engineVersion'])assert.equal(loaded.catalog[field],identity[field]);kernels.push({abi:loaded.abi,identity,lazy:lazyKernel(loaded.abi,loaded.bytes,identity)});}
 const routed=routeKernels(kernels[0].lazy,kernels.slice(1).map(k=>k.lazy));
 for(const {abi,identity}of kernels){const room=JSON.parse(abi.newGame('0123456789abcdef01234567','OFFLINE','duel','P0','watchers','18446744073709551615'));assert.deepEqual(JSON.parse(routed.view(room.state,0)),room.view);assert.equal(routed.catalog(room.state),abi.catalog());assert.equal(routed.joinGame(room.state,'P1','hunters'),abi.joinGame(room.state,'P1','hunters'));assert.equal(routed.supportsPacing(room.state),!!abi.pollRoom);assert.equal(JSON.parse(routed.view(room.state,0)).versions.engine,identity.engineVersion);}
 assert.equal(JSON.parse(routed.newGame('0123456789abcdef01234567','NEW','duel','P0','watchers','1')).view.versions.engine,'rust-v0.2.34-mill-public-candidate');
 assert.equal(createHash('sha256').update(current.bytes).digest('hex'),'b17c350a65ba68291a9ff8c44f28382298932abb17a502a35cb9b6a73620482e');
});

test('actual workerd/D1 reopens31/32/33 and preserves34 trigger and corpse fixtures',async t=>{
 const persist=mkdtempSync(join(tmpdir(),'hegemony-mill-public-site-d1-'));
 const options=convertV4MiniflareOptions({name:'mill-public-site-offline',resourcePersistencePath:persist,modules:[{type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:resolve('dist/server',f)}))],compatibilityDate:'2026-10-02',d1Databases:{DB:'mill-public-site-test'}});
 let mf=new Miniflare(options);t.after(()=>mf.dispose());let db=await mf.getD1Database('DB');let store=new RoomStore(db);
 for(const file of readdirSync('drizzle').filter(f=>f.endsWith('.sql')).sort())for(const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 const key=()=>crypto.randomUUID();
 const api=async(path,body,token)=>{const response=await mf.dispatchFetch('http://localhost'+path,{method:body?'POST':'GET',headers:{...(body?{'Content-Type':'application/json'}:{}),...(token?{Authorization:'Bearer '+token}:{})},...(body?{body:JSON.stringify(body)}:{})});return{status:response.status,body:response.status===204?null:await response.json()};};
 const reopen=async()=>{await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);};
 const catalog=await api('/api/catalog');assert.equal(catalog.status,200);assert.equal(catalog.body.cards.length,98);assert.equal(catalog.body.societies.length,8);for(const id of ['XQ36','XQ46'])assert(catalog.body.cards.some(c=>c.id===id));assert(catalog.body.societies.some(c=>c.id==='MSJC03'));
 const created=await api('/api/rooms',{name:'Local blue lobby',mode:'duel',deckId:'watchers',requestId:key()});assert.equal(created.status,200);assert.equal(created.body.view.versions.engine,current.catalog.engineVersion);
 for(const minor of [31,32,33]){
  const old=await load('legacy-v0.2.'+minor),id=String(minor).repeat(12),token=String(minor).repeat(32);const room=JSON.parse(old.abi.newGame(id,'OLD'+minor,'duel','old'+minor,'watchers','1'));
  await store.create({id,invite:'OLD'+minor,state:room.state,nonce:key(),tokenHash:await digest(token),requestHash:await digest(key()),intentHash:'local-old-fixture',response:'{}'});
  const c=await api('/api/rooms/'+id+'/catalog',null,token);assert.equal(c.status,200);assert.equal(c.body.engineVersion,old.catalog.engineVersion);assert.equal(c.body.cards.length,minor===31?93:minor===32?94:96);assert(!c.body.cards.some(c=>c.id==='XQ36'));assert(!c.body.cards.some(c=>c.id==='XQ46'));
  const joined=await api('/api/rooms/join',{inviteCode:'OLD'+minor,name:'old opponent',deckId:'hunters',requestId:key()});assert.equal(joined.status,200);
  const command={commandId:key(),expectedVersion:1,action:{kind:'game',action:{kind:'ready'}}};const ready=await api('/api/rooms/'+id+'/commands',command,token);assert.equal(ready.status,200);assert.equal(ready.body.versions.engine,old.catalog.engineVersion);
  await reopen();assert.deepEqual(await api('/api/rooms/'+id+'/commands',command,token),ready);const after=await api('/api/rooms/'+id+'/state',null,token);assert.equal(after.status,200);assert.deepEqual(withoutClock(after.body),withoutClock(ready.body));
 }
 const fixtures=JSON.parse(readFileSync('web/src/game/millPublicTest.fixture.json','utf8')).fixtures;
 let fixtureCount=0,fullSeatViews=0,commands=0,rejected=0;
 for(const [kind,selected]of [['entry-choice-0-5',['accept']],['entry-choice-0-5',[]],['corpse-aura-0',null],['corpse-forced-hidden',null]]){
  const row=fixtures.find(r=>r.kind===kind),id=String(40+fixtureCount).repeat(12),tokens=[0,1,2,3].map(()=>crypto.randomUUID().replaceAll('-','').repeat(2));
  await store.create({id,invite:'FIXTURE'+fixtureCount,state:row.state,nonce:key(),tokenHash:await digest(tokens[0]),requestHash:await digest(key()),intentHash:'explicit-mill-public-offline',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(row.views[0].version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  for(let seat=0;seat<4;seat++){const r=await api('/api/rooms/'+id+'/state',null,tokens[seat]);assert.equal(r.status,200);assert.deepEqual(withoutClock(r.body),withoutClock(row.views[seat]));fullSeatViews++;}
  await reopen();
  for(let seat=0;seat<4;seat++){const r=await api('/api/rooms/'+id+'/state',null,tokens[seat]);assert.equal(r.status,200);assert.deepEqual(withoutClock(r.body),withoutClock(row.views[seat]));fullSeatViews++;}
  const before=(await store.room(id)).state;const choice=row.views[0].pendingChoice;
  if(choice){const invalid=await api('/api/rooms/'+id+'/commands',{commandId:key(),expectedVersion:row.views[0].version,action:{kind:'game',action:{kind:'choose',choiceId:choice.id,selected:['accept','accept']}}},tokens[0]);assert.equal(invalid.status,400);assert.equal((await store.room(id)).state,before);rejected++;}
  const action=choice?{kind:'choose',choiceId:choice.id,selected}:JSON.parse(row.views[0].legalActions.find(a=>a.kind==='pass').id);
  const cmd={commandId:key(),expectedVersion:row.views[0].version,action:{kind:'game',action}};
  const result=await api('/api/rooms/'+id+'/commands',cmd,tokens[0]);assert.equal(result.status,200,JSON.stringify(result));
  const expected=JSON.parse(current.abi.applyRoom(before,0,JSON.stringify(cmd),String(result.body.serverNowMs)));assert.equal((await store.room(id)).state,expected.state);assert.deepEqual(result.body,expected.view);
  await reopen();assert.deepEqual(await api('/api/rooms/'+id+'/commands',cmd,tokens[0]),result);assert.equal((await db.prepare('SELECT count(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,1);fixtureCount++;commands++;
 }
 t.diagnostic(JSON.stringify({actualLocalWorker:true,actualLocalD1:true,oldVersionsReopened:[31,32,33],new34ExplicitFixtureCases:fixtureCount,new34CommandsCompared:commands,invalidTriggerSelectionsStateUnchanged:rejected,fourSeatViewsCompared:fullSeatViews,duplicateReceiptsAfterReopen:true,explicitOfflineFixtures:true,liveRoomsTouched:false}));
});
