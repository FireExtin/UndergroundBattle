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
test('actual31 keeps all 32 prior kernels and local D1 lobbies unchanged across Worker reopen',async t=>{
 const base=resolve('rust-game-wasm');const names=['pkg',...readdirSync(base).filter(n=>/^legacy-v0\.2\.\d+(?:-resource-policy)?$/.test(n)).sort((a,b)=>Number(a.match(/legacy-v0\.2\.(\d+)/)[1])-Number(b.match(/legacy-v0\.2\.(\d+)/)[1])||a.localeCompare(b))];assert.equal(names.length,33);
 const kernels=[];const versions=[];
 for(const name of names){const p=join(base,name);const k=await import(pathToFileURL(join(p,'hegemony_wasm.js')));const bytes=readFileSync(join(p,'hegemony_wasm_bg.wasm'));k.initSync({module:bytes});const c=JSON.parse(k.catalog());kernels.push(k);versions.push({name,engine:c.engineVersion,pool:c.cardPoolVersion,sha256:createHash('sha256').update(bytes).digest('hex')});}
 assert.equal(versions[0].sha256,'d5c544fde889abf066cf794b24b62106d439aa453e9e6a210e88217e3e872bbe');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.29').sha256,'78b6f630ecef5a35b50cd07852374566cd1063d55af15a3802e54df7cc27b19a');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.30').sha256,'06dc444774f8681d0590794ce760fb0c6fd1ca9fc789532eb8cbda18b000a6a5');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.28').sha256,'a8a928e12e239161c082671a6008b64775c7b8e9e0181e7d765db23af8bb32e8');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.27').sha256,'04b5ed811e39922ef4d0af5d8cb9539f16573f37c88d595cdc50a61856eddcc4');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.25-resource-policy').sha256,'7e77929d15020872fd7754547725a535f916baa943f2e7aa0486bfee097db105');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.26').sha256,'c187a493d3e32f82c8609cb203e12ffb1661fbe348260f6e010254bb9af395af');
 assert.equal(versions.find(v=>v.name==='legacy-v0.2.26-resource-policy').sha256,'79b0a9fa5e9f2acc015230ec3cd42e7f0fae41d8e8f720be7e6c81db52cd824e');
 const current=JSON.parse(kernels[0].catalog());assert.equal(current.cards.length,93);assert.deepEqual(current.societies.map(s=>s.id),['MSJC09','MSJC01','MSJC07','MSJC06','MSJC08','MSJC11','MSJC02']);
 const routed=routeKernels(kernels[0],kernels.slice(1));
 for(let i=0;i<names.length;i++){const k=kernels[i];let v=JSON.parse(k.newGame((i+1).toString(16).padStart(24,'0'),'ROUTE27'+i,'teams','Local0','watchers','18446744073709551615'));for(let seat=1;seat<4;seat++)v=JSON.parse(k.joinGame(v.state,'Local'+seat,'watchers'));assert.deepEqual(JSON.parse(routed.catalog(v.state)),JSON.parse(k.catalog()));for(let seat=0;seat<4;seat++)assert.deepEqual(JSON.parse(routed.view(v.state,seat)),JSON.parse(k.view(v.state,seat)));}
 const persist=mkdtempSync(join(tmpdir(),'hegemony-25-compatible-'));
 const options=convertV4MiniflareOptions({name:'hegemony-25-compatible',resourcePersistencePath:persist,modules:[{type:'ESModule',path:resolve('dist/server/index.js')},...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve('dist/server',n)}))],compatibilityDate:'2026-10-02',d1Databases:{DB:'hegemony-25-compatible-db'}});
 let mf=new Miniflare(options);t.after(()=>mf.dispose());let db=await mf.getD1Database('DB');
 for(const f of readdirSync('drizzle').filter(n=>n.endsWith('.sql')))for(const sql of readFileSync(join('drizzle',f),'utf8').split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 let store=new RoomStore(db);const saved=[];
 const get=async(id,token,suffix)=>{const response=await mf.dispatchFetch('http://local/api/rooms/'+id+'/'+suffix,{headers:{Authorization:'Bearer '+token}});assert.equal(response.status,200);return response.json();};
 const catalogResponse=await mf.dispatchFetch('http://local/api/catalog');assert.equal(catalogResponse.status,200);const catalog=await catalogResponse.json();assert.equal(catalog.engineVersion,current.engineVersion);assert.equal(catalog.cardPoolVersion,current.cardPoolVersion);
 for(let minor=1;minor<names.length;minor++){
  const k=kernels[minor],id=minor.toString(16).padStart(24,'0'),token=minor.toString(16).padStart(64,'0');const initial=JSON.parse(k.newGame(id,'COMPAT25V'+minor,'duel','Local'+minor,'watchers','18446744073709551615'));
  await store.create({id,invite:'COMPAT25V'+minor,state:initial.state,nonce:'nonce'+minor,tokenHash:await digest(token),requestHash:await digest('request'+minor),intentHash:'local-compatibility-fixture',response:'{}'});
  const before=(await store.room(id)).state;const view=await get(id,token,'state');assert.deepEqual(stable(view),stable(initial.view));assert.deepEqual(await get(id,token,'catalog'),{...JSON.parse(k.catalog()),transport:'polling',entryIdempotency:true});assert.equal((await store.room(id)).state,before);
  saved.push({id,token,state:before,view:stable(view)});
 }
 await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');store=new RoomStore(db);
 for(const s of saved){assert.deepEqual(stable(await get(s.id,s.token,'state')),s.view);assert.equal((await store.room(s.id)).state,s.state);}
 t.diagnostic(JSON.stringify({currentEngine:current.engineVersion,kernelCount:33,oldLocalD1Lobbies:32,allRestoredExact:true,versions,publicRoomAccessed:false,msjc07Included:true,msjc06Included:true,msjc08Included:true,ruleOrOldRoomIdentityMigration:false}));
});
