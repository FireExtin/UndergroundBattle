import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { frozenAbi77, frozenAbi78, frozenAbi79, frozenAbi80 } from '../../tools/frozen-kernels.mjs';
import { kernel as current } from '../../rust-game-wasm/tests/current81-kernel.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
const saved=[frozenAbi77(),frozenAbi78(),frozenAbi79(),frozenAbi80()];
const old=saved.map(s=>{const abi=createRequire(import.meta.url)(s.jsPath);abi.initSync({module:s.wasm});return abi;});
const routed=routeKernels(current,old);
for(const [index,abi]of old.entries())test(`five-kernel router preserves exact ABI${77+index} state, views and transitions`,()=>{
  const room=JSON.parse(abi.newGame(`retained-${77+index}`,'LOCAL','duel','P0','watchers','18446744073709551615'));
  assert.equal(routed.catalog(room.state),abi.catalog());assert.equal(routed.view(room.state,0),abi.view(room.state,0));
  assert.throws(()=>current.view(room.state,0));
  const command=JSON.stringify({commandId:'old-ready',expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}});
  assert.equal(routed.applyRoom(room.state,0,command,'0'),abi.applyRoom(room.state,0,command,'0'));
  assert.equal(routed.joinGame(room.state,'P1','hunters'),abi.joinGame(room.state,'P1','hunters'));
  assert.equal(routed.pollRoom(room.state,0,'0'),abi.pollRoom(room.state,0,'0'));
  assert(room.state.includes('"seed":18446744073709551615'));
});
