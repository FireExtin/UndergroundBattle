// Directed scheduler contract against frozen2d8 imports. Synthetic admission
// oracle/store, not a Rust-card reproduction, D1 test or real browser match.
import test from 'node:test';
import assert from 'node:assert/strict';
import {RoomService} from '../UndergroundBattle-bot-difficulty/sites/src/service.mjs';
import {createBotPolicy,botCandidateAlternatives} from '../UndergroundBattle-bot-difficulty/sites/src/bot-policy.mjs';
const id='0123456789abcdef01234567';
function fixture(pending){
 const policy=createBotPolicy({roomId:id,difficulty:'normal',botSeats:[1],seed:'9007199254740993'});
 const authority={roomId:id,roomKind:'bot',difficulty:'normal',humanSeats:[0],botSeats:[1]};
 const options=Array.from({length:12},(_,n)=>({id:`v${n}`}));
 const view={roomId:id,you:'p1',version:7,status:'playing',hand:[],
   ...(pending?{pendingChoice:{id:'complex-pair',kind:'target',playerId:'p1',min:2,max:2,options}}:{}),
   legalActions:pending?[{kind:'choose',choiceId:'complex-pair'}]:[...Array.from({length:80},(_,n)=>({kind:'activate',cardId:`s${n}`,abilityId:'synthetic-budget'})),{kind:'pass'}]};
 const row={id,version:7,state:'opaque-QA-domain-not-parsed'};const calls=[];let writes=0;
 const admissible=action=>pending?action.kind==='game'&&JSON.stringify(action.action.selected)===JSON.stringify(['v10','v11']):action.kind==='game'&&action.action.kind==='pass';
 const kernel={assertSupported:s=>assert.equal(s,row.state),view:(s,seat)=>{assert.equal(s,row.state);assert.equal(seat,1);return JSON.stringify(view);},
   applyRoom:(s,seat,serialized)=>{assert.equal(s,row.state);assert.equal(seat,1);const c=JSON.parse(serialized);calls.push(c);const accepted=admissible(c.action);return JSON.stringify({state:s,version:accepted?8:7,seat,changed:accepted,outcome:accepted?'accepted':'rejected'});}};
 const store={room:async()=>row,botState:async()=>({genesis:JSON.stringify(policy),current:JSON.stringify(policy)}),command:async()=>{writes++;throw Error('unexpected commit');},system:async()=>{writes++;throw Error('unexpected tick');}};
 const service=new RoomService(null,kernel,()=>0,{botEnabled:true});service.store=store;
 return {policy,authority,view,row,calls,kernel,service,writes:()=>writes,admissible};
}
for(const pending of [true,false])test(`48 rejected candidates retain attention and never force pass (${pending?'pending pair':'ordinary actions'})`,async()=>{
 const f=fixture(pending),before=JSON.stringify({policy:f.policy,room:f.row});
 const candidates=botCandidateAlternatives(f.policy,f.authority,1,()=>f.view);
 assert.equal(candidates.length,48);assert(candidates.every(c=>!f.admissible(c.action)));
 // A visible witness outside this finite enumeration is admissible under the
 // synthetic oracle. Exhaustion therefore cannot mean "no legal action".
 const witness=pending?{kind:'game',action:{kind:'choose',choiceId:'complex-pair',selected:['v10','v11']}}:{kind:'game',action:{kind:'pass'}};
 assert(f.admissible(witness));
 const result=await f.service.advanceBots(id);
 assert.equal(f.calls.length,48);assert.equal(result.blocked,true);assert.equal(result.room,f.row);assert.equal(f.writes(),0);
 assert.equal(JSON.stringify({policy:f.policy,room:f.row}),before);
 assert(f.calls.every(c=>c.action.kind==='game'&&c.action.action.kind!=='pass'));
 assert(f.calls.every(c=>c.commandId===candidates[0].commandId&&c.expectedVersion===7));
});
