import * as abi from '/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm.js';
import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root='/tmp/sealing-family-evidence/restart49';
const wasm=readFileSync('/tmp/undergroundbattle-shared-cloud/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
abi.initSync({module:wasm});
const catalog=JSON.parse(abi.catalog());
assert.equal(catalog.engineVersion,'rust-v0.2.49-sealed-restart-candidate');
assert.equal(catalog.cardPoolVersion,'limited-v2.44-jz50-death-search-candidate');
assert.equal(catalog.cards.length,107);assert.equal(catalog.societies.length,8);
assert(catalog.cards.some(c=>c.id==='JZ50'));assert(!catalog.cards.some(c=>['BQ104','XQ48'].includes(c.id)));
let views=0,chainCommands=0;
const chains={};
for(const folder of ['jz50','pause','sealing','restart-trace','fresh']) {
 const trace=JSON.parse(readFileSync(root+'/'+folder+'/native-trace.json'));let state=trace.initialState;let count=0;
 for(const step of trace.steps) {
  const actual=JSON.parse(step.command?abi.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs):abi.pollRoom(state,step.seat,step.serverNowMs));
  assert.deepEqual(actual,step.transition,folder+': whole chained transition');state=actual.state;chainCommands++;count++;
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(abi.view(state,seat)),step.views[seat],folder+': view');views++;}
 }chains[folder]=count;
}
const fresh=JSON.parse(readFileSync(root+'/fresh/fresh-match.json'));
const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(k=>[k,canonical(value[k])])):value;
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
let state=fresh.initialState;
for(const [index,step] of fresh.steps.entries()) {
 const actual=JSON.parse(abi.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
 assert.equal(hash(JSON.stringify(canonical(actual))),step.transitionSha256,'fresh '+index+': complete transition');
 assert.equal(hash(actual.state),step.stateSha256,'fresh '+index+': exact opaque state');
 for(let seat=0;seat<4;seat++){assert.equal(hash(JSON.stringify(canonical(JSON.parse(abi.view(actual.state,seat))))),step.viewSha256[seat],'fresh view '+index+'/'+seat);views++;}
 state=actual.state;
 if(index===fresh.steps.length-3)assert.equal(state,fresh.terminalState);
 if(index===fresh.steps.length-2)assert.equal(state,fresh.restartState);
}
assert.equal(state,fresh.nextChoiceState);
const old48=readFileSync(root+'/frozen-engine48.wasm');
assert.equal(hash(old48),'90ba2fdc49c8dc256b69a2b4cc2acdf3acfaf87060140ec2829701db10805114');
const old47=readFileSync('/tmp/search-family-evidence/frozen-engine47/hegemony_wasm_bg.wasm');
assert.equal(createHash('sha256').update(old47).digest('hex'),'42ae5289b3998520bd8f9d1c35a32ffd2a1693960dc54579a9f47e184a01c4a9');
for(const [p,h] of [['preexisting-engine46','9e6362c967046ff245b94ab650bc9ed0f3af32e45cc4ca979305b3921543a2a9'],['frozen-engine45','afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7']]) assert.equal(createHash('sha256').update(readFileSync('/tmp/sealing-family-evidence/'+p+'/hegemony_wasm_bg.wasm')).digest('hex'),h);
const result={engine:catalog.engineVersion,pool:catalog.cardPoolVersion,cards:107,societies:8,actualWasm:true,additionalChainExecutions:chainCommands,chains,views,completeTransitionEqual:true,opaqueStateByteEqual:true,wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),freshCommands:fresh.steps.length,freshScope:fresh.scope,frozen45_46_47_48BytePreserved:true};
writeFileSync(root+'/native-wasm49-parity.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
