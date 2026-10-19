// Additional actual Native88 witness for the two independently reproduced P2s.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {gzipSync,gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {assertSources,identity,sha256} from './xq23-oracle.mjs';
const moduleUrl=import.meta.url,root=fileURLToPath(new URL('../',moduleUrl));
export function reviewSummaryCases(){
 const control=readFileSync(path.join(root,'docs/evidence/xq23-p2-2026-10-19/paid-room-before.json'),'utf8');
 assert.equal(Buffer.byteLength(control),31197);assert.equal(sha256(control),'76a47a29fb9d3b1d772ffdb6087e9180cac93f91fdd0c3af0d989fdc36b48c65');
 const original=JSON.parse(control);
 const invalid=[['target-summary-other-asset-uid','instanceId','i241'],['target-summary-empty-uid','instanceId',''],['target-summary-hidden-kind','kind','hidden'],['target-summary-character-kind','kind','character']].map(([name,field,value])=>{
  const changed=structuredClone(original);changed.game.stack[0].frame.targets[0].public[field]=value;
  assert.equal(changed.game.stack[0].target,'i243');assert.equal(changed.game.stack[0].frame.targets[0].id,'i243');
  const restored=structuredClone(changed);restored.game.stack[0].frame.targets[0].public[field]=original.game.stack[0].frame.targets[0].public[field];assert.deepEqual(restored,original);
  return{name,state:JSON.stringify(changed),changedPath:'/game/stack/0/frame/targets/0/public/'+field};
 });
 return{control,invalid};
}
export function validateP2Native(raw){
 assert.equal(raw.format,'xq23-owner-elimination-native-v1');assertSources(raw.sourceBindings);
 for(const[key,value]of Object.entries(identity))assert.equal(raw.catalog[key],value);
 assert.equal(sha256(raw.initial.state),'dc7bb4725fa63701c785416876c6c200d30ee9a79f6d338e3532dcf6ea280d76');
 assert.equal(sha256(raw.beforeElimination),'09721ef434dff49da5bc5ceda8a1f7e974f13857a8fd6522fe3e8691e9e584a8');
 let current=raw.initial.state,rejected=0;
 for(const row of raw.steps){
  assert.equal(row.state,current);assert.equal(row.serverNowMs,'0');assert.equal(row.views.length,4);assert.deepEqual(row.expected.view,row.views[row.seat]);
  if(row.expected.errorCode){rejected++;assert.equal(row.expected.changed,false);assert.equal(row.expected.state,current);assert.deepEqual(row.expected.journal,[]);}
  else{assert.equal(row.expected.outcome,'accepted');assert.equal(row.expected.changed,true);}
  current=row.expected.state;
 }
 const before=JSON.parse(raw.beforeElimination).game,after=JSON.parse(raw.afterElimination).game,final=JSON.parse(current).game;
 assert.equal(before.players[0].assets.length,11);assert.equal(after.players[0].assets.length,10);
 assert.equal(after.players[2].eliminated,true);assert.equal(after.players[3].eliminated,false);assert.equal(after.status,'playing');
 assert(after.players.flatMap(p=>p.assets).every(c=>c.owner!==2));
 assert(after.players[0].assets.some(c=>c.definition==='XQ23'&&c.owner===3&&c.controller===0));
 assert.equal(final.players[2].hand.length,0);assert(final.players[3].hand.some(c=>c.definition==='XQ23'));
 const last=raw.steps.at(-1).views[0];assert.equal(last.players[2].score,4);assert.equal(last.players[3].score,2);
 assert.equal(raw.summary.commands,raw.steps.length);assert.equal(raw.steps.length,166);assert.equal(rejected,1);
 return{commands:raw.steps.length,rejected,projections:4*(raw.steps.length+1),nativeInputs:139};
}
export function loadP2Fixture(){
 const bundle=JSON.parse(gunzipSync(readFileSync(path.join(root,'rust-game-wasm/tests/xq23P2Native88.fixture.json.gz'))));
 assert.equal(bundle.format,'xq23-p2-native88-raw-v1');assert.equal(Buffer.byteLength(bundle.raw.json),bundle.raw.bytes);assert.equal(sha256(bundle.raw.json),bundle.raw.sha256);
 const raw=JSON.parse(bundle.raw.json);assert.deepEqual(validateP2Native(raw),bundle.counts);return{...bundle,...raw};
}
export function compareP2(f,abi,wasm){
 assert.equal(sha256(wasm),f.wasmSha256);assert.deepEqual(JSON.parse(abi.catalog()),f.catalog);
 f.initial.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(f.initial.state,seat)),view));
 for(const[index,row]of f.steps.entries()){
  assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,'P2 Native command '+index);
  row.views.forEach((view,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),view,'P2 Native view '+index+'/'+seat));
 }
 const boundary=reviewSummaryCases();
 for(let seat=0;seat<4;seat++)assert.equal(JSON.parse(abi.view(boundary.control,seat)).stack[0].targetSummaries[0].instanceId,'i243');
 for(const bad of boundary.invalid){
  const error=e=>{assert.match(String(e),/XQ23/);return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(bad.state,seat),error);
  assert.throws(()=>abi.pollRoom(bad.state,0,'0'),error);
  assert.throws(()=>abi.applyRoom(bad.state,0,JSON.stringify({commandId:'p2-'+bad.name,expectedVersion:9,action:{kind:'pauseRoom'}}),'0'),error);
 }
 return{...f.counts,wasmSha256:f.wasmSha256,nativeRawSha256:f.raw.sha256,exactPreReviewPaidStateRejectedCases:4,opaqueTransitionsJournalsAndFourViewsEqual:true};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(moduleUrl)){
 const[nativeFile,wasmFile,destination]=process.argv.slice(2);if(!destination)throw Error('Usage: xq23-p2-oracle.mjs actualNativeP2.json current.wasm NEW.fixture.json.gz');
 const json=readFileSync(nativeFile,'utf8'),raw=JSON.parse(json),bytes=readFileSync(wasmFile),counts=validateP2Native(raw);
 const bundle={format:'xq23-p2-native88-raw-v1',raw:{json,bytes:Buffer.byteLength(json),sha256:sha256(json)},counts,wasmSha256:sha256(bytes)};
 const{kernel,moduleBytes}=await import('../rust-game-wasm/tests/current88-kernel.mjs');assert.deepEqual(bytes,moduleBytes);
 const result=compareP2({...bundle,...raw},kernel,bytes);writeFileSync(destination,gzipSync(JSON.stringify(bundle),{level:9,mtime:0}),{flag:'wx'});console.log(JSON.stringify({destination,...result}));
}
