// Local workerd + real HTTP/service/store + persisted D1. Not browser evidence.
import test,{after} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,mkdtempSync,rmSync,existsSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {Miniflare,convertV4MiniflareOptions} from 'miniflare';
import {loadCurrentFixture} from '../../tools/current-native-source.mjs';
import {RoomStore} from '../src/store.mjs';
import {digest} from '../src/service.mjs';
import {assertPublicView,assertSessionJournal} from './assert-session-journal.mjs';
const project=fileURLToPath(new URL('../',import.meta.url));
const root=existsSync(join(project,'web/package.json'))?project:fileURLToPath(new URL('../../',import.meta.url));
const ownOutput=process.env.SENTINEL_WORKER_DIR?null:mkdtempSync(join(tmpdir(),'sentinel-current-'));
const worker=process.env.SENTINEL_WORKER_DIR?resolve(process.env.SENTINEL_WORKER_DIR):join(ownOutput,'worker');
if(ownOutput){
 execFileSync(process.execPath,[join(root,'tools/build-current-local-worker.mjs'),join(project,'node_modules'),join(root,'rust-game-wasm/pkg'),worker]);
 after(()=>rmSync(ownOutput,{recursive:true,force:true}));
}
const scenarios=process.env.SENTINEL_WORKER_WITNESSES
 ?process.env.SENTINEL_WORKER_WITNESSES.split(':').flatMap(p=>JSON.parse(readFileSync(p,'utf8')))
 :loadCurrentFixture('rust-game-wasm/tests/current112/sentinel-worker.json.gz');
