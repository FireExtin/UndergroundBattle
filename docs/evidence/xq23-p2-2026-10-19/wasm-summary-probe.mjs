import assert from 'node:assert/strict';
import {readFileSync,writeFileSync}from'node:fs';
import {createHash}from'node:crypto';
import{kernel as abi,moduleBytes}from'/workspace/UndergroundBattle/rust-game-wasm/tests/current88-kernel.mjs';
const input=JSON.parse(readFileSync(process.argv[2],'utf8')),results=[];
for(const c of input.cases){
 const state=readFileSync(c.roomFile,'utf8'),result={name:c.name,actualTarget:c.actualTarget,mutatedPublicTarget:c.frozenPublicTarget};
 try{
  const views=Array.from({length:4},(_,seat)=>JSON.parse(abi.view(state,seat)).stack[0].targetSummaries);
  const polls=Array.from({length:4},(_,seat)=>{const r=JSON.parse(abi.pollRoom(state,seat,'0'));return{changed:r.changed,projectedTargets:r.view.stack[0].targetSummaries};});
  let current=state;const transitions=[];
  for(const row of input.remainingSteps){const next=JSON.parse(abi.applyRoom(current,row.seat,JSON.stringify(row.command),'0'));transitions.push({changed:next.changed,errorCode:next.errorCode??null,version:next.version});current=next.state;}
  const final=JSON.parse(current);
  Object.assign(result,{accepted:true,projectedTargetsFourSeats:views,polls,remainingTransitions:transitions,finalStateMatchesOriginalNative:current===input.originalFinalState,originalActualTargetStillInAssets:final.game.players.some(p=>p.assets.some(c=>c.id==='i243')),forgedSummaryTargetStillInAssets:final.game.players.some(p=>p.assets.some(c=>c.id==='i241')),handInstances:final.game.players[0].hand.map(c=>({id:c.id,definition:c.definition}))});
 }catch(error){Object.assign(result,{accepted:false,error:String(error)});}
 results.push(result);
}
const out={wasmSha256:createHash('sha256').update(moduleBytes).digest('hex'),results};
if(process.argv[3])writeFileSync(process.argv[3],JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify(out));
