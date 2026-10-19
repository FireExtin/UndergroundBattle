import assert from 'node:assert/strict';
import {readFileSync,writeFileSync}from'node:fs';
import {createHash}from'node:crypto';
import{kernel as abi,moduleBytes}from'/workspace/UndergroundBattle/rust-game-wasm/tests/current88-kernel.mjs';
const f=JSON.parse(readFileSync(process.argv[2],'utf8'));
f.initial.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(f.initial.state,seat)),v));
for(const[index,row]of f.steps.entries()){
 assert.deepEqual(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected,'native command '+index);
 row.views.forEach((v,seat)=>assert.deepEqual(JSON.parse(abi.view(row.expected.state,seat)),v,'resulting view '+index+' seat '+seat));
}
const result={wasmSha256:createHash('sha256').update(moduleBytes).digest('hex'),scope:'Real JC129 and XQ23 declarations/responses, ordinary passes to next-turn empty draw; complete actual Native opaque transitions/journals and four-seat views equal',commands:f.steps.length,views:4*(f.steps.length+1),summary:f.summary};
if(process.argv[3])writeFileSync(process.argv[3],JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
