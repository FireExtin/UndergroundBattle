import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { frozenAbi77, frozenAbi78, frozenAbi79, frozenAbi80, frozenAbi81 } from '../../tools/frozen-kernels.mjs';
import { kernel as current, catalog } from '../../rust-game-wasm/tests/current82-kernel.mjs';
import { loadFixture as load81 } from '../../tools/jc054-oracle.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
const saved=[frozenAbi77(),frozenAbi78(),frozenAbi79(),frozenAbi80(),frozenAbi81()];
const retained=saved.map(s=>{const abi=createRequire(import.meta.url)(s.jsPath);abi.initSync({module:s.wasm});return abi;});
const routed=routeKernels(current,retained);
test('ABI82 changes only engine identity in the entire published v63 catalog',()=>{
  const published=JSON.parse(retained[4].catalog());
  assert.deepEqual({...catalog,engineVersion:published.engineVersion},published);
  assert.equal(catalog.engineVersion,'rust-v0.2.82-response-source-preview-candidate');
});
for(const [index,abi]of retained.entries())test(`six-kernel router preserves original ABI${77+index} state, views, joins and transitions`,()=>{
  const room=JSON.parse(abi.newGame(`retained-${77+index}`,'LOCAL','duel','P0','watchers','18446744073709551615'));
  assert.equal(routed.catalog(room.state),abi.catalog());assert.equal(routed.view(room.state,0),abi.view(room.state,0));
  assert.throws(()=>current.view(room.state,0));
  const command=JSON.stringify({commandId:'old-ready',expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}});
  assert.equal(routed.applyRoom(room.state,0,command,'0'),abi.applyRoom(room.state,0,command,'0'));
  assert.equal(routed.joinGame(room.state,'P1','hunters'),abi.joinGame(room.state,'P1','hunters'));
  assert.equal(routed.pollRoom(room.state,0,'0'),abi.pollRoom(room.state,0,'0'));
  assert(room.state.includes('"seed":18446744073709551615'));
});
test('all original Native81 pending layouts retain pre-preview wire shape and reducer results',()=>{
  const fixture=load81();let views=0,commands=0;
  for(const scenario of fixture.scenarios) {
    for(const [seat,view]of scenario.initial.views.entries()){
      assert.deepEqual(JSON.parse(routed.view(scenario.initial.state,seat)),view);views++;
      if(view.responseWindow)assert.equal('previewSources'in view.responseWindow,false);
    }
    for(const row of scenario.steps){
      assert.deepEqual(JSON.parse(routed.applyRoom(row.state,row.seat,JSON.stringify(row.command),'0')),row.expected);commands++;
      for(const [seat,view]of row.views.entries()){assert.deepEqual(JSON.parse(routed.view(row.expected.state,seat)),view);views++;}
    }
  }
  assert.equal(commands,214);assert.equal(views,888);
});
