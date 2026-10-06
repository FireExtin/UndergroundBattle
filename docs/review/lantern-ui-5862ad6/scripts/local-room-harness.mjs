import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import {readFileSync,readdirSync,writeFileSync,mkdirSync,cpSync} from 'node:fs';
import {resolve,join,extname} from 'node:path';
const root='/workspace/lantern-ui-integration-game';
const evidence='/workspace/lantern-ui-e2e-evidence';
const require=createRequire(root+'/sites/package.json');
const {Miniflare,convertV4MiniflareOptions}=require('miniflare');

// Local test-only entropy control: production create still chooses the seed and
// creates all initial rows through its ordinary API. Only its 8-byte seed input
// is fixed. IDs, tokens and nonces keep native entropy. No room DB injection.
export async function startHarness(run='run-1',seed=9){
 const persist=join(evidence,run,'sqlite');mkdirSync(persist,{recursive:true});
 const wrapper=join(root,'sites/dist/server/lantern-e2e-test-entry.mjs');
 const worker=resolve(root,'sites/dist/server/index.js');
 writeFileSync(wrapper,`import worker from './index.js';
const native=crypto.getRandomValues.bind(crypto);
Object.defineProperty(crypto,'getRandomValues',{value(buffer){
 if(buffer.byteLength===8 && new Error().stack.includes('RoomService.create')){let n=BigInt(${JSON.stringify(String(seed))});for(let i=7;i>=0;i--){buffer[i]=Number(n&255n);n>>=8n;}return buffer;}
 return native(buffer);
}});
export default worker;\n`);
 const options=convertV4MiniflareOptions({name:'lantern-real-ui-'+run,resourcePersistencePath:persist,modules:[
  {type:'ESModule',path:wrapper},{type:'ESModule',path:worker},
  ...readdirSync(root+'/sites/dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:resolve(root,'sites/dist/server',n)}))
 ],compatibilityDate:'2026-10-02',cf:false,d1Databases:{DB:'lantern-ui-'+run+'-db'}});
 console.log('harness starting',run);
 let mf=new Miniflare(options),db=await mf.getD1Database('DB');
 console.log('harness D1 bound',run);
 const exists=await db.prepare("SELECT name FROM sqlite_master WHERE name='rooms'").first();
 if(!exists){
  for(const file of readdirSync(root+'/sites/drizzle').filter(n=>n.endsWith('.sql')).sort())
   for(const sql of readFileSync(root+'/sites/drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(x=>x.trim()))await db.prepare(sql).run();
 }
 const requests=[];
 const server=createServer(async(req,res)=>{
  try{
   if(req.url.startsWith('/api/')){
    const chunks=[];for await(const c of req)chunks.push(c);
    const body=Buffer.concat(chunks).toString('utf8');
    const response=await mf.dispatchFetch('http://localhost'+req.url,{method:req.method,headers:req.headers,...(body?{body}:{})});
    const bytes=Buffer.from(await response.arrayBuffer());
    if(req.method==='POST')requests.push({at:Date.now(),url:req.url,method:req.method,body:JSON.parse(body),status:response.status,response:JSON.parse(bytes.toString())});
    res.writeHead(response.status,Object.fromEntries(response.headers));res.end(bytes);return;
   }
   const route=decodeURIComponent(req.url.split('?')[0]);
   const file=resolve(root,'web/dist','.'+route);
   let bytes,type;
   try{if(!file.startsWith(root+'/web/dist/'))throw Error('path');bytes=readFileSync(file);type=extname(file);}catch{bytes=readFileSync(root+'/web/dist/index.html');type='.html';}
   const types={'.html':'text/html; charset=utf-8','.js':'text/javascript','.css':'text/css','.jpg':'image/jpeg','.png':'image/png','.svg':'image/svg+xml','.json':'application/json'};
   res.writeHead(200,{'Content-Type':types[type]||'application/octet-stream','Cache-Control':'no-store'});res.end(bytes);
  }catch(error){res.writeHead(503,{'Content-Type':'application/json'});res.end(JSON.stringify({error:'local_harness_unavailable',message:String(error)}));}
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 console.log('harness HTTP listening',server.address().port);
 const url='http://127.0.0.1:'+server.address().port;
 async function snapshot(roomId){
  const room=await db.prepare('SELECT * FROM rooms WHERE id=?').bind(roomId).first();
  const commands=(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(roomId).all()).results;
  const journal=(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(roomId).all()).results;
  const seats=(await db.prepare('SELECT * FROM seats WHERE room_id=? ORDER BY seat').bind(roomId).all()).results;
  const entries=(await db.prepare('SELECT * FROM entry_receipts WHERE room_id=? ORDER BY request_hash').bind(roomId).all()).results;
  return {room,commands,journal,seats,entries};
 }
 async function reopen(roomId){
  const before=await snapshot(roomId);await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');
  const after=await snapshot(roomId);return {before,after};
 }
 async function close(){await new Promise(done=>server.close(done));await mf.dispose();writeFileSync(join(evidence,run,'requests.json'),JSON.stringify(requests,null,2));}
 return {url,requests,snapshot,reopen,close,persist,run,seed};
}
