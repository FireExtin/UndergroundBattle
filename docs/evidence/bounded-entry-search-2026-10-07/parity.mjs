import {resolve,dirname} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
const pkg=resolve(process.env.WASM_PKG_DIR||'rust-game-wasm/pkg');
const abi=await import(pathToFileURL(resolve(pkg,'hegemony_wasm.js')).href);
import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root=resolve(process.env.ENTRY_SEARCH_EVIDENCE_DIR||dirname(fileURLToPath(import.meta.url)))+'/';
const canonical=x=>Array.isArray(x)?x.map(canonical):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,canonical(x[k])])):x;
const hash=x=>createHash('sha256').update(x).digest('hex');
abi.initSync({module:readFileSync(resolve(pkg,'hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(abi.catalog());assert.equal(catalog.cards.length,109);assert.equal(catalog.engineVersion,'rust-v0.2.50-bounded-entry-search-candidate');
let steps=0,accepted=0,rejected=0;
for(const name of readdirSync(root+'native-parity').filter(n=>(n.startsWith('step-')||n.startsWith('reject-'))&&n.endsWith('.json'))){const x=JSON.parse(readFileSync(root+'native-parity/'+name));
 const r=JSON.parse(abi.applyRoom(x.state,x.seat,JSON.stringify(x.command),'0'));assert.deepEqual(r,x.expected,name);assert.equal(r.state,x.expected.state,name+' state');
 for(let s=0;s<4;s++)assert.deepEqual(JSON.parse(abi.view(r.state,s)),x.views[s],name+' seat'+s);steps++;if(r.outcome==='accepted')accepted++;else rejected++;
}
const fresh=JSON.parse(readFileSync(root+'fresh/fresh-match.json'));const g=JSON.parse(fresh.initialState).game;
let state=JSON.parse(abi.newGameWithDeck(g.room_id,g.invite_code,g.mode,g.players[0].name,JSON.stringify(g.players[0].deck_snapshot),String(g.seed))).state;
for(const p of g.players.slice(1))state=JSON.parse(abi.joinGameWithDeck(state,p.name,JSON.stringify(p.deck_snapshot))).state;
assert.equal(state,fresh.initialState,'fresh factory+joins byte equality');
for(const [i,x] of fresh.steps.entries()){
 const r=JSON.parse(abi.applyRoom(state,x.seat,JSON.stringify(x.command),x.serverNowMs));assert.equal(hash(JSON.stringify(canonical(r))),x.transitionSha256,'fresh transition '+i);assert.equal(hash(r.state),x.stateSha256,'fresh state '+i);
 for(let s=0;s<4;s++)assert.equal(hash(JSON.stringify(canonical(JSON.parse(abi.view(r.state,s))))),x.viewSha256[s],'fresh view '+i+' seat'+s);
 state=r.state;
}
assert.equal(state,fresh.terminalState,'fresh exact terminal state');
const result={engine:catalog.engineVersion,pool:catalog.cardPoolVersion,ordinaryCards:109,explicitNativeSteps:steps,accepted,rejected,fourSeatViewParity:true,freshLegalFactoryByteEqual:true,freshNormalCommands:fresh.steps.length,freshWholeTransitionsAndOpaqueStatesExact:true,freshTerminalStateByteEqual:true,wasmSha256:hash(readFileSync(resolve(pkg,'hegemony_wasm_bg.wasm')))};
writeFileSync(root+'parity-result.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
