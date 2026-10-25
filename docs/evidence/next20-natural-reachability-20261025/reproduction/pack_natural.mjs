import fs from 'node:fs';
import readline from 'node:readline';
import assert from 'node:assert/strict';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const root=path.dirname(process.argv[1]);
const {kernel,catalog}=await import(pathToFileURL(path.join(process.cwd(),'rust-game-wasm/tests/current-kernel.mjs')));
const {writeCurrentFixture,sha256,currentSourceManifestSha256,currentWasmSha256}=await import(pathToFileURL(path.join(process.cwd(),'tools/current-native-source.mjs')));
const names=['bq034-natural-entry-counter','bq034-natural-shown-feed','cq25-xg25-natural-seal-move-search'];
const results=[];
for(const name of names){
 const file=path.join(root,'runs/fixed-native-06/native',name+'.jsonl');let initial,complete;const all=[];
 for await(const line of readline.createInterface({input:fs.createReadStream(file),crlfDelay:Infinity})){
  const row=JSON.parse(line);if(row.kind==='initial')initial=row;else if(row.kind==='step')all.push(row);else if(row.kind==='complete')complete=row;
 }
 assert(initial&&complete);assert.equal(all.length,complete.commands);assert.deepEqual(initial.catalog,catalog);assert.equal(initial.publicAdmission,false);
 assert.equal(all[2].command.action.action.kind,'start');const rejected=JSON.parse(kernel.applyRoom(all[2].state,all[2].seat,JSON.stringify(all[2].command),'0'));
 assert.equal(rejected.outcome,'rejected');assert.equal(rejected.state,all[2].state);assert.match(rejected.errorMessage,/不能加入当前玩家卡组/);
 const nativeSetup=all.slice(0,3),steps=all.slice(3),startedState=all[2].expected.state;
 let state=startedState,views=0,duplicates=0;
 for(const row of steps){
  assert.equal(row.state,state);const actual=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs));assert.deepEqual(actual,row.expected);state=actual.state;
  for(let seat=0;seat<2;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),row.views[seat]);views++;}
  assert.deepEqual(JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.serverNowMs)),row.duplicate);duplicates++;
 }
 assert.equal(state,complete.finalState);
 const packet={format:'next20-natural-reachability-v1',name,publicAdmission:false,candidateConstructionOnly:initial.candidateConstructionOnly,
  seedDecimal:initial.seedDecimal,decks:initial.decks,catalog:initial.catalog,nativeSetup,startedState,steps,complete,
  sourceProvenance:{rawNativeFile:file,rawNativeSha256:sha256(fs.readFileSync(file)),nativeSetupOnly:'3 real Native candidate-admission setup commands; production start deliberately rejects',currentSourceManifestSha256,currentWasmSha256}};
 writeCurrentFixture(`rust-game-wasm/tests/current112/${name}.json.gz`,packet);
 results.push({name,seed:initial.seedDecimal,nativeCommands:all.length,productionReplayedCommands:steps.length,views,duplicates,rawNativeSha256:packet.sourceProvenance.rawNativeSha256});
 console.log(JSON.stringify(results.at(-1)));global.gc?.();
}
fs.writeFileSync(path.join(root,'natural-cross-comparison.json'),JSON.stringify({currentSourceManifestSha256,currentWasmSha256,publicAdmission:false,cases:results},null,2)+'\n',{flag:'wx'});
