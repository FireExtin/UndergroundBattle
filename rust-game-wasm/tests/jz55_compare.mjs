// Current ABI against matching current native snapshots; no state rewriting.
// The two response chains below are independent continuous production command chains.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import { kernel as k, catalog as current } from './current-kernel.mjs';
assert(current.cards.some(card => card.id === 'JZ55'));
const root=resolve(process.argv[2]);let commands=0,rejections=0,checkpoints=0,views=0,fixtureViews=0,receiptCommands=0;
for(const folder of ['oracle-clean'])for(const name of readdirSync(resolve(root,folder)).sort()){
 if(!name.endsWith('.json'))continue;
 const row=JSON.parse(readFileSync(resolve(root,folder,name),'utf8'));
 let state=row.state;
 if(row.command){const actual=JSON.parse(k.applyRoom(state,row.seat,JSON.stringify(row.command),'0'));assert.deepEqual(actual,row.expected,folder+'/'+name);if(actual.errorCode){assert.equal(actual.state,state);rejections++;}state=actual.state;commands++;}
 else checkpoints++;
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(state,seat)),row.views[seat]);views++;}
}
const chains=[];
for(const filename of ['response-chain.json','cascade-response-chain.json']){
 const chain=JSON.parse(readFileSync(resolve(root,'response-final',filename),'utf8'));let state=chain.initialState;
 const kinds={};
 for(const step of chain.steps){assert.equal(step.state,state);const actual=JSON.parse(k.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));assert.deepEqual(actual,step.expected);state=actual.state;kinds[step.command.action.kind]=(kinds[step.command.action.kind]||0)+1;for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(state,seat)),step.views[seat]);views++;}}
 assert.equal(state,chain.finalState);assert.equal(kinds.beginResponse,1);assert.equal(kinds.submitResponse,1);
 const submitted=chain.steps.find(s=>s.command.action.kind==='submitResponse');const submittedRoom=JSON.parse(submitted.expected.state);
 if(filename==='response-chain.json'){
  assert.equal(submittedRoom.game.stack.length,1);assert.equal(submittedRoom.game.stack[0].id,chain.underlyingStackInstance);
  assert(!submittedRoom.game.regions.flatMap(r=>r.cards).some(c=>c.id===chain.targetInstance));
 }else{
  assert.equal(submittedRoom.game.stack.length,2);assert.equal(submittedRoom.game.stack[0].id,chain.underlyingStackInstance);
  assert.equal(submittedRoom.game.stack[0].frame.source.card.definition,'JZ31');
  assert.equal(submittedRoom.game.stack[1].frame.source.card.definition,'JC063');
  assert.equal(submittedRoom.game.stack[0].frame.actor,2);
  assert.deepEqual(JSON.parse(state).game.regions[2].influence,[0,1]);
 }
 chains.push({filename,commands:chain.steps.length,kinds});
}
for(const name of readdirSync(resolve(root,'frontend-final'))){const row=JSON.parse(readFileSync(resolve(root,'frontend-final',name),'utf8'));for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(row.state,seat)),row.views[seat]);fixtureViews++;}}
for(const name of ['game','submit']){const row=JSON.parse(readFileSync(resolve(root,'receipts',name+'-receipt.json'),'utf8'));assert.deepEqual(JSON.parse(k.applyRoom(row.initialState,2,JSON.stringify(row.command),'0')),row.expected);assert.equal(row.persistedState,row.expected.state);receiptCommands++;}
assert(commands>0&&rejections>0&&checkpoints>0&&fixtureViews>0);
console.log(JSON.stringify({independentNativeCommands:commands,rejections,serializationCheckpoints:checkpoints,nativeTransitionViews:views,continuousResponseChains:chains,fixtureViews,receiptCommands}));
