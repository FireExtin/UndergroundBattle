import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import * as frozen from '../../tools/frozen-kernels.mjs';
import {kernel as current,catalog} from '../../rust-game-wasm/tests/current89-kernel.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
import {loadFixture} from '../../tools/xq23-oracle.mjs';
import {loadP2Fixture} from '../../tools/xq23-p2-oracle.mjs';
const saved=Array.from({length:12},(_,i)=>frozen['frozenAbi'+(77+i)]());
const retained=saved.map(s=>{const k=createRequire(import.meta.url)(s.jsPath);k.initSync({module:s.wasm});return k;});
const routed=routeKernels(current,retained);
test('ABI89 adds only complete JC035 and preserves the entire published88 catalog apart from identity',()=>{
 const prior=JSON.parse(retained[11].catalog());
 assert.deepEqual({...catalog,engineVersion:prior.engineVersion,cardPoolVersion:prior.cardPoolVersion,cards:catalog.cards.filter(c=>c.id!=='JC035')},prior);
 assert.equal(catalog.cards.at(-1).id,'JC035');assert.equal(catalog.cards.at(-1).supported,true);
 assert.equal(catalog.cards.length,153);assert.equal(catalog.societies.length,10);
});
for(const[i,k]of retained.entries())test(`thirteen-kernel router keeps exact ABI${77+i} state, joins, polls and commands`,()=>{
 const created=JSON.parse(k.newGame('jc035-retained-'+(77+i),'LOCAL','duel','P0','watchers','18446744073709551615'));
 assert.equal(routed.catalog(created.state),k.catalog());assert.equal(routed.view(created.state,0),k.view(created.state,0));
 assert.throws(()=>current.view(created.state,0));
 const command=JSON.stringify({commandId:'retained-ready',expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}});
 assert.equal(routed.applyRoom(created.state,0,command,'0'),k.applyRoom(created.state,0,command,'0'));
 assert.equal(routed.joinGame(created.state,'P1','hunters'),k.joinGame(created.state,'P1','hunters'));
 assert.equal(routed.pollRoom(created.state,0,'0'),k.pollRoom(created.state,0,'0'));
 assert(created.state.includes('"seed":18446744073709551615'));
});
test('thirteen-kernel router replays the unchanged actual Native88 legal and owner-elimination witnesses',()=>{
 const original=loadFixture(),p2=loadP2Fixture();let commands=0,projections=0;
 for(const s of [...original.scenarios,p2]){
  s.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(s.initial.state,seat)),v);projections++;});
  for(const row of s.steps){
   assert.deepEqual(JSON.parse(routed.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;
   row.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(row.expected.state,seat)),v);projections++;});
  }
 }
 assert.equal(commands,238);assert.equal(projections,992);
});
