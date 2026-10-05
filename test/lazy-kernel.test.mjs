import {test} from 'node:test';
import assert from 'node:assert/strict';
import {lazyKernel} from '../src/lazy-kernel.mjs';
import {routeKernels} from '../src/kernel-router.mjs';

test('routing starts no core and initializes only each used fixed identity once', () => {
 const calls=Array.from({length:26},()=>({init:0,catalog:0}));
 const identities=calls.map((_,i)=>({rulesVersion:'rules',cardPoolVersion:'pool'+i,engineVersion:'engine'+i}));
 const cores=calls.map((count,i)=>lazyKernel({
  initSync(){count.init++;},
  catalog(){count.catalog++;return JSON.stringify(identities[i]);},
  stateIdentity(state){return state;},
  pollRoom(state){return state;},
  view(state){return 'core'+i+':'+state;},
 },{},identities[i]));
 const routed=routeKernels(cores[0],cores.slice(1));
 assert.ok(calls.every(c=>c.init===0&&c.catalog===0));
 assert.equal(routed.catalog(),JSON.stringify(identities[0]));
 assert.deepEqual(calls[0],{init:1,catalog:1});
 assert.ok(calls.slice(1).every(c=>c.init===0));
 const state=JSON.stringify({state_schema:3,versions:{rules:'rules',cardPool:'pool25',engine:'engine25'}});
 assert.equal(routed.view(state,0),'core25:'+state);
 assert.equal(routed.view(state,0),'core25:'+state);
 assert.equal(routed.catalog(state),JSON.stringify(identities[25]));
 assert.deepEqual(calls[25],{init:1,catalog:1});
 assert.ok(calls.slice(1,25).every(c=>c.init===0));
 const unknown=JSON.stringify({state_schema:3,versions:{rules:'rules',cardPool:'pool25',engine:'unknown'}});
 assert.throws(()=>routed.view(unknown,0),/Unsupported/);
 assert.ok(calls.slice(1,25).every(c=>c.init===0));
});

test('a registered identity must match the actual frozen ABI before any state call', () => {
 let views=0;
 const kernel=lazyKernel({initSync(){},catalog(){return JSON.stringify({rulesVersion:'r',cardPoolVersion:'p',engineVersion:'actual'});},view(){views++;}}, {},{rulesVersion:'r',cardPoolVersion:'p',engineVersion:'incorrect'});
 assert.throws(()=>kernel.view('opaque',0),/identity mismatch/);
 assert.equal(views,0);
});
