import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {frozenAbi77,frozenAbi78,frozenAbi79,frozenAbi80,frozenAbi81,frozenAbi82,frozenAbi83,frozenAbi84,frozenAbi85} from '../../tools/frozen-kernels.mjs';
import {kernel as current,catalog} from '../../rust-game-wasm/tests/current86-kernel.mjs';
import {loadFixture} from '../../tools/jc044-oracle.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
const saved=[frozenAbi77(),frozenAbi78(),frozenAbi79(),frozenAbi80(),frozenAbi81(),frozenAbi82(),frozenAbi83(),frozenAbi84(),frozenAbi85()];
const retained=saved.map(s=>{const k=createRequire(import.meta.url)(s.jsPath);k.initSync({module:s.wasm});return k;});
const routed=routeKernels(current,retained);
test('ABI86 adds only complete JC066 and keeps all 149 previous definitions, societies and presets',()=>{
  const published=JSON.parse(retained[8].catalog());
  assert.deepEqual({...catalog,engineVersion:published.engineVersion,cardPoolVersion:published.cardPoolVersion,cards:catalog.cards.filter(c=>c.id!=='JC066')},published);
  assert.equal(catalog.cards.at(-1).id,'JC066');assert.equal(catalog.cards.at(-1).supported,true);
});
for(const [i,k]of retained.entries())test(`ten-kernel router retains exact ABI${77+i} views, joins, commands and polls`,()=>{
  const room=JSON.parse(k.newGame(`retained-${77+i}`,'LOCAL','duel','P0','watchers','18446744073709551615'));
  assert.equal(routed.catalog(room.state),k.catalog());assert.equal(routed.view(room.state,0),k.view(room.state,0));
  assert.throws(()=>current.view(room.state,0));
  const cmd=JSON.stringify({commandId:'old-ready',expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}});
  assert.equal(routed.applyRoom(room.state,0,cmd,'0'),k.applyRoom(room.state,0,cmd,'0'));
  assert.equal(routed.joinGame(room.state,'P1','hunters'),k.joinGame(room.state,'P1','hunters'));
  assert.equal(routed.pollRoom(room.state,0,'0'),k.pollRoom(room.state,0,'0'));
  assert(room.state.includes('"seed":18446744073709551615'));
});
test('ten-kernel router preserves original Native85 opaque states, journals and all four views',()=>{
  const f=loadFixture();let commands=0,views=0;
  for(const s of f.scenarios){
    s.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(s.initial.state,seat)),v);views++;});
    for(const row of s.steps){assert.deepEqual(JSON.parse(routed.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;
      row.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(row.expected.state,seat)),v);views++;});}
  }
  assert.deepEqual({commands,views},{commands:118,views:524});
});