async function harness(t){
 const persist=mkdtempSync(join(tmpdir(),'sentinel-worker-'));
 const options=convertV4MiniflareOptions({name:'sentinel-local',resourcePersistencePath:persist,modulesRoot:worker,modules:[{type:'ESModule',path:join(worker,'worker.js')},{type:'CompiledWasm',path:join(worker,'candidate.wasm')}],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'sentinel-local'}});
 let mf=new Miniflare(options),db=await mf.getD1Database('DB');
 t.after(async()=>{await mf.dispose();rmSync(persist,{recursive:true,force:true});});
 for(const file of readdirSync(join(project,'drizzle')).filter(n=>n.endsWith('.sql'))){for(const sql of readFileSync(join(project,'drizzle',file),'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();}
 return {db:()=>db,store:()=>new RoomStore(db),reopen:async()=>{await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');},api:async(path,token,body,now='0')=>{
  const response=await mf.dispatchFetch('http://localhost/api/'+path,{method:body?'POST':'GET',headers:{'X-Local-Clock':now,...(token?{Authorization:'Bearer '+token}:{}),...(body?{'Content-Type':'application/json'}:{})},...(body?{body:JSON.stringify(body)}:{})});
  return {status:response.status,body:await response.json()};
 }};
}
for(const s of scenarios)test('isolated Worker/D1 '+s.name,async t=>{
 const h=await harness(t),initial=s.initialViews[0],id=initial.roomId,tokens=[1,2,3,4].map(n=>String(n).repeat(64));
 await h.store().create({id,invite:initial.inviteCode,state:s.initial,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'sentinel-local-fixture',response:'{}'});
 await h.db().prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
 for(let seat=1;seat<4;seat++)await h.db().prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
 let views=0,reloads=0,duplicates=0;
 for(let seat=0;seat<4;seat++){const x=await h.api(`rooms/${id}/state`,tokens[seat]);assert.equal(x.status,200);assertPublicView(x.body,s.initialViews[seat]);views++;}
 for(const row of s.steps){
  assert.equal((await h.store().room(id)).state,row.before);
  const x=await h.api(`rooms/${id}/commands`,tokens[row.seat],row.command,row.now);assert.equal(x.status,200,JSON.stringify(x.body));assertPublicView(x.body,row.expected.view);
  assert.equal((await h.store().room(id)).state,row.expected.state);
  const journal=await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,row.expected.version).first();assertSessionJournal(journal.entry,row.expected.journal);
  const duplicate=await h.api(`rooms/${id}/commands`,tokens[row.seat],row.command,row.now);assert.equal(duplicate.status,200);assert.deepEqual(duplicate.body,x.body);duplicates++;
  await h.reopen();reloads++;assert.equal((await h.store().room(id)).state,row.expected.state);
  const replay=await h.api(`rooms/${id}/commands`,tokens[row.seat],row.command,row.now);assert.equal(replay.status,200);assert.deepEqual(replay.body,x.body);duplicates++;
  for(let seat=0;seat<4;seat++){const view=await h.api(`rooms/${id}/state`,tokens[seat],undefined,row.now);assert.equal(view.status,200);assertPublicView(view.body,row.views[seat]);views++;}
 }
 assert.equal((await h.db().prepare('SELECT count(*) AS n FROM commands WHERE room_id=?').bind(id).first()).n,s.steps.length);
 t.diagnostic(JSON.stringify({scenario:s.name,commands:s.steps.length,views,reloads,duplicates,exactOpaqueD1AndNativeJournals:true}));
});
test('isolated Worker actual Sentinel deck selection and lobby commands',async t=>{
 const h=await harness(t),catalog=(await h.api('catalog')).body;
 assert(catalog.societies.some(s=>s.id==='MSJC14'));assert.equal(catalog.cards.find(c=>c.id==='WM044').supported,false);
 const deckDraft={id:'sentinel-local',name:'Sentinel local',description:'normal legal selection',societyId:'MSJC14',cards:[{cardId:'JC125',count:50}],rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:'2026-10-23'};
 const create={requestId:crypto.randomUUID(),name:'P0',mode:'teams',deckDraft};const entry=await h.api('rooms',null,create);assert.equal(entry.status,200,JSON.stringify(entry.body));
 assert.deepEqual((await h.api('rooms',null,create)).body,entry.body);
 const id=entry.body.roomId,tokens=[entry.body.token];
 for(let seat=1;seat<4;seat++){const join=await h.api('rooms/join',null,{requestId:crypto.randomUUID(),name:'P'+seat,inviteCode:entry.body.inviteCode,deckDraft});assert.equal(join.status,200,JSON.stringify(join.body));tokens.push(join.body.token);}
 let accepted=0;
 async function command(seat,action){const view=await h.api(`rooms/${id}/state`,tokens[seat]);const c={commandId:crypto.randomUUID(),expectedVersion:view.body.version,action:{kind:'game',action}};const x=await h.api(`rooms/${id}/commands`,tokens[seat],c);assert.equal(x.status,200,JSON.stringify(x.body));assert.deepEqual((await h.api(`rooms/${id}/commands`,tokens[seat],c)).body,x.body);accepted++;return x.body;}
 for(let seat=0;seat<4;seat++)await command(seat,{kind:'ready'});await command(0,{kind:'start'});
 for(let n=0;n<16;n++){let chosen=false;for(let seat=0;seat<4;seat++){const view=(await h.api(`rooms/${id}/state`,tokens[seat])).body;if(view.pendingChoice){await command(seat,{kind:'choose',choiceId:view.pendingChoice.id,selected:[]});chosen=true;break;}}if(!chosen)break;}
 const before=await h.store().room(id);await h.reopen();assert.deepEqual(await h.store().room(id),before);
 for(let seat=0;seat<4;seat++){const v=await h.api(`rooms/${id}/state`,tokens[seat]);assert.equal(v.status,200);assert.equal(v.body.status,'playing');assert.equal(v.body.societyZones[seat].card.cardId,'MSJC14');assert(v.body.legalActions);}
 t.diagnostic(JSON.stringify({normalCreate:1,normalJoins:3,acceptedCommands:accepted,duplicateReceipts:accepted+1,normalReload:1,selectedSociety:'MSJC14'}));
});
