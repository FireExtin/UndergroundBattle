import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
import {frozenAbi77,frozenAbi78,frozenAbi79,frozenAbi80,frozenAbi81,frozenAbi82,frozenAbi83,frozenAbi84,frozenAbi85,frozenAbi86,frozenAbi87} from '../../tools/frozen-kernels.mjs';
import {kernel as current,catalog} from '../../rust-game-wasm/tests/current88-kernel.mjs';
import {loadFixture} from '../../tools/jc041-oracle.mjs';
import {routeKernels} from '../src/kernel-router.mjs';
const saved=[frozenAbi77(),frozenAbi78(),frozenAbi79(),frozenAbi80(),frozenAbi81(),frozenAbi82(),frozenAbi83(),frozenAbi84(),frozenAbi85(),frozenAbi86(),frozenAbi87()];
const retained=saved.map(s=>{const k=createRequire(import.meta.url)(s.jsPath);k.initSync({module:s.wasm});return k;});
const routed=routeKernels(current,retained);
test('ABI88 adds only complete XQ23 and keeps all 151 previous definitions, societies and presets',()=>{
  const published=JSON.parse(retained[10].catalog());
  assert.deepEqual({...catalog,engineVersion:published.engineVersion,cardPoolVersion:published.cardPoolVersion,cards:catalog.cards.filter(c=>c.id!=='XQ23')},published);
  assert.equal(catalog.cards.at(-1).id,'XQ23');assert.equal(catalog.cards.at(-1).supported,true);
});
for(const [i,k]of retained.entries())test(`twelve-kernel router retains exact ABI${77+i} views, joins, commands and polls`,()=>{
  const room=JSON.parse(k.newGame(`retained-${77+i}`,'LOCAL','duel','P0','watchers','18446744073709551615'));
  assert.equal(routed.catalog(room.state),k.catalog());assert.equal(routed.view(room.state,0),k.view(room.state,0));
  assert.throws(()=>current.view(room.state,0));
  const cmd=JSON.stringify({commandId:'old-ready',expectedVersion:0,action:{kind:'game',action:{kind:'ready'}}});
  assert.equal(routed.applyRoom(room.state,0,cmd,'0'),k.applyRoom(room.state,0,cmd,'0'));
  assert.equal(routed.joinGame(room.state,'P1','hunters'),k.joinGame(room.state,'P1','hunters'));
  assert.equal(routed.pollRoom(room.state,0,'0'),k.pollRoom(room.state,0,'0'));
  assert(room.state.includes('"seed":18446744073709551615'));
});
test('twelve-kernel router preserves original Native87 opaque states, journals and all four views',()=>{
  const f=loadFixture();let commands=0,views=0;
  for(const s of f.scenarios){
    s.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(s.initial.state,seat)),v);views++;});
    for(const row of s.steps){assert.deepEqual(JSON.parse(routed.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;
      row.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(row.expected.state,seat)),v);views++;});}
  }
  assert.deepEqual({commands,views},{commands:155,views:660});
});
test('exact retained87 preserves the original P2 UID probes and all four legal source-history trajectories',t=>{
  const bytes=readFileSync(new URL('../../docs/evidence/jc041-p2-source-region-2026-10-18/after.native-p2.json.gz',import.meta.url));
  const sha=b=>createHash('sha256').update(b).digest('hex');
  assert.equal(sha(bytes),'0cd458ea6bacf038ff4696f836645077a3e73fc2dd41fb7f5be289e8fa875a7f');
  const raw=gunzipSync(bytes);assert.equal(sha(raw),'33ea61c247bd98c3a71941cfb762511a034b09cb4adc44211cc5723612566874');
  const f=JSON.parse(raw),old=retained[10];assert.equal(f.expectedAccepted,false);
  assert.equal(sha(f.originalState),'ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c');
  assert.throws(()=>current.view(f.originalState,0));
  f.originalViews.forEach((v,seat)=>assert.deepEqual(JSON.parse(routed.view(f.originalState,seat)),v));
  let badChecks=0,commands=0,views=0;
  for(const c of f.cases){
    const error=e=>{assert.equal(String(e),c.roomError);return true;};
    for(const k of [old,routed]){
      for(let seat=0;seat<4;seat++){assert.throws(()=>k.view(c.state,seat),error);badChecks++;}
      assert.throws(()=>k.applyRoom(c.state,0,JSON.stringify(c.command),'0'),error);badChecks++;
      assert.throws(()=>k.pollRoom(c.state,0,'0'),error);badChecks++;
    }
  }
  for(const s of f.legal){
    s.initial.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(s.initial.state,seat)),v);views++;});
    for(const row of s.steps){
      assert.deepEqual(JSON.parse(routed.applyRoom(row.state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.expected);commands++;
      row.views.forEach((v,seat)=>{assert.deepEqual(JSON.parse(routed.view(row.expected.state,seat)),v);views++;});
    }
  }
  assert.deepEqual({badChecks,commands,views},{badChecks:48,commands:56,views:240});
  t.diagnostic(JSON.stringify({originalP2ArchiveUnchanged:true,exactAbi87:saved[10].identity.wasmSha256,badChecks,commands,views}));
});
