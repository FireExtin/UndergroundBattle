// Exact independent counterexample command payloads, unchanged across kernels.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import * as red from '/tmp/jc089-mechanism-game/rust-game-wasm/pkg/hegemony_wasm.js';
import * as green from '/tmp/jc089-glory-fix-game/rust-game-wasm/pkg/hegemony_wasm.js';
const redBytes=readFileSync('/tmp/jc089-mechanism-game/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
const greenBytes=readFileSync('/tmp/jc089-glory-fix-game/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
red.initSync({module:redBytes});green.initSync({module:greenBytes});
const commandsBytes=readFileSync('/tmp/review-jc089-mechanism-evidence/frozen31/command-stream.json');
const commands=JSON.parse(commandsBytes);assert.equal(commands.length,31);
const initial=JSON.parse(readFileSync('/tmp/review-jc089-mechanism-evidence/removal-boundary-fixture.json')).state;
const runs={};
for(const[name,kernel]of[['red',red],['green',green]]){
 let state=initial;const receipts=[];
 for(const step of commands){
  const before=state;const result=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNow));
  if(result.errorCode)assert.equal(result.state,before);
  state=result.state;receipts.push({commandId:step.command.commandId,errorCode:result.errorCode??null,version:result.version});
 }
 const view=JSON.parse(kernel.view(state,0));const host=view.regions[2].characters.find(c=>c.cardId==='JC018');
 runs[name]={submitted:31,accepted:receipts.filter(r=>!r.errorCode).length,rejected:receipts.filter(r=>r.errorCode).length,
  influence:view.regions[2].influence,currentRenown:host?.currentRenown??false,currentCombatGlory:host?.currentCombatGlory??false,receipts};
}
assert.equal(runs.red.accepted,31);assert.deepEqual(runs.red.influence,[2,0]);
assert.deepEqual(runs.green.influence,[1,0]);assert.equal(runs.green.currentRenown,false);assert.equal(runs.green.currentCombatGlory,true);
assert(runs.green.rejected>0,'old false-renown choice/response payloads must reject rather than add influence');
const summary={scope:'unchanged 31 original commands; old false-renown follow-ups reject on the fixed kernel',
 commandsSha256:createHash('sha256').update(commandsBytes).digest('hex'),originalDriverSha256:createHash('sha256').update(readFileSync('/tmp/review-jc089-mechanism-evidence/removal-boundary-repro.mjs')).digest('hex'),
 redWasmSha256:createHash('sha256').update(redBytes).digest('hex'),greenWasmSha256:createHash('sha256').update(greenBytes).digest('hex'),runs};
writeFileSync('/tmp/jc089-glory-fix-evidence/closure/exact-31-red-green.json',JSON.stringify(summary,null,2));console.log(JSON.stringify(summary));
