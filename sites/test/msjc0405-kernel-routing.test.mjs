import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {frozenAbi77,frozenAbi78,frozenAbi79,frozenAbi80,frozenAbi81,frozenAbi82} from '../../tools/frozen-kernels.mjs';
import {kernel as current,catalog} from '../../rust-game-wasm/tests/current83-kernel.mjs';
import {loadFixture as load82} from '../../tools/response-preview-oracle.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
const saved=[frozenAbi77(),frozenAbi78(),frozenAbi79(),frozenAbi80(),frozenAbi81(),frozenAbi82()];
const retained=saved.map(s=>{const abi=createRequire(import.meta.url)(s.jsPath);abi.initSync({module:s.wasm});return abi;});
const routed=routeKernels(current,retained);
test('ABI83 admits only complete MSJC04/MSJC05 and preserves every previous catalog definition and preset',()=>{
  const published=JSON.parse(retained[5].catalog());
  assert.deepEqual({...catalog,engineVersion:published.engineVersion,cardPoolVersion:published.cardPoolVersion,
    societies:catalog.societies.filter(c=>!['MSJC04','MSJC05'].includes(c.id))},published);
  assert.equal(catalog.societies.length,10);
  assert.deepEqual(catalog.societies.slice(-2).map(c=>c.id),['MSJC04','MSJC05']);
});
for(const [index,abi]of retained.entries())test(`seven-kernel router retains exact ABI${77+index} state, views, joins, commands and polls`,()=>{
  const room=JSON.parse(abi.newGame(`retained-${77+index}`,'LOCAL','duel','P0','watchers','18446744073709551615'));
  assert.equal(routed.catalog(room.state),abi.catalog());assert.equal(routed.view(room.state,0),abi.view(room.state,0));
  assert.throws(()=>current.view(room.state,0));
  const command=JSON.stringify({commandId:'old-ready',expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}});
  assert.equal(routed.applyRoom(room.state,0,command,'0'),abi.applyRoom(room.state,0,command,'0'));
  assert.equal(routed.joinGame(room.state,'P1','hunters'),abi.joinGame(room.state,'P1','hunters'));
  assert.equal(routed.pollRoom(room.state,0,'0'),abi.pollRoom(room.state,0,'0'));
  assert(room.state.includes('"seed":18446744073709551615'));
});
test('every original Native82 response preview, state, quote and transition routes through its published reducer',()=>{
  const fixture=load82();let commands=0,polls=0,quotes=0,projections=0;
  for(const scenario of fixture.scenarios) {
    scenario.initial.views.forEach((view,seat)=>{assert.deepEqual(JSON.parse(routed.view(scenario.initial.state,seat)),view);projections++;});
    for(const row of scenario.steps) {
      if(row.kind==='quote') {
        if(row.error) { let error;try{routed.quoteRoom(row.state,row.seat,JSON.stringify(row.request));}catch(e){error=String(e);}assert.equal(error,row.error); }
        else assert.deepEqual(JSON.parse(routed.quoteRoom(row.state,row.seat,JSON.stringify(row.request))),row.expected);
        quotes++;continue;
      }
      if(row.kind==='command'){assert.deepEqual(JSON.parse(routed.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;}
      if(row.kind==='poll'){assert.deepEqual(JSON.parse(routed.pollRoom(row.state,row.seat,row.serverNowMs)),row.expected);polls++;}
      const state=row.expected?.state??row.state;
      row.views.forEach((view,seat)=>{assert.deepEqual(JSON.parse(routed.view(state,seat)),view);projections++;});
    }
  }
  assert.deepEqual({commands,polls,quotes,projections},{commands:8,polls:2,quotes:2,projections:100});
});
