// Whole native transitions and restored private projections; no state rewriting.
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import * as kernel from '../pkg/hegemony_wasm.js';
const instance=kernel.initSync({module:readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',import.meta.url))});
const root=resolve(process.argv[2]);
let commands=0,rejections=0,checkpoints=0,projections=0;
for(const name of readdirSync(root).sort()) {
 if(!name.endsWith('.json')) continue;
 if(process.argv[3] && name!==process.argv[3]) continue;
 const row=JSON.parse(readFileSync(resolve(root,name),'utf8'));
 let state=row.state;
 if(row.command) {
  let result;
  try { result=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),'0')); }
  catch(error) { console.error(JSON.stringify({name,commands,checkpoints,projections,stateBytes:state.length,memoryBytes:instance.memory.buffer.byteLength})); throw error; }
  assert.deepEqual(result,row.expected,name);
  if(row.expected.errorCode) { assert.equal(result.state,state,name); rejections++; }
  state=result.state; commands++;
 } else checkpoints++;
 for(let seat=0;seat<4;seat++) {
  assert.deepEqual(JSON.parse(kernel.view(state,seat)),row.views[seat],`${name} seat${seat}`);
  projections++;
 }
}
if(!process.argv[3]) assert.ok(commands>100 && rejections>0 && checkpoints>100);
console.log(JSON.stringify({commands,rejections,checkpoints,projections}));
