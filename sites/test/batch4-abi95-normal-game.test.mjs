// One normal legal-input smoke client from a fresh two-player lobby to terminal.
// It controls two synthetic seats through their own public views; it is not a
// product Bot, browser playtest, card-batch acceptance or archive-boundary probe.
import test from 'node:test';
import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
import {kernel,catalog} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {harness} from './xq23-http-harness.mjs';
const seeded=JSON.parse(kernel.newGame('95000000000000000000abcd','95ABCD','duel','Normal0','hunters','18446744073709551520'));
const joined=JSON.parse(kernel.joinGame(seeded.state,'Normal1','hunters'));
const initial={state:joined.state,views:[0,1].map(s=>JSON.parse(kernel.view(joined.state,s)))};
const definitions=new Map(catalog.cards.map(c=>[c.id,c]));
function choiceAction(choice){
 const base={kind:'choose',choiceId:choice.id};
 if(choice.kind==='investigation'||choice.kind==='order')return{...base,top:choice.options.map(o=>o.id),bottom:[]};
 if(choice.kind==='bottom')return{...base,bottom:choice.options.map(o=>o.id)};
 if(choice.kind==='damage'){
  let remaining=choice.amount;const allocations={};
  for(const option of choice.options){
   const card=definitions.get(option.card?.cardId),printed=(card?.text||'').match(/护卫\s*([0-9]+)/);
   const guard=printed?Number(printed[1]):0,assigned=Math.min(remaining,guard);
   if(assigned){allocations[option.id]=assigned;remaining-=assigned;}
  }
  if(remaining)allocations[choice.options[0].id]=(allocations[choice.options[0].id]||0)+remaining;
  return{...base,allocations};
 }
 return{...base,selected:choice.options.slice(0,choice.min||0).map(o=>o.id)};
}
test('ABI95 normal legal HTTP game reaches terminal from a fresh lobby',async t=>{
 const h=await harness(t,{initial});let views=initial.views;const trace=[];let ended=false;
 for(let i=0;i<2500;i++){
  views=await Promise.all([0,1].map(async s=>{const r=await h.api('state',s);assert.equal(r.status,200);return r.body;}));
  if(views[0].status==='finished'){ended=true;break;}
  let seat=views.findIndex(v=>v.pendingChoice),action;
  if(seat>=0)action={kind:'game',action:choiceAction(views[seat].pendingChoice)};
  else{
   seat=views.findIndex(v=>v.responseWindow?.members.some(m=>m.playerId===v.you&&m.status==='undecided'));
   if(seat>=0)action={kind:'passResponse',windowId:views[seat].responseWindow.id};
   else{
    const priorities=['start','ready','deploy','asset','pass'];let entry;
    for(const kind of priorities){seat=views.findIndex(v=>v.legalActions.some(a=>a.kind===kind)&&(kind!=='ready'||!v.players.find(p=>p.id===v.you)?.ready));if(seat>=0){entry=views[seat].legalActions.find(a=>a.kind===kind);break;}}
    assert(entry,JSON.stringify({message:'A normal public legal action must exist',status:views[0].status,turn:views[0].turn,step:views[0].step}));action={kind:'game',action:JSON.parse(entry.id)};
   }
  }
  const version=views[seat].version,command={commandId:`normal95-${i}`,expectedVersion:version,action};
  const result=await h.api('commands',seat,command);assert.equal(result.status,200,JSON.stringify({index:i,seat,command,error:result.body}));
  trace.push({seat,command,turn:result.body.turn,phase:result.body.phase,step:result.body.step,version:result.body.version});
  if(i&&i%200===0)t.diagnostic(JSON.stringify({normalCommands:i+1,turn:result.body.turn,phase:result.body.phase}));
 }
 assert(ended,'Normal legal-input game must reach a terminal state within the explicit smoke budget');assert([0,1].includes(views[0].winnerTeam));assert.equal(views[1].winnerTeam,views[0].winnerTeam);
 const result={scope:'One normal current Worker legal-input duel, not browser or full50 acceptance',engine:views[0].versions.engine,commands:trace.length,turn:views[0].turn,winnerTeam:views[0].winnerTeam,terminalViews:views,trace};
 if(process.env.BATCH4_ABI95_NORMAL_OUT)writeFileSync(process.env.BATCH4_ABI95_NORMAL_OUT,JSON.stringify(result));
 t.diagnostic(JSON.stringify({engine:result.engine,normalCommands:trace.length,turn:result.turn,winnerTeam:result.winnerTeam,terminal:true,legalInputsOnly:true}));
});
