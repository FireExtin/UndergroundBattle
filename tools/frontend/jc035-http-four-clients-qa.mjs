// Local production UI -> HTTP/workerd/one current WASM module -> persistent synthetic D1.
// Start from an unchanged Native93 pending-choice state; never online gameplay.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdir,mkdtemp,readFile,readdir,writeFile} from 'node:fs/promises';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {loadFixture,sha256} from '../jc035-oracle.mjs';
import {kernel,moduleBytes} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {RoomStore} from '../../sites/src/store.mjs';
import {digest} from '../../sites/src/service.mjs';
import {assertPublicView,assertSessionJournal} from '../../sites/test/assert-session-journal.mjs';
const repo=resolve(dirname(fileURLToPath(import.meta.url)),'../..'),sites=join(repo,'sites');
const requireWeb=createRequire(join(repo,'web/package.json')),requireSites=createRequire(join(sites,'package.json'));
const{chromium}=requireWeb('playwright'),{expect}=requireWeb('playwright/test');
const{Miniflare,convertV4MiniflareOptions}=await import(pathToFileURL(requireSites.resolve('miniflare')).href);
assert(process.argv[2],'Provide a NEW evidence directory');const output=resolve(process.argv[2]);await mkdir(output);
const f=loadFixture();assert.equal(sha256(moduleBytes),f.wasmSha256);
const original=f.scenarios.find(s=>s.name==='same-controller-order');
const offset=original.steps.findIndex(r=>r.command.action.action?.kind==='choose');assert(offset>0);
const scenario={initial:{state:original.steps[offset].state,views:original.steps[offset-1].views},steps:original.steps.slice(offset)};
const initial=scenario.initial.views[0],id=initial.roomId,worker=dirname(process.env.CURRENT_CLOCK_WORKER||join(repo,'sites/dist/current-clock/worker.js'));
const client=join(sites,'dist/client'),viewport={width:1366,height:768};
const modules=await Promise.all((await readdir(worker)).filter(n=>n.endsWith('.wasm')).map(async file=>({file,sha256:sha256(await readFile(join(worker,file)))})));
assert.deepEqual(modules.map(m=>m.sha256).sort(),[f.wasmSha256]);
const persist=await mkdtemp(join(process.env.TMPDIR,'jc035-browser-')),tokens=Array.from({length:4},(_,s)=>String(s+1).repeat(64));
const options={name:'jc035-browser',rootPath:sites,modulesRoot:worker,host:'127.0.0.1',port:0,cf:false,compatibilityDate:'2026-10-02',resourcePersistencePath:persist,
 modules:[{type:'ESModule',path:join(worker,'worker.js')},...modules.map(m=>({type:'CompiledWasm',path:join(worker,m.file)}))],
 d1Databases:{DB:'jc035-browser'},bindings:{LOCAL_TEST_CLOCK_MS:'0'},assets:{directory:client,binding:'ASSETS',run_worker_first:['/api/*'],routerConfig:{has_user_worker:true},assetConfig:{not_found_handling:'single-page-application'}}};
const report={passed:false,onlineGameplay:false,syntheticNativeInitialState:true,siteChanged:false,mainMerged:false,
 scope:'Actual local production UI, listening HTTP/workerd/one current WASM module/D1 and four browser contexts; three ordered JC035 declarations.',
 nativeInitialStateSha256:sha256(scenario.initial.state),nativeRawSha256:f.raw.sha256,nativeInputs:141,wasmSha256:f.wasmSha256,modules,viewport,persist,
 driverSha256:sha256(await readFile(fileURLToPath(import.meta.url))),workerSha256:sha256(await readFile(join(worker,'worker.js'))),productionWorkerSha256:sha256(await readFile(join(sites,'dist/server/index.js'))),
 clientIndexSha256:sha256(await readFile(join(client,'index.html'))),commands:[],screenshots:[],posts:[0,0,0,0],pageErrors:[],reopened:0};
