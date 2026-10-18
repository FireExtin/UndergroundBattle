import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as abi from '/workspace/UndergroundBattle/rust-game-wasm/pkg/hegemony_wasm.js';
const base='/workspace/jc057-p2-20261018/';
const wasm=readFileSync('/workspace/UndergroundBattle/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
abi.initSync({module:wasm});
const cases=[];
for(const [name,file,seat,selected,valid]of [
 ['original-owner-skip','checkpoint-original.state.json',0,[],true],
 ['original-owner-target','checkpoint-original.state.json',0,['p1'],true],
 ['original-other-skip','checkpoint-original.state.json',1,[],true],
 ['bad-seat-only-skip','bad-seat-only.state.json',1,[],false],
 ['bad-seat-and-player-id-target','bad-seat-and-player-id.state.json',1,['p1'],false],
]) {
 const state=readFileSync(base+file,'utf8');
 const command={commandId:'before-fix-'+name,expectedVersion:13,action:{kind:'game',action:{kind:'choose',choiceId:'i249',selected}}};
 if(valid) {
  abi.view(state,seat);
  const result=JSON.parse(abi.applyRoom(state,seat,JSON.stringify(command),'0'));
  assert.equal(result.outcome,seat===0?'accepted':'rejected');
  assert.equal(result.version,seat===0?14:13);
  cases.push({name,seat,selected,restored:true,outcome:result.outcome,errorCode:result.errorCode,changed:result.changed,version:result.version,nextState:result.state});
 } else {
  let viewError,commandError;
  try{abi.view(state,seat);}catch(error){viewError=String(error);}
  try{abi.applyRoom(state,seat,JSON.stringify(command),'0');}catch(error){commandError=String(error);}
  assert.equal(viewError,'JC057触发待选必须属于冻结行动者');assert.equal(commandError,viewError);
  cases.push({name,seat,selected,restored:false,viewError,commandError});
 }
}
const result={wasmSha256:createHash('sha256').update(wasm).digest('hex'),scope:'Exact same losslessly preserved original and malformed persisted archives as pre-fix reproduction',cases};
writeFileSync(base+'repro-after.json',JSON.stringify(result,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({...result,cases:cases.map(({nextState,...row})=>row)}));
