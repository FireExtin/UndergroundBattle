import test from 'node:test';
import assert from 'node:assert/strict';
import { createBotPolicy, restoreBotPolicy, assertBotAuthority, publicBotDisclosure, botDecisionContext, chooseBotCandidate, assertHumanBotCommandId } from '../src/bot-policy.mjs';
const roomId='0123456789abcdef01234567';
const policy=(difficulty='normal',seed='9007199254740993')=>createBotPolicy({roomId,difficulty,botSeats:[1],seed});
const authority=p=>({roomId:p.roomId,roomKind:'bot',difficulty:p.difficulty,humanSeats:[0],botSeats:[1]});
const view=(seat,changes={})=>({roomId,you:`p${seat}`,version:7,status:'playing',hand:[],legalActions:[{id:'p',kind:'pass',label:'让过'}],...changes});
const getter=(views,calls=[])=>seat=>{calls.push(seat);return views[seat]??view(seat);};

test('normal/easy context only requests its own legal view and excludes arbitrary server data',()=>{
 for(const d of ['easy','normal']){const p=policy(d),calls=[];const v=view(1,{opaqueState:'secret',otherHand:'secret'});const c=botDecisionContext(p,authority(p),1,getter({1:v},calls));assert.deepEqual(calls,[1]);assert.deepEqual(c.humanHands,[]);assert(!Object.hasOwn(c.view,'opaqueState'));assert(!Object.hasOwn(c.view,'otherHand'));}
});
test('hard sees only explicitly permitted current human hand, not the entire other view',()=>{
 const p=policy('hard'),calls=[];const c=botDecisionContext(p,authority(p),1,getter({0:view(0,{hand:[{cardId:'X',cost:5}],privateDeckTop:{cardId:'TOP'},opaqueState:'secret'})},calls));
 assert.deepEqual(calls,[1,0]);assert.deepEqual(c.humanHands,[{seat:0,cards:[{cardId:'X',cost:5}]}]);assert(!JSON.stringify(c).includes('TOP'));assert(!JSON.stringify(c).includes('secret'));
});
test('human/fair room and mismatched persistent identity cannot reuse hard policy',()=>{
 const p=policy('hard');for(const change of [{roomKind:'human'},{roomId:'f'.repeat(24)},{difficulty:'normal'},{botSeats:[0]},{humanSeats:[0,2]}]) assert.throws(()=>assertBotAuthority(p,{...authority(p),...change},1));
 assert.throws(()=>assertBotAuthority(p,authority(p),0));
});
test('mode, advantages and difficulty are restored under a closed policy schema',()=>{
 const p=policy('hard');for(const change of [{advantages:['peekHumanHands','deckRewrite']},{difficulty:'god'},{roomKind:'human'},{schema:2},{frontEndOverride:true}])assert.throws(()=>restoreBotPolicy(JSON.stringify({...p,...change})));
 assert.deepEqual(restoreBotPolicy(JSON.stringify(p)),p);assert(Object.isFrozen(p));assert(Object.isFrozen(p.botSeats));
 assert.throws(()=>createBotPolicy({roomId,difficulty:'hard',botSeats:[1],seed:'1',humanRoomOverride:true}));
});
test('uint64 seeds and PRNG state remain decimal strings above JS safe integer',()=>{
 const p=policy();const d=chooseBotCandidate(p,authority(p),1,getter({}));assert.equal(d.nextPolicy.seed,'9007199254740993');assert.equal(typeof d.nextPolicy.random,'string');assert.equal(d.nextPolicy.counter,'1');assert.deepEqual(restoreBotPolicy(JSON.stringify(d.nextPolicy)),d.nextPolicy);
 for(const seed of [9007199254740993,'01','-1','18446744073709551616'])assert.throws(()=>policy('normal',seed));assert.equal(policy('normal','18446744073709551615').seed,'18446744073709551615');
});
test('same seed, policy and legal view produce the same candidate after reopen',()=>{
 let a=policy('easy'),b=restoreBotPolicy(JSON.stringify(a));const get=getter({1:view(1,{legalActions:[{kind:'deploy',cardId:'A'},{kind:'deploy',cardId:'B'}]})});
 for(let i=0;i<12;i++){const x=chooseBotCandidate(a,authority(a),1,get),y=chooseBotCandidate(b,authority(b),1,get);assert.deepEqual(x,y);a=x.nextPolicy;b=restoreBotPolicy(JSON.stringify(y.nextPolicy));}
});
test('hard hand information changes ranking only within the Bot legal action list',()=>{
 const normal=policy(),hard=policy('hard');const actions=[{kind:'deploy',cardId:'own',region:0},{kind:'play',cardId:'discard',targetId:'p0'}];const get=getter({0:view(0,{hand:[{cost:6}]}),1:view(1,{legalActions:actions})});
 assert.equal(chooseBotCandidate(normal,authority(normal),1,get).action.action.cardId,'own');assert.equal(chooseBotCandidate(hard,authority(hard),1,get).action.action.cardId,'discard');assert.deepEqual(actions[0],{kind:'deploy',cardId:'own',region:0});
});
test('candidate strips UI labels and always requires authoritative Rust validation',()=>{
 const p=policy();const d=chooseBotCandidate(p,authority(p),1,getter({1:view(1,{legalActions:[{kind:'deploy',cardId:'A',region:1,id:'label',label:'not command data',hiddenPrivilege:true}]})}));
 assert.deepEqual(d.action,{kind:'game',action:{kind:'deploy',cardId:'A',region:1}});assert(d.mustValidateInRust);assert.equal(d.expectedVersion,7);
});
test('optional choices and mandatory choices only use own displayed option identities',()=>{
 for(const min of [0,1]){const p=policy('easy');const own=view(1,{pendingChoice:{id:'c',kind:'trigger',playerId:'p1',min,max:1,options:[{id:'allowed'}]},legalActions:[{kind:'choose',choiceId:'c'}]});const d=chooseBotCandidate(p,authority(p),1,getter({1:own}));assert.deepEqual(d.action.action.selected,min?['allowed']:[]);}
});
test('ordered and damage candidates preserve exact IDs and total, with acceptance left to Rust',()=>{
 const p=policy();for(const kind of ['order','investigation','damage']){const own=view(1,{pendingChoice:{id:'c',kind,playerId:'p1',amount:3,options:[{id:'a'},{id:'b'}]},legalActions:[{kind:'choose',choiceId:'c'}]});const d=chooseBotCandidate(p,authority(p),1,getter({1:own}));if(kind==='damage')assert.deepEqual(d.action.action.allocations,{a:3});else{assert.deepEqual(d.action.action.top,['a','b']);assert.deepEqual(d.action.action.bottom,[]);}assert(d.mustValidateInRust);}
});
test('wrong-room or stale human private projection is rejected',()=>{
 const p=policy('hard');for(const human of [view(0,{version:6}),view(0,{roomId:'f'.repeat(24)}),view(1)])assert.throws(()=>botDecisionContext(p,authority(p),1,getter({0:human})));
 assert.throws(()=>botDecisionContext(p,authority(p),1,getter({1:view(0)})));
 assert.throws(()=>botDecisionContext(p,authority(p),1,getter({1:view(1,{version:-1})})));
});
test('response starts an intent then submits only under the active intent',()=>{
 const p=policy();const w={id:'w',canBegin:true,previewSources:[{cardId:'A'}],members:[{playerId:'p1',status:'undecided'}]};const d=chooseBotCandidate(p,authority(p),1,getter({1:view(1,{responseWindow:w})}));assert.equal(d.action.kind,'beginResponse');
 const next=view(1,{responseWindow:{...w,myIntentId:d.action.intentId,members:[{playerId:'p1',status:'composing'}]},legalActions:[{kind:'play',cardId:'A',targetId:'T'}]});const s=chooseBotCandidate(d.nextPolicy,authority(p),1,getter({1:next}));assert.deepEqual(s.action,{kind:'submitResponse',windowId:'w',intentId:d.action.intentId,action:{kind:'play',cardId:'A',targetId:'T'}});
});
test('easy response passes; composing with no substantive action cancels and passes',()=>{
 for(const d of ['easy','normal']){const p=policy(d);const w={id:'w',canBegin:false,members:[{playerId:'p1',status:'undecided'}]};assert.equal(chooseBotCandidate(p,authority(p),1,getter({1:view(1,{responseWindow:w})})).action.kind,'passResponse');}
 const p=policy();const v=view(1,{responseWindow:{id:'w',myIntentId:'i',members:[{playerId:'p1',status:'composing'}]}});assert.equal(chooseBotCandidate(p,authority(p),1,getter({1:v})).action.kind,'cancelAndPass');
});
test('pause, finished room, passed member, absent action and foreign pending choice yield no candidate',()=>{
 const p=policy();for(const change of [{pause:{}},{status:'finished'},{legalActions:[]},{responseWindow:{members:[{playerId:'p1',status:'passed'}]}},{pendingChoice:{playerId:'p0'},legalActions:[{kind:'choose'}]}])assert.equal(chooseBotCandidate(p,authority(p),1,getter({1:view(1,change)})),null);
});
test('solo seat partition and known difficulty cannot silently widen room access',()=>{
 for(const botSeats of [[],[0],[2],[1,1],[2,1],[1,2],[1,2,4]])assert.throws(()=>createBotPolicy({roomId,difficulty:'normal',botSeats,seed:'0'}));assert.equal(createBotPolicy({roomId,difficulty:'normal',botSeats:[1,2,3],seed:'0'}).botSeats.length,3);
 assert.throws(()=>policy('unknown'));
});
test('public disclosure names actual advantage and excludes seeds, counter and private hand',()=>{
 const p=policy('hard'),v=publicBotDisclosure(p);assert.deepEqual(v.advantages,['peekHumanHands']);assert.match(v.description,/当前手牌/);for(const key of ['seed','random','counter','humanHands'])assert(!Object.hasOwn(v,key));assert.deepEqual(publicBotDisclosure(policy()).advantages,[]);
});
test('decision counter overflow fails without mutating input and server-reserved IDs cannot be supplied by humans',()=>{
 const p=restoreBotPolicy(JSON.stringify({...policy(),counter:'18446744073709551615'}));assert.throws(()=>chooseBotCandidate(p,authority(p),1,getter({})));assert.equal(p.counter,'18446744073709551615');assert.throws(()=>assertHumanBotCommandId('bot:reserved'));assert.doesNotThrow(()=>assertHumanBotCommandId('human-command'));
 assert.throws(()=>assertHumanBotCommandId(''));assert.throws(()=>assertHumanBotCommandId('x'.repeat(65)));
});