let mf,db,browser,origin;const pages=[],receipts=[],store=()=>new RoomStore(db);
const snapshot=async()=>({room:await store().room(id),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results,journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions(options));const url=await mf.ready;origin=url.origin;options.port=Number(url.port);db=await mf.getD1Database('DB',options.name);};
const capture=async(page,name)=>{await page.screenshot({path:join(output,name+'.png'),fullPage:true});report.screenshots.push(name+'.png');};
const fetchRoom=(seat,command)=>pages[seat].evaluate(async({id,command})=>{const s=JSON.parse(localStorage.getItem('hegemony.session.v1'));const r=await fetch(`/api/rooms/${id}/${command?'commands':'state'}`,{method:command?'POST':'GET',headers:{Authorization:'Bearer '+s.token,...(command?{'Content-Type':'application/json'}:{})},...(command?{body:JSON.stringify(command)}:{})});return{status:r.status,body:await r.json()};},{id,command});
const reloadAll=async views=>{for(let seat=0;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator('.hg-connection-online')).toBeVisible();const r=await fetchRoom(seat);assert.equal(r.status,200);assertPublicView(r.body,views[seat]);await expect(pages[seat].getByRole('dialog',{name:'待完成的选择'})).toHaveCount(views[seat].pendingChoice?1:0);}};
try{
 await start();for(const file of(await readdir(join(sites,'drizzle'))).filter(f=>f.endsWith('.sql')))for(const sql of(await readFile(join(sites,'drizzle',file),'utf8')).split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
 await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'local-jc035-native-pending',response:'{}'});
 await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
 browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});report.browser=browser.version();
 for(let seat=0;seat<4;seat++){
  const context=await browser.newContext({viewport});await context.addInitScript(s=>{localStorage.setItem('hegemony.session.v1',JSON.stringify(s));localStorage.setItem('hegemony.seats.v1',JSON.stringify([s]));localStorage.setItem('hegemony.screen.v1','table');localStorage.setItem('hegemony.autoPass.v1','false');},{roomId:id,inviteCode:initial.inviteCode,seat,token:tokens[seat]});
  const page=await context.newPage();pages.push(page);await page.clock.install({time:new Date('2026-10-18T03:00:00Z')});await page.clock.pauseAt(new Date('2026-10-18T04:00:00Z'));
  page.on('pageerror',e=>report.pageErrors.push(String(e)));page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')&&r.method()==='POST')report.posts[seat]++;});await page.goto(origin);await expect(page.locator('.hg-connection-online')).toBeVisible();
 }
 await reloadAll(scenario.initial.views);const beforeRead=await snapshot();
 const reader=pages[0];await reader.getByRole('button',{name:'放大阅读传媒魔咒',exact:true}).first().click();
 const reading=reader.locator('.hg-reading');await expect(reading).toContainText('准备步骤触发');await expect(reading).toContainText('少于或等于三张');await expect(reading).toContainText('媒体');
 await reading.getByRole('button',{name:'原始牌面',exact:true}).click();const scan=reading.getByRole('img',{name:'传媒魔咒原始牌面',exact:true});await expect(scan).toHaveAttribute('src','/cards/JC035.jpg');await expect.poll(()=>scan.evaluate(i=>i.complete&&i.naturalWidth>0)).toBe(true);await capture(reader,'original-jc035');await reader.getByRole('button',{name:'关闭放大阅读',exact:true}).click();assert.deepEqual(await snapshot(),beforeRead);
 for(const[index,row]of scenario.steps.entries()){
  report.activeStep={index,seat:row.seat,action:row.command.action.kind};assert.equal((await store().room(id)).state,row.state);assert(!row.expected.errorCode);
  const page=pages[row.seat],session=row.command.action,action=session.action;await page.reload();await expect(page.locator('.hg-connection-online')).toBeVisible();
  if(action?.kind==='choose'){
   let dialog=page.getByRole('dialog',{name:'待完成的选择'});await expect(dialog).toContainText('传媒魔咒：选择下一项准备触发');await expect(dialog).toContainText('触发 1');
   const chosen=action.selected[0];assert(chosen);if(index===0){
    const other=dialog.locator(`[data-choice-option]:not([data-choice-option="${chosen}"])`).first();await other.click();
   }
   await dialog.locator(`[data-choice-option="${chosen}"]`).click();await expect(dialog.locator('[data-choice-option][aria-pressed="true"]')).toHaveCount(1);await expect(dialog).toContainText('下一项入栈：');
   if(index===0){const saved=await snapshot();await mf.dispose();await start();report.reopened++;await reloadAll(scenario.initial.views);assert.deepEqual(await snapshot(),saved);dialog=page.getByRole('dialog',{name:'待完成的选择'});await expect(dialog.locator(`[data-choice-option="${chosen}"]`)).toHaveAttribute('aria-pressed','true');await capture(page,'restored-pending-draft');}
  }
  const response=page.waitForResponse(r=>new URL(r.url()).pathname===`/api/rooms/${id}/commands`&&r.request().method()==='POST');void response.catch(()=>{});const posts=report.posts[row.seat];
  if(action?.kind==='choose')await page.getByRole('dialog',{name:'待完成的选择'}).getByRole('button',{name:'确认选择',exact:true}).dblclick({delay:30});
  else{assert.equal(session.kind,'passResponse');await page.getByRole('button',{name:'不连锁，让过',exact:true}).click();}
  const http=await response;assert.equal(http.status(),200);const sent=http.request().postDataJSON(),receipt=await http.json();assert.deepEqual(sent.action,session);assert.equal(sent.expectedVersion,row.command.expectedVersion);
  const expected=JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify(sent),'0'));assert.equal(expected.state,row.expected.state);assertPublicView(receipt,expected.view);assert.equal((await store().room(id)).state,expected.state);
  const entry=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(entry.entry,expected.journal);assert.equal(report.posts[row.seat]-posts,1);
  for(let seat=0;seat<4;seat++){const r=await fetchRoom(seat);assert.equal(r.status,200);assertPublicView(r.body,row.views[seat]);}
  receipts.push({seat:row.seat,command:sent,receipt});report.commands.push({index,seat:row.seat,kind:session.kind,gameKind:action?.kind,stateSha256:sha256(expected.state)});
  if(index===0){const saved=await snapshot();await mf.dispose();await start();report.reopened++;await reloadAll(row.views);assert.deepEqual(await snapshot(),saved);await capture(page,'next-source-after-reopen');}
 }
 const saved=await snapshot();await mf.dispose();await start();report.reopened++;for(const r of receipts){const retry=await fetchRoom(r.seat,r.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,r.receipt);assert.deepEqual(await snapshot(),saved);}
 await reloadAll(scenario.steps.at(-1).views);await capture(pages[0],'resolved-three-regions');assert.deepEqual(report.pageErrors,[]);report.originalReceiptsAfterReopen=receipts.length;report.passed=true;
}catch(error){report.failure={name:error.name,message:error.message,stack:error.stack};process.exitCode=1;if(browser)for(const[seat,page]of pages.entries())try{await capture(page,'failure-'+seat);}catch{}}
finally{if(browser)await browser.close();if(mf)await mf.dispose();await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({passed:report.passed,commands:report.commands.length,reopened:report.reopened,failure:report.failure,output}));}
