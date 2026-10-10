// Offline, read-only audit of the REAL local natural table. Never inject state in a Worker/D1.
import * as abi from '/tmp/lantern-site-fce0-official/rust-game-wasm/pkg/hegemony_wasm.js';
import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root=fileURLToPath(new URL('.',import.meta.url));
const server='/tmp/lantern-site-fce0-official/dist/server/';
const wasmFiles=readdirSync(server).filter(n=>n.endsWith('.wasm'));assert.equal(wasmFiles.length,1);
const wasm=readFileSync(server+wasmFiles[0]);
const hash=x=>createHash('sha256').update(x).digest('hex');
assert.equal(hash(wasm),'9b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885');
abi.initSync({module:wasm});
const catalog=JSON.parse(abi.catalog());assert.equal(catalog.cards.length,107);assert.equal(catalog.societies.length,8);
assert(!catalog.cards.some(c=>['BQ104','XQ48'].includes(c.id)));
const trace=JSON.parse(readFileSync(root+'natural-canonical-journal.json'));
const initial=JSON.parse(trace.initialState),g=initial.game;
assert.equal(g.seed,16);assert.equal(g.players.length,1);
let state=JSON.parse(abi.newGameWithDeck(g.room_id,g.invite_code,g.mode,g.players[0].name,JSON.stringify(g.players[0].deck_snapshot),String(g.seed))).state;
assert.ok(state===trace.initialState,'initial persisted state must equal untouched actual factory output');
const commandKinds={},gameKinds={};let commands=0,ticks=0,joins=0,previousVersion=0;
for(const row of trace.journal){
 assert.equal(row.version,++previousVersion,'canonical revision chain must be continuous');
 if(row.entry.JoinWithDeck){
  const x=row.entry.JoinWithDeck;state=JSON.parse(abi.joinGameWithDeck(state,x.name,JSON.stringify(x.deck_draft))).state;joins++;
 }else{
  assert.deepEqual(Object.keys(row.entry),['SessionEvents']);const events=row.entry.SessionEvents.events;
  assert.equal(events.length,1,'audit uses observed single-event journal rows');
  const e=events[0];let result;
  if(e.Command){
   const x=e.Command;result=JSON.parse(abi.applyRoom(state,x.seat,JSON.stringify(x.command),String(x.server_now_ms)));
   assert.equal(result.outcome,'accepted');commands++;
   const kind=x.command.action.kind;commandKinds[kind]=(commandKinds[kind]||0)+1;
   if(kind==='game'){const k=x.command.action.action.kind;gameKinds[k]=(gameKinds[k]||0)+1;}
  }else{
   assert.deepEqual(Object.keys(e),['Tick']);result=JSON.parse(abi.pollRoom(state,0,String(e.Tick.server_now_ms)));ticks++;
  }
  assert.deepEqual(result.journal,events,'actual WASM must reproduce exact canonical journal event');state=result.state;
 }
 assert.equal(JSON.parse(abi.view(state,0)).version,row.version);
}
assert.ok(state===trace.terminalState,'exact opaque terminal state must equal local D1 state');
assert.equal(previousVersion,trace.terminalVersion);assert.equal(commands,391);assert.equal(joins,1);assert.equal(ticks,5);
const finalView=JSON.parse(abi.view(state,0));assert(finalView.pause);assert.equal(finalView.turn,7);
const result={scope:trace.scope,wasmSha256:hash(wasm),wasmBytes:wasm.length,compiledWorkerSha256:hash(readFileSync(server+'index.js')),engine:catalog.engineVersion,cardPool:catalog.cardPoolVersion,cards:107,societies:8,
 factorySeed:16,initialFactoryByteEqual:true,canonicalJournalRevisions:previousVersion,normalCommittedCommands:commands,normalJoinWithLegalDeck:joins,timingPollEvents:ticks,exactJournalEventParity:true,terminalOpaqueStateByteEqual:true,
 initialStateSha256:hash(trace.initialState),terminalStateSha256:hash(state),commandKinds,gameKinds,terminalTurn:finalView.turn,terminalPaused:true,bq104Xq48Absent:true,noLiveStateWritesDuringReplay:true};
writeFileSync(root+'canonical-wasm-replay-check.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
