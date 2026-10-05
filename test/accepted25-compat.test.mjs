// Local generated lobbies and local D1 only: no public rooms or browser actions.
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
const stable=v=>{const copy=structuredClone(v);delete copy.serverNowMs;return copy;};
test('actual25 keeps all 25 prior kernels and local D1 lobbies unchanged across Worker reopen',async t=>{
 const base=resolve('rust-game-wasm');const names=['pkg',...readdirSync(base).filter(n=>/^legacy-v0\.2\.\d+$/.test(n)).sort((a,b)=>Number(a.split('.').at(-1))-Number(b.split('.').at(-1)))];assert.equal(names.length,26);
 const kernels=[];const versions=[];
 for(const name of names){const p=join(base,name);const k=await import(pathToFileURL(join(p,'hegemony_wasm.js')));const bytes=readFileSync(join(p,'hegemony_wasm_bg.wasm'));k.initSync({module:bytes});const c=JSON.parse(k.catalog());kernels.push(k);versions.push({name,engine:c.engineVersion,pool:c.cardPoolVersion,sha256:createHash('sha256').update(bytes).digest('hex')});}
 assert.equal(versions[0].sha256,'7e77929d15020872fd7754547725a535f916baa943f2e7aa0486bfee097db105');
 const current=JSON.parse(kernels[0].catalog());assert.equal(current.cards.length,87);assert.deepEqual(current.societies.map(s=>s.id),['MSJC09','MSJC01']);
 const routed=routeKernels(kernels[0],kernels.slice(1));
 for(let i=0;i<26;i++){const k=kernels[i];const v=JSON.parse(k.newGame((i+1).toString(16).padStart(24,'0'),'ROUTE25'+i,'duel','Local','watchers','18446744073709551615'));assert.deepEqual(JSON.parse(routed.catalog(v.state)),JSON.parse(k.catalog()));assert.deepEqual(JSON.parse(routed.view(v.state,0)),JSON.parse(k.view(v.state,0)));}
 const persist=mkdtempSync(join(tmpdir(),'hegemony-25-compatible-'));
 const options=convertV4MiniflareOptions({name:'hegemony-25-compatible',resourcePersistencePath:persist,modules:[{type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))],compatibilityDate:'2026-10-02',d1Databases:{DB:'hegemony-25-compatible-db'}});
 let mf=new Miniflare(options);t.after(()=>mf.dispose());let db=await mf.getD1Database('DB');
 for(const f of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync(join('drizzle',f),'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 let store=new RoomStore(db);const saved=[];
 const get=async(id,token,suffix)=>{const response=await mf.dispatchFetch('http://local/api/rooms/'+id+'/'+suffix,{headers:{Authorization:'Bearer '+token}});assert.equal(response.status,200);return response.json();};
 const catalogResponse=await mf.dispatchFetch('http://local/api/catalog');assert.equal(catalogResponse.status,200);const catalog=await catalogResponse.json();assert.equal(catalog.engineVersion,current.engineVersion);assert.equal(catalog.cardPoolVersion,current.cardPoolVersion);
 for(let minor=1;minor<=25;minor++){
  const k=kernels[minor],id=minor.toString(16).padStart(24,'0'),token=minor.toString(16).padStart(64,'0');const initial=JSON.parse(k.newGame(id,'COMPAT25V'+minor,'duel','Local'+minor,'watchers','18446744073709551615'));
  await store.create({id,invite:'COMPAT25V'+minor,state:initial.state,nonce:'nonce'+minor,tokenHash:await digest(token),requestHash:await digest('request'+minor),intentHash:'local-compatibility-fixture',response:'{}'});
  const before=(await store.room(id)).state;const view=await get(id,token,'state');assert.deepEqual(stable(view),stable(initial.view));assert.deepEqual(await get(id,token,'catalog'),{...JSON.parse(k.catalog()),transport:'polling',entryIdempotency:true});assert.equal((await store.room(id)).state,before);
  saved.push({id,token,state:before,view:stable(view)});
 }
 await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);
 for(const s of saved){assert.deepEqual(stable(await get(s.id,s.token,'state')),s.view);assert.equal((await store.room(s.id)).state,s.state);}
 t.diagnostic(JSON.stringify({currentEngine:current.engineVersion,kernelCount:26,oldLocalD1Lobbies:25,allRestoredExact:true,versions,publicRoomAccessed:false,msjc07Included:false}));
});
