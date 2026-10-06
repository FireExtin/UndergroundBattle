import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {writeFileSync,mkdirSync} from 'node:fs';
import {startHarness} from './local-room-harness.mjs';
import {groupObjectActions} from '/workspace/lantern-ui-integration-game/web/src/game/objectActions.ts';
const require=createRequire('/workspace/lantern-ui-integration-game/web/package.json');
const {chromium}=require('playwright');
const run=process.argv[2]||'four-seat-run-1';
const folder='/workspace/lantern-ui-e2e-evidence/'+run;mkdirSync(folder,{recursive:true});
const h=await startHarness(run,9);
const browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});
const slots=[],latest=[],steps=[],shots=[],checks={};let version=0,roomId;
const delay=ms=>new Promise(r=>setTimeout(r,ms));
const save=()=>writeFileSync(folder+'/progress.json',JSON.stringify({run,roomId,fixedSeed:9,version,steps,shots,checks},null,2));
async function fresh(s){
 const started=Date.now();
 while(!latest[s]||latest[s].version<version){if(Date.now()-started>12000)throw Error('UI poll stale seat '+s+' '+latest[s]?.version+' < '+version);await delay(80);}
 await delay(70);return latest[s];
}
async function clickCommand(s,locator,kind){
 const p=slots[s].page;
 const wait=p.waitForResponse(r=>r.request().method()==='POST'&&r.url().endsWith('/commands'),{timeout:10000});
 await locator.click({timeout:10000});const r=await wait,b=await r.json(),command=r.request().postDataJSON();
 assert.equal(r.status(),200,JSON.stringify(b));
 const inner=command.action.action||command.action;if(kind)assert.equal(inner.kind,kind);
 version=Math.max(version,b.version);steps.push({seat:s,command,status:r.status(),view:b});save();
 if(steps.length%15===0)console.log('UI commands',steps.length,'turn',b.turn,'phase',b.phase,b.step);
 return b;
}
async function shot(s,name){
 await fresh(s);const p=slots[s].page;const path=folder+'/'+name+'-seed9.png';await p.screenshot({path,fullPage:false});
 shots.push({path,roomId,fixedSeed:9,seat:s,version:latest[s].version,turn:latest[s].turn,url:p.url(),viewport:p.viewportSize()});save();
}
const css=(name,value)=>`[${name}=${JSON.stringify(value)}]`;
async function choice(s){
 const v=await fresh(s),p=slots[s].page,c=v.pendingChoice;
 assert(c);const dialog=p.locator(css('data-choice-id',c.id));await dialog.waitFor();
 if(c.kind==='mulligan')return clickCommand(s,dialog.getByRole('button',{name:'保留全部手牌'}),'choose');
 if(c.kind==='damage'){
  const option=dialog.locator('[data-choice-option]').first();for(let n=0;n<c.amount;n++)await option.click();
 }else if(!['order','investigation','region_return'].includes(c.kind)){
  const n=c.min??1;for(const o of c.options.slice(0,n))await dialog.locator(css('data-choice-option',o.id)).click();
 }
 return clickCommand(s,dialog.getByRole('button',{name:'确认选择',exact:true}),'choose');
}
async function perform(s,a,{cancel=false,lostAck=false}={}){
 const v=await fresh(s),p=slots[s].page;
 if(a.kind==='choose')return choice(s);
 if(a.kind==='pass'&&v.responseWindow){return clickCommand(s,p.getByRole('button',{name:'不连锁，让过',exact:true}),'passResponse');}
 if(a.cardId)await p.locator('button.hg-card'+css('data-card-instance',a.cardId)+':visible').first().click();
 const group=groupObjectActions(v.legalActions.filter(x=>x.cardId===a.cardId)).find(g=>g.actions.some(x=>x.id===a.id));
 const destination=a.cardId&&(a.targetId||a.region!==undefined);
 async function selectDestination(){
  await p.locator(css('data-object-action',group.key)).click();
  if(a.targetId){
   const card=p.locator('button.hg-card'+css('data-card-instance',a.targetId)+':visible').first();
   if(await card.count())await card.click();
   else if(await p.locator(css('data-player-target',a.targetId)).count())await p.locator(css('data-player-target',a.targetId)).click();
   else await p.locator(css('data-attachment-instance',a.targetId)+':visible').first().click();
  }
  if(a.region!==undefined)await p.locator('#hg-region-'+a.region+' .hg-region-center').click();
 }
 if(destination){
  assert(group,'missing group');await selectDestination();
  if(cancel){
   const before=await h.snapshot(roomId);await p.getByRole('button',{name:'取消选目标',exact:true}).click();const after=await h.snapshot(roomId);
   assert.equal(before.room.state,after.room.state);assert.deepEqual(before.commands,after.commands);assert.deepEqual(before.journal,after.journal);
   checks.targetCancellation=(checks.targetCancellation||0)+1;await selectDestination();await shot(s,'target-confirm-'+checks.targetCancellation);
  }
 }
 const button=destination?p.locator('[aria-label="确认目标行动"] '+css('data-action-id',a.id)):p.locator(css('data-action-id',a.id)).filter({visible:true}).first();
 if(!lostAck)return clickCommand(s,button,a.kind);
 let captured,deliver=false,lostResponses=0;
 const loseAck=async route=>{
  if(deliver){await route.continue();return;}
  const req=route.request(),response=await route.fetch(),receipt={command:req.postDataJSON(),body:await response.json(),status:response.status()};
  if(!captured)captured=receipt;else{assert.deepEqual(receipt.command,captured.command);assert.deepEqual(receipt.body,captured.body);}
  lostResponses++;await route.abort('failed');
 };
 // Keep both immediate retry acknowledgements unavailable, plus reconnect retries,
 // until the real UI has retained its pending command across reload.
 const losePoll=route=>route.abort('failed');
 await p.route('**/api/rooms/*/commands',loseAck);
 await p.route('**/api/rooms/*/state?afterVersion=*',losePoll);
 const before=await h.snapshot(roomId);await button.click();while(!captured)await delay(50);
 assert.equal(captured.status,200);version=captured.body.version;
 const committed=await h.snapshot(roomId);assert.equal(committed.commands.length,before.commands.length+1);
 await p.getByRole('button',{name:'确认上一行动',exact:true}).waitFor();
 const pendingBefore=await p.evaluate(()=>localStorage.getItem('hegemony.pending.v1'));assert(pendingBefore);
 const beforeReloadLost=lostResponses;await p.reload();const retry=p.getByRole('button',{name:'确认上一行动',exact:true});await retry.waitFor();
 const retryStarted=Date.now();while(lostResponses<beforeReloadLost+2||!await retry.isEnabled()){if(Date.now()-retryStarted>15000)throw Error('reload retry did not settle');await delay(50);}
 assert.equal(await p.evaluate(()=>localStorage.getItem('hegemony.pending.v1')),pendingBefore);
 assert(lostResponses>=2);checks.lostAcknowledgements=lostResponses;deliver=true;
 const replay=await clickCommand(s,p.getByRole('button',{name:'确认上一行动',exact:true}),a.kind);
 assert.deepEqual(steps.at(-1).command,captured.command);assert.deepEqual(replay,captured.body);
 const after=await h.snapshot(roomId);assert.equal(after.room.state,committed.room.state);assert.deepEqual(after.commands,committed.commands);assert.deepEqual(after.journal,committed.journal);
 await p.unroute('**/api/rooms/*/commands',loseAck);await p.unroute('**/api/rooms/*/state?afterVersion=*',losePoll);
 checks.lostAckReloadAndUIReceiptReplay=true;checks.replayedCommand=captured.command;await shot(s,'lost-ack-recovered');return replay;
}
async function passOrChoice(){
 const vs=await Promise.all(slots.map((_,s)=>fresh(s)));
 const chooser=vs.findIndex(v=>v.pendingChoice);if(chooser>=0)return choice(chooser);
 for(let s=0;s<4;s++){
  const a=vs[s].legalActions.find(a=>a.kind==='pass');
  if(a&&(!vs[s].responseWindow||vs[s].responseWindow.members.find(m=>m.playerId===vs[s].you)?.status==='undecided'))return perform(s,a);
 }
 throw Error('no UI pass/choice at '+JSON.stringify(vs.map(v=>({you:v.you,phase:v.phase,step:v.step,actions:v.legalActions,window:v.responseWindow}))));
}
try{
 for(let s=0;s<4;s++){
  const context=await browser.newContext({viewport:{width:1440,height:1000}}),page=await context.newPage();slots.push({context,page});
  page.on('response',async r=>{if(!r.url().includes('/api/')||r.status()!==200)return;try{const b=await r.json(),v=b.view||b;if(v.you&&v.roomId){if(!latest[s]||v.version>=latest[s].version)latest[s]=v;version=Math.max(version,v.version);}}catch{}});
  await page.goto(h.url);await page.locator('[data-deck-id="responders"]').click();await page.getByLabel('你的称呼').fill(['灯席一','灯席二','灯席三','灯席四'][s]);
  if(s===0)await page.getByRole('button',{name:/四人协作/}).click();else{await page.getByRole('button',{name:'邀请码加入',exact:true}).click();await page.getByLabel('邀请码',{exact:true}).fill(slots[0].session.inviteCode);}
  const response=page.waitForResponse(r=>r.request().method()==='POST'&&r.url().endsWith(s===0?'/api/rooms':'/api/rooms/join'));
  await page.getByRole('button',{name:s===0?'创建牌桌 →':'加入牌桌 →',exact:true}).click();const r=await response;assert.equal(r.status(),200);slots[s].session=await r.json();assert.equal(slots[s].session.seat,s);roomId=slots[s].session.roomId;
 }
 for(let s=0;s<4;s++)await slots[s].context.storageState({path:folder+'/original-seat-'+s+'-storage.json'});
 checks.fourIndependentBrowserSeats=true;
 const initial=await h.snapshot(roomId);assert.equal(initial.seats.length,4);assert.match(initial.room.initial_state,/"seed":9[,}]/);checks.fixedSeedVerifiedFromInitialState=true;
 for(let s=0;s<4;s++){const v=await fresh(s);await perform(s,v.legalActions.find(a=>a.kind==='ready'));}
 await perform(0,(await fresh(0)).legalActions.find(a=>a.kind==='start'));
 for(let i=0;i<100;i++){const vs=await Promise.all(slots.map((_,s)=>fresh(s)));if(vs.some(v=>v.phase==='action')&&!vs.some(v=>v.pendingChoice))break;await passOrChoice();}
 await shot(0,'four-seats-action');
 const assetDone=new Set(),deployed=new Set();let activated=false,responded=false,responseBefore;
 for(let n=0;n<180;n++){
  const vs=await Promise.all(slots.map((_,s)=>fresh(s)));assert(vs[0].turn===1,'first-turn plan ran out');
  if(vs.some(v=>v.pendingChoice)){await passOrChoice();continue;}
  if(vs.some(v=>v.stack.length)){
   if(activated&&!responded&&vs[2].responseWindow?.canBegin&&vs[2].responseWindow.members.find(m=>m.playerId==='p2')?.status==='undecided'){
    await clickCommand(2,slots[2].page.getByRole('button',{name:'连锁',exact:true}),'beginResponse');
    const response=(await fresh(2)).legalActions.find(a=>a.abilityId==='funeral');assert(response,'seed9 funeral response unavailable');
    const before=await fresh(2);responseBefore={target:response.targetId,hand:before.hand.length,actorDeck:before.players.find(p=>p.id==='p2').deckCount,ownerDeck:before.players.find(p=>p.id==='p1').deckCount};
    await perform(2,response,{cancel:true});responded=true;checks.actualTargetedStackResponse=true;await shot(2,'two-effect-response');
   }else await passOrChoice();
   continue;
  }
  if(activated&&responded){
   assert(!vs[2].graveyard.some(c=>c.instanceId===responseBefore.target));
   assert.equal(vs[2].hand.length,responseBefore.hand);
   assert.equal(vs[2].players.find(p=>p.id==='p1').deckCount,responseBefore.ownerDeck+1);
   assert.equal(vs[2].players.find(p=>p.id==='p2').deckCount,responseBefore.actorDeck-1);
   checks.funeralReturnedTargetAndDrewOne=true;break;
  }
  assert.equal(vs[0].phase,'action','script action plan missed its legal window');
  let acted=false;
  if(activated&&!responded){
   const card=vs[0].hand.find(c=>c.cardId==='JC125');
   const a=card&&vs[0].legalActions.find(a=>a.kind==='deploy'&&a.cardId===card.instanceId&&a.region===0);
   if(a){await perform(0,a);acted=true;}
  }
  for(let s=0;s<4&&!acted;s++)if(!assetDone.has(s)){
   const wanted=s===0||s===3?'LC19':s===1?'JC049':'JC125';const card=vs[s].hand.find(c=>c.cardId===wanted);
   const a=card&&vs[s].legalActions.find(a=>a.kind==='asset'&&a.cardId===card.instanceId);
   if(a){await perform(s,a,{lostAck:s===2});assetDone.add(s);acted=true;}
  }
  for(const s of [0,2,1,3])if(!acted&&!deployed.has(s)){
   const wanted=s===1?'JC042':'JC125';const card=vs[s].hand.find(c=>c.cardId===wanted);
   const a=card&&vs[s].legalActions.find(a=>a.kind==='deploy'&&a.cardId===card.instanceId&&a.region===(s===1||s===3?2:0));
   if(a){await perform(s,a,{cancel:s===0});deployed.add(s);acted=true;}
  }
  if(!acted&&deployed.has(1)&&deployed.has(2)&&assetDone.has(2)&&!activated){
   const source=vs[1].regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC042'&&c.controller==='p1');const a=source&&vs[1].legalActions.find(a=>a.cardId===source.instanceId&&a.abilityId==='reduce-next');
   if(a){await perform(1,a);activated=true;acted=true;}
  }
  if(!acted)await passOrChoice();
 }
 assert(activated&&responded,'real targeted response was not reached');
 for(let n=0;n<600;n++){if((await fresh(0)).turn>=2||(await fresh(0)).status==='finished')break;await passOrChoice();}
 const v=await fresh(0);assert(v.turn>=2||v.status==='finished');checks.completeFirstRound=true;checks.finalTurn=v.turn;
 await shot(0,'round-complete');
 const beforeNeg=await h.snapshot(roomId),command=checks.replayedCommand;
 const response=await slots[2].context.request.post(h.url+'/api/rooms/'+roomId+'/commands',{data:command,headers:{Authorization:'Bearer '+slots[2].session.token}});
 assert.equal(response.status(),200);const afterReplay=await h.snapshot(roomId);assert.equal(afterReplay.room.state,beforeNeg.room.state);assert.deepEqual(afterReplay.commands,beforeNeg.commands);assert.deepEqual(afterReplay.journal,beforeNeg.journal);checks.duplicateAfterRound=true;
 const stale={commandId:'stale-negative-'+Date.now(),expectedVersion:0,action:{kind:'game',action:{kind:'pass'}}};
 const rejected=await slots[0].context.request.post(h.url+'/api/rooms/'+roomId+'/commands',{data:stale,headers:{Authorization:'Bearer '+slots[0].session.token}});assert.equal(rejected.status(),409);
 const afterStale=await h.snapshot(roomId);assert.equal(afterStale.room.state,beforeNeg.room.state);assert.deepEqual(afterStale.commands,beforeNeg.commands);assert.deepEqual(afterStale.journal,beforeNeg.journal);checks.staleRejectedWithoutWrites=true;
 const reopened=await h.reopen(roomId);assert.deepEqual(reopened.after,reopened.before);checks.SQLiteReopenByteExact=true;
 for(let s=0;s<4;s++){await slots[s].page.reload();await slots[s].page.locator('.hg-lantern-table').waitFor({timeout:12000});await fresh(s);assert.equal(latest[s].you,'p'+s);}
 checks.fourSeatsRestoredAfterReopen=true;
 const p=slots[0].page;await p.getByRole('button',{name:'返回大厅 / 新建牌桌',exact:true}).click();await p.getByRole('button',{name:/回到牌桌/}).first().click();await p.locator('.hg-lantern-table').waitFor();
 const restored=await h.snapshot(roomId);assert.equal(restored.seats.length,4);assert.equal(restored.entries.length,4);checks.lobbyReturnRestoresOriginalSeat=true;
 await p.setViewportSize({width:390,height:844});await shot(0,'mobile-restored-room');await p.setViewportSize({width:1440,height:1000});
 const final=await h.snapshot(roomId);writeFileSync(folder+'/final-snapshot.json',JSON.stringify(final,null,2));
 checks.journalEntries=final.journal.length;checks.commandReceipts=final.commands.length;checks.seats=final.seats.length;
 for(let s=0;s<4;s++)await slots[s].context.storageState({path:folder+'/seat-'+s+'-storage.json'});
 checks.success=true;save();console.log(JSON.stringify({run,roomId,seed:9,checks}));
}catch(error){
 checks.success=false;checks.error=String(error.stack||error);save();
 for(let s=0;s<slots.length;s++)try{await slots[s].context.storageState({path:folder+'/failure-seat-'+s+'-storage.json'});await slots[s].page.screenshot({path:folder+'/failure-seat-'+s+'.png'});writeFileSync(folder+'/failure-seat-'+s+'.txt',await slots[s].page.locator('body').innerText());}catch{}
 if(roomId)writeFileSync(folder+'/failure-snapshot.json',JSON.stringify(await h.snapshot(roomId),null,2));
 console.error(error);process.exitCode=1;
}finally{await browser.close();await h.close();}
