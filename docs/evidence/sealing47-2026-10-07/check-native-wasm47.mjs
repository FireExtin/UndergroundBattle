import * as abi from '/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm.js';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
const root='/tmp/sealing-family-evidence';
const wasm=readFileSync('/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
abi.initSync({module:wasm});
const catalog=JSON.parse(abi.catalog());
assert.equal(catalog.engineVersion,'rust-v0.2.47-sealed-cards-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.43-sealed-cards-candidate');
let checkpoints=0,commands=0,rejected=0,views=0,chainCommands=0;
for (const name of readdirSync(root+'/native47').sort()) {
  const d=JSON.parse(readFileSync(root+'/native47/'+name,'utf8'));
  let state=d.state;
  if (d.command) {
    const actual=JSON.parse(abi.applyRoom(state,d.seat,JSON.stringify(d.command),'0'));
    assert.deepEqual(actual,d.expected,name+': complete transition');
    state=actual.state;commands++;if(actual.outcome==='rejected')rejected++;
  } else checkpoints++;
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(abi.view(state,seat)),d.views[seat],name+': seat '+seat);views++;}
}
for(const folder of ['pause-native47','room-native47']) {
  const trace=JSON.parse(readFileSync(root+'/'+folder+'/native-trace.json','utf8'));
  let state=trace.initialState;
  for(const step of trace.steps) {
    const actual=JSON.parse(step.command?abi.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs):abi.pollRoom(state,step.seat,step.serverNowMs));
    assert.deepEqual(actual,step.transition,folder+': complete chained transition');state=actual.state;chainCommands++;
    for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(abi.view(state,seat)),step.views[seat],folder+': chained view');views++;}
  }
}
for(const filename of readdirSync(root+'/ui-native47')){
  const actual=JSON.parse(readFileSync(root+'/ui-native47/'+filename,'utf8'));assert.deepEqual(actual.catalog,catalog,filename+': catalog');
}
const old=readFileSync(root+'/preexisting-engine46/hegemony_wasm_bg.wasm');
assert.equal(createHash('sha256').update(old).digest('hex'),'9e6362c967046ff245b94ab650bc9ed0f3af32e45cc4ca979305b3921543a2a9');
const result={engine:catalog.engineVersion,pool:catalog.cardPoolVersion,cards:catalog.cards.length,societies:catalog.societies.length,
  actualWasm:true,commands,rejected,checkpoints,chainCommands,views,completeTransitionEqual:true,opaqueStateByteEqual:true,
  wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),preservedEngine46Bytes:old.length};
writeFileSync(root+'/native-wasm47-parity.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
