// Production UI -> listening local HTTP -> twelve exact WASM cores -> persistent D1.
// Only the initial Native layout and trusted local clock are deterministic inputs.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdir,mkdtemp,readFile,readdir,writeFile} from 'node:fs/promises';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {loadFixture,sha256} from '../xq23-oracle.mjs';
import {kernel as abi,moduleBytes} from '../../rust-game-wasm/tests/current88-kernel.mjs';
import {RoomStore} from '../../sites/src/store.mjs';
import {digest} from '../../sites/src/service.mjs';
import {assertPublicView,assertSessionJournal} from '../../sites/test/assert-session-journal.mjs';
const repo=resolve(dirname(fileURLToPath(import.meta.url)),'../..'),sites=join(repo,'sites');
const requireWeb=createRequire(join(repo,'web/package.json')),requireSites=createRequire(join(sites,'package.json'));
const {chromium}=requireWeb('playwright'),{expect}=requireWeb('playwright/test');
const {Miniflare,convertV4MiniflareOptions}=await import(pathToFileURL(requireSites.resolve('miniflare')).href);
assert(process.argv[2],'Provide a new evidence directory');const output=resolve(process.argv[2]);await mkdir(output);
const fixture=loadFixture();assert.equal(sha256(moduleBytes),fixture.wasmSha256);
const scenario=fixture.scenarios.find(s=>s.name===(process.env.XQ23_BROWSER_SCENARIO||'actor-0'));assert(scenario);assert(['actor-0','actor-3','ordinary-deploy'].includes(scenario.name),'Browser driver scope is explicit; response composition is checked separately');
const initial=scenario.initial.views[0],id=initial.roomId,worker=process.env.HEGEMONY_CURRENT_WORKER_DIR;assert(worker);
const client=resolve(process.env.XQ23_CLIENT_DIRECTORY||join(sites,'dist/client'));
const wasmModules=await Promise.all((await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(async file=>({file,sha256:sha256(await readFile(join(worker,file)))})));
const previous=await Promise.all([77,78,79,80,81,82,83,84,85,86,87].map(async n=>JSON.parse(await readFile(join(repo,`rust-game-wasm/frozen/abi${n}/identity.json`))).wasmSha256));
assert.deepEqual(wasmModules.map(m=>m.sha256).sort(),[fixture.wasmSha256,...previous].sort());
const persist=await mkdtemp(join(process.env.TMPDIR,'xq23-real-http-'));
const tokens=Array.from({length:4},(_,s)=>String(s+1).repeat(64)); // Synthetic local seats only.
const options={name:'xq23-real-http',rootPath:sites,modulesRoot:worker,host:'127.0.0.1',port:0,cf:false,compatibilityDate:'2026-10-02',resourcePersistencePath:persist,
 modules:[{type:'ESModule',path:join(worker,'worker.js')},...wasmModules.map(m=>({type:'CompiledWasm',path:join(worker,m.file)}))],
 d1Databases:{DB:'xq23-real-http'},bindings:{LOCAL_TEST_CLOCK_MS:'0'},
 assets:{directory:client,binding:'ASSETS',run_worker_first:['/api/*'],routerConfig:{has_user_worker:true},assetConfig:{not_found_handling:'single-page-application'}}};
const report={passed:false,onlineGameplay:false,siteChanged:false,mainMerged:false,syntheticInitialLayout:true,
 scope:'Native88 initial layout; actual production GameApp, listening local HTTP/workerd/twelve-WASM/D1 and four independent browser contexts.',
 driverSha256:sha256(await readFile(fileURLToPath(import.meta.url))),scenario:scenario.name,engine:fixture.catalog.engineVersion,wasmSha256:fixture.wasmSha256,wasmModules,persist,
 productionWorkerSha256:sha256(await readFile(join(sites,'dist/server/index.js'))),workerSha256:sha256(await readFile(join(worker,'worker.js'))),
 clientIndexSha256:sha256(await readFile(join(client,'index.html'))),commands:[],clientPosts:[0,0,0,0],pageErrors:[],screenshots:[],reopened:0};
let mf,db,browser,origin;const pages=[],receipts=[];const store=()=>new RoomStore(db);
const snapshot=async()=>({room:await store().room(id),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results,
 journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions(options));const url=await mf.ready;origin=url.origin;options.port=Number(url.port);db=await mf.getD1Database('DB',options.name);};
const capture=async(page,name)=>{await page.screenshot({path:join(output,name+'.png'),fullPage:true});report.screenshots.push(name+'.png');};
const hit=async control=>{await control.scrollIntoViewIfNeeded();assert(await control.evaluate(el=>{const r=el.getBoundingClientRect(),top=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return top===el||el.contains(top);}),'Visible control receives its ordinary click');};
try{
 await start();
 for(const f of (await readdir(join(sites,'drizzle'))).filter(f=>f.endsWith('.sql')))for(const sql of (await readFile(join(sites,'drizzle',f),'utf8')).split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'offline-native88-xq23',response:'{}'});
 await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
 for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
 browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});report.browser=browser.version();
 for(let seat=0;seat<4;seat++){
  const context=await browser.newContext({viewport:{width:1366,height:900}});
  await context.addInitScript(saved=>{localStorage.setItem('hegemony.session.v1',JSON.stringify(saved));localStorage.setItem('hegemony.seats.v1',JSON.stringify([saved]));localStorage.setItem('hegemony.screen.v1','table');localStorage.setItem('hegemony.autoPass.v1','false');},{roomId:id,inviteCode:initial.inviteCode,seat,token:tokens[seat]});
  // Install before the intended freeze point: installing and pausing at the
  // same instant races elapsed setup time. The application still starts at04:00.
  const page=await context.newPage();pages.push(page);await page.clock.install({time:new Date('2026-10-18T03:00:00Z')});await page.clock.pauseAt(new Date('2026-10-18T04:00:00Z'));
  page.on('pageerror',e=>report.pageErrors.push(String(e)));page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')&&r.method()==='POST')report.clientPosts[seat]++;});
  await page.goto(origin);await expect(page.locator('.hg-connection-online')).toBeVisible();
 }
 // Read the actual source before it becomes anonymous in the asset area.
 const readerSeat=scenario.name==='actor-3'?3:0;
 const sourceView=scenario.initial.views[readerSeat];
 const source=[...sourceView.hand,...sourceView.regions.flatMap(r=>r.characters)].find(c=>c.cardId==='XQ23');assert(source);
 const reader=pages[readerSeat],beforeRead=await snapshot(),readPosts=report.clientPosts[readerSeat];
 await reader.locator(`[data-card-instance="${source.instanceId}"]`).first().click();
 await reader.getByRole('button',{name:'放大阅读谨慎的办事员',exact:true}).click();
 const reading=reader.locator('.hg-reading');await expect(reading).toContainText('公开');await expect(reading).toContainText('撤回');await expect(reading).toContainText('拥有者手上');await expect(reading).toContainText('资产区');
 await expect(reading.getByLabel('当前有效图标：调查0，战斗0，势力1',{exact:true})).toBeVisible();
 await expect(reading.getByLabel('印刷图标：调查0，战斗0，势力1',{exact:true})).toBeVisible();
 await reading.getByRole('button',{name:'原始牌面',exact:true}).click();
 const scan=reading.getByRole('img',{name:'谨慎的办事员原始牌面',exact:true});await expect(scan).toHaveAttribute('src','/cards/XQ23.jpg');
 await expect(scan).toBeVisible();await expect.poll(()=>scan.evaluate(img=>img.complete&&img.naturalWidth>0)).toBe(true);
 await capture(reader,'xq23-original-print');await reader.getByRole('button',{name:'关闭放大阅读',exact:true}).click();
 assert.equal(report.clientPosts[readerSeat],readPosts);assert.deepEqual(await snapshot(),beforeRead);
 report.sourceRead={seat:readerSeat,cardId:'XQ23',currentIcons:source.icons,printedIcons:{investigation:0,combat:0,influence:1},scanLoaded:true,noCommandOrMutation:true};
 const fetchRoom=(seat,operation,command)=>pages[seat].evaluate(async({id,operation,command})=>{const saved=JSON.parse(localStorage.getItem('hegemony.session.v1'));const r=await fetch(`/api/rooms/${id}/${operation}`,{method:command?'POST':'GET',headers:{Authorization:'Bearer '+saved.token,...(command?{'Content-Type':'application/json'}:{})},...(command?{body:JSON.stringify(command)}:{})});return{status:r.status,body:await r.json()};},{id,operation,command});
 for(const[index,row]of scenario.steps.entries()){
  const page=pages[row.seat],session=row.command.action,action=session.action;report.activeStep={index,seat:row.seat,kind:session.kind,gameKind:action?.kind};
  assert.equal((await store().room(id)).state,row.state);
  if(row.expected.errorCode){const before=await snapshot(),r=await fetchRoom(row.seat,'commands',row.command);assert(r.status>=400);assert.equal(r.body.error,row.expected.errorCode);assert.deepEqual(await snapshot(),before);report.commands.push({index,via:'browser-fetch-expected-rejection',status:r.status});continue;}
  await page.reload();await expect(page.locator('.hg-connection-online')).toBeVisible();const current=JSON.parse(abi.view(row.state,row.seat));
  if(action?.kind==='choose'){
   const before=await snapshot();await mf.dispose();await start();report.reopened++;assert.deepEqual(await snapshot(),before);
   for(let seat=0;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator('.hg-connection-online')).toBeVisible();await expect(pages[seat].getByRole('dialog',{name:'待完成的选择'})).toHaveCount(seat===row.seat?1:0);}
   await capture(page,'restored-choice-'+index);
  }
  const pending=page.waitForResponse(r=>new URL(r.url()).pathname===`/api/rooms/${id}/commands`&&r.request().method()==='POST');void pending.catch(()=>{});const posts=report.clientPosts[row.seat];
  if(session.kind==='pauseRoom')await page.getByRole('button',{name:current.pendingChoice?'暂停并保存此桌':'暂停并保存',exact:true}).click();
  else if(session.kind==='resumeRoom')await page.getByRole('button',{name:'恢复对局',exact:true}).click();
  else if(session.kind==='passResponse')await page.getByRole('button',{name:'不连锁，让过',exact:true}).click();
  else if(session.kind==='game'&&action.kind==='choose'){
   const dialog=page.getByRole('dialog',{name:'待完成的选择'});for(const choice of action.selected)await dialog.locator(`[data-choice-option="${choice}"]`).click();
   const button=dialog.getByRole('button',{name:current.pendingChoice.kind==='trigger'&&!action.selected.length?'跳过此选择':'确认选择',exact:true});await hit(button);await button.dblclick({delay:30});
  }else if(session.kind==='game'&&['activate','deploy','reveal'].includes(action.kind)){
   const legal=current.legalActions.find(a=>a.kind===action.kind&&a.cardId===action.cardId&&(!action.abilityId||a.abilityId===action.abilityId)&&a.targetId===action.targetId&&a.region===action.region&&JSON.stringify(a.costSelected)===JSON.stringify(action.costSelected));assert(legal,'Unique exact Native legal UI action');
   await page.locator(`[data-card-instance="${action.cardId}"]`).first().click();
   if(action.targetId||action.region!==undefined){
    const groups=page.locator('button[data-object-action]'),payloads=await groups.evaluateAll(nodes=>nodes.map(n=>JSON.parse(n.dataset.objectAction)[0]));
    const match=payloads.map((p,i)=>({p,i})).filter(({p})=>p.kind===action.kind&&p.cardId===action.cardId&&(!action.abilityId||p.abilityId===action.abilityId)&&JSON.stringify(p.costSelected)===JSON.stringify(action.costSelected));assert.equal(match.length,1);
    await groups.nth(match[0].i).click();
    if(action.region!==undefined)await page.locator(`button.hg-region-center[data-region-index="${action.region}"]`).click();
    if(action.targetId)await page.locator(/^p[0-3]$/.test(action.targetId)?`[data-player-target="${action.targetId}"]`:`[data-card-instance="${action.targetId}"]`).first().click();
   }
   const controls=page.locator('button[data-action-id]'),ids=await controls.evaluateAll(nodes=>nodes.map(n=>n.dataset.actionId)),position=ids.indexOf(legal.id);assert(position>=0);await hit(controls.nth(position));await controls.nth(position).dblclick({delay:20});
  }else throw Error('Unhandled actual UI action '+JSON.stringify(session));
  const http=await pending;assert.equal(http.status(),200);const sent=http.request().postDataJSON(),receipt=await http.json();
  const comparable=structuredClone(sent.action);
  if(action&&!action.abilityId&&comparable.action?.abilityId){
   const matches=current.legalActions.filter(a=>a.kind===action.kind&&a.cardId===action.cardId&&a.targetId===action.targetId&&a.region===action.region);
   assert.equal(new Set(matches.map(a=>a.abilityId)).size,1);assert.equal(matches[0].abilityId,comparable.action.abilityId);delete comparable.action.abilityId;
  }
  assert.deepEqual(comparable,session);assert.equal(sent.expectedVersion,row.command.expectedVersion);
  const expected=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(sent),'0'));assert.equal(expected.state,row.expected.state);assertPublicView(receipt,expected.view);assert.equal((await store().room(id)).state,expected.state);
  const entry=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(entry.entry,expected.journal);
  for(let seat=0;seat<4;seat++){const r=await fetchRoom(seat,'state');assert.equal(r.status,200);assertPublicView(r.body,row.views[seat]);}
  if(action&&['choose','activate','deploy','reveal'].includes(action.kind))assert.equal(report.clientPosts[row.seat]-posts,1,'Ordinary double click submits exactly once');
  receipts.push({seat:row.seat,command:sent,receipt});report.commands.push({index,seat:row.seat,via:'production-ui',kind:session.kind,gameKind:action?.kind,version:expected.version,stateSha256:sha256(expected.state)});
  if((session.kind==='game'&&['activate','deploy'].includes(action?.kind))||session.kind==='pauseRoom'){
   const before=await snapshot();await mf.dispose();await start();report.reopened++;assert.deepEqual(await snapshot(),before);
   for(let seat=0;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator('.hg-connection-online')).toBeVisible();const restored=await fetchRoom(seat,'state');assert.equal(restored.status,200);assertPublicView(restored.body,row.views[seat]);}
   await capture(pages[row.seat],'restored-paid-or-paused-'+index);
  }
 }
 const saved=await snapshot();await mf.dispose();await start();report.reopened++;assert.deepEqual(await snapshot(),saved);
 for(const r of receipts){const retry=await fetchRoom(r.seat,'commands',r.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,r.receipt);assert.deepEqual(await snapshot(),saved);}report.originalReceiptsAfterReopen=receipts.length;
 report.assetReads=[];
 for(let seat=0;seat<4;seat++){
  const view=scenario.steps.at(-1).views[seat],original=new Set(scenario.initial.views[seat].assets.map(c=>c.instanceId));
  const converted=view.assets.filter(c=>!original.has(c.instanceId));
  const page=pages[seat];await page.reload();await expect(page.locator('.hg-connection-online')).toBeVisible();
  for(const asset of converted){
   assert.equal(asset.kind,'asset');assert.equal(asset.name,'资产');assert.equal(asset.exhausted,true);assert.equal(asset.cardId,undefined);assert.equal(asset.text,undefined);assert.equal(asset.icons,undefined);
   const before=await snapshot(),posts=report.clientPosts[seat];
   await page.locator(`[data-card-instance="${asset.instanceId}"]`).first().click();await page.getByRole('button',{name:'放大阅读资产',exact:true}).click();
   const reading=page.locator('.hg-reading');await expect(reading.getByRole('button',{name:'原始牌面',exact:true})).toHaveCount(0);await expect(reading).not.toContainText('谨慎的办事员');
   await capture(page,'anonymous-asset-seat-'+seat);await page.getByRole('button',{name:'关闭放大阅读',exact:true}).click();
   assert.equal(report.clientPosts[seat],posts);assert.deepEqual(await snapshot(),before);
   report.assetReads.push({seat,instanceId:asset.instanceId,exhausted:true,originalIdentityHidden:true,noOriginalScan:true,noCommandOrMutation:true});
  }
 }
 if(scenario.name.startsWith('actor-'))assert.equal(report.assetReads.length,4);else assert.equal(report.assetReads.length,0);
 await capture(pages[0],'final');assert.deepEqual(report.pageErrors,[]);report.passed=true;
}catch(error){report.failure={name:error.name,message:error.message,stack:error.stack};process.exitCode=1;if(browser)for(const[seat,page]of pages.entries())try{await capture(page,'failure-'+seat);}catch{}}
finally{if(browser)await browser.close();if(mf)await mf.dispose();await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({passed:report.passed,scenario:scenario.name,commands:report.commands.length,reopened:report.reopened,failure:report.failure,output}));}
