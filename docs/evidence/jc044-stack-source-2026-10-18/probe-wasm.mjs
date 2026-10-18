import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {kernel,moduleBytes} from '/workspace/UndergroundBattle/rust-game-wasm/tests/current85-kernel.mjs';
const root='/workspace/jc044-stack-source-review-20261018',phase=process.argv[2];
const raw=JSON.parse(readFileSync(root+'/original-native85.json','utf8')),rows=[];
for(const name of ['entry-deploy-skip','entry-reveal-skip']){
 const state=readFileSync(root+'/'+name+'-bad.room','utf8');let accepted=0,rejected=0;
 const methods=[...Array.from({length:4},(_,seat)=>()=>kernel.view(state,seat)),()=>kernel.applyRoom(state,0,JSON.stringify({commandId:'p2-'+name,expectedVersion:9,action:{kind:'pauseRoom'}}),'0'),()=>kernel.pollRoom(state,0,'0')];
 for(const call of methods){try{const result=JSON.parse(call());assert.equal(phase,'before','Corrupted Room unexpectedly accepted');if('errorCode'in result)assert.equal(result.errorCode,null);accepted++;}catch(error){if(phase==='before')throw error;assert.equal(String(error),'JC044不能挂接其他牌的堆栈程序');rejected++;}}
 const scenario=raw.scenarios.find(s=>s.name===name);
 for(const row of scenario.steps){assert.deepEqual(JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);row.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(kernel.view(row.expected.state,seat)),v));}
 rows.push({name,accepted,rejected,legalCommandsEqualOriginal:scenario.steps.length,legalViewsEqualOriginal:scenario.steps.length*4});
}
console.log(JSON.stringify({phase,wasmSha256:createHash('sha256').update(moduleBytes).digest('hex'),opaqueRoomNeverParsedOrReencodedInJs:true,rows},null,2));
