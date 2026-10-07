import * as abi from '/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm.js';
import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root='/tmp/search-family-evidence';
const wasm=readFileSync('/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
abi.initSync({module:wasm});
const catalog=JSON.parse(abi.catalog());
assert.equal(catalog.engineVersion,'rust-v0.2.48-jz50-death-search-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.44-jz50-death-search-candidate');
assert.equal(catalog.cards.length,107);assert.equal(catalog.societies.length,8);
assert(catalog.cards.some(c=>c.id==='JZ50'));assert(!catalog.cards.some(c=>['BQ104','XQ48'].includes(c.id)));
let commands=0,rejected=0,checkpoints=0,views=0,chainCommands=0;
for(const file of readdirSync(root+'/native48-exact').sort()) {
 const d=JSON.parse(readFileSync(root+'/native48-exact/'+file));let state=d.state;
 if(d.command) {
  const actual=JSON.parse(abi.applyRoom(state,d.seat,JSON.stringify(d.command),'0'));
  assert.deepEqual(actual,d.expected,file+': whole transition');state=actual.state;
  commands++;if(actual.outcome==='rejected')rejected++;
 } else checkpoints++;
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(abi.view(state,seat)),d.views[seat],file+': seat '+seat);views++;}
}
const chains={};
for(const folder of ['room-native48-exact','pause-native48-final','sealing-native48-final']) {
 const trace=JSON.parse(readFileSync(root+'/'+folder+'/native-trace.json'));let state=trace.initialState;let count=0;
 for(const step of trace.steps) {
  const actual=JSON.parse(step.command?abi.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs):abi.pollRoom(state,step.seat,step.serverNowMs));
  assert.deepEqual(actual,step.transition,folder+': whole chained transition');state=actual.state;chainCommands++;count++;
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(abi.view(state,seat)),step.views[seat],folder+': view');views++;}
 }chains[folder]=count;
}
for(const file of readdirSync(root+'/ui-native48-exact')) assert.deepEqual(JSON.parse(readFileSync(root+'/ui-native48-exact/'+file)).catalog,catalog,file+': Native catalog');
const old47=readFileSync(root+'/frozen-engine47/hegemony_wasm_bg.wasm');
assert.equal(createHash('sha256').update(old47).digest('hex'),'42ae5289b3998520bd8f9d1c35a32ffd2a1693960dc54579a9f47e184a01c4a9');
for(const [p,h] of [['preexisting-engine46','9e6362c967046ff245b94ab650bc9ed0f3af32e45cc4ca979305b3921543a2a9'],['frozen-engine45','afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7']]) assert.equal(createHash('sha256').update(readFileSync('/tmp/sealing-family-evidence/'+p+'/hegemony_wasm_bg.wasm')).digest('hex'),h);
const result={engine:catalog.engineVersion,pool:catalog.cardPoolVersion,cards:107,societies:8,actualWasm:true,commands,rejected,checkpoints,chainCommands,chains,views,completeTransitionEqual:true,opaqueStateByteEqual:true,wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),frozen45_46_47BytePreserved:true};
writeFileSync(root+'/native-wasm48-parity.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
