// Four real browser contexts, production GameApp/assets, listening HTTP,
// workerd/WASM and persistent local D1. Only the initial layout and clock are
// deterministic test inputs; no API mocking, opaque-state rewrite or live Site.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadFixture, sha256 } from '../jc057-oracle.mjs';
import { kernel as abi, moduleBytes } from '../../rust-game-wasm/tests/current84-kernel.mjs';
import { RoomStore } from '../../sites/src/store.mjs';
import { digest } from '../../sites/src/service.mjs';
import { assertPublicView, assertSessionJournal } from '../../sites/test/assert-session-journal.mjs';
const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const sites = join(repo, 'sites'), requireWeb = createRequire(join(repo, 'web/package.json'));
const clientDirectory = resolve(process.env.JC057_CLIENT_DIRECTORY || join(sites, 'dist/client'));
const requireSite = createRequire(join(sites, 'package.json'));
const { Miniflare, convertV4MiniflareOptions } = await import(pathToFileURL(requireSite.resolve('miniflare')).href);
const { chromium } = requireWeb('playwright'), { expect } = requireWeb('playwright/test');
assert(process.argv[2], 'Provide a new evidence directory');
const output = resolve(process.argv[2]); await mkdir(output);
const fixture = loadFixture(); assert.equal(sha256(moduleBytes), fixture.wasmSha256);
const scenario=fixture.scenarios.find(s=>s.name===(process.env.JC057_BROWSER_SCENARIO||'reveal-p1'));assert(scenario);
const initial=scenario.initial.views[0],id=initial.roomId;
const tokens=Array.from({length:4},(_,s)=>String(s+1).repeat(64)); // Local synthetic seats only.
const worker=process.env.HEGEMONY_CURRENT_WORKER_DIR;assert(worker);
const wasmModules=await Promise.all((await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(async file=>({file,sha256:sha256(await readFile(join(worker,file)))})));
const previous=await Promise.all([77,78,79,80,81,82,83].map(async n=>JSON.parse(await readFile(join(repo,`rust-game-wasm/frozen/abi${n}/identity.json`))).wasmSha256));
assert.deepEqual(wasmModules.map(m=>m.sha256).sort(),[fixture.wasmSha256,...previous].sort());
const persist=await mkdtemp(join(process.env.TMPDIR,'jc057-real-http-'));
const options={name:'jc057-real-http',rootPath:sites,modulesRoot:worker,host:'127.0.0.1',port:0,cf:false,compatibilityDate:'2026-10-02',resourcePersistencePath:persist,
  modules:[{type:'ESModule',path:join(worker,'worker.js')},...(await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:join(worker,f)}))],
  d1Databases:{DB:'jc057-real-http'},bindings:{LOCAL_TEST_CLOCK_MS:'0'},
  assets:{directory:clientDirectory,binding:'ASSETS',run_worker_first:['/api/*'],routerConfig:{has_user_worker:true},assetConfig:{not_found_handling:'single-page-application'}}};
const report={passed:false,onlineGameplay:false,siteChanged:false,mainMerged:false,syntheticInitialLayout:true,scope:'Explicit Native84 initial layout; then real production GameApp/HTTP/workerd/eight-WASM/D1 commands and four browser contexts. Not online gameplay.',scenario:scenario.name,
  engine:fixture.catalog.engineVersion,wasmSha256:fixture.wasmSha256,wasmModules,persist,
  productionWorkerSha256:sha256(await readFile(join(sites,'dist/server/index.js'))),workerSha256:sha256(await readFile(join(worker,'worker.js'))),
  clientIndexSha256:sha256(await readFile(join(clientDirectory,'index.html'))),commands:[],privateReads:[],clientPosts:[0,0,0,0],pageErrors:[],screenshots:[],reopened:0};
let mf,db,browser,origin,pages=[];
const store=()=>new RoomStore(db);
const snapshot=async()=>({room:await store().room(id),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results,journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
const start=async()=>{mf=new Miniflare(convertV4MiniflareOptions(options));const url=await mf.ready;origin=url.origin;options.port=Number(url.port);db=await mf.getD1Database('DB',options.name);};
const capture=async(page,name)=>{await page.screenshot({path:join(output,name+'.png'),fullPage:true});report.screenshots.push(name+'.png');};
const hit=async control=>{await control.scrollIntoViewIfNeeded();assert(await control.evaluate(el=>{const r=el.getBoundingClientRect(),top=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return top===el||el.contains(top);}),'Visible control must receive its ordinary click');};
let heldReader=null,lastCommand,lastReceipt;
try {
  await start();
  for(const f of (await readdir(join(sites,'drizzle'))).filter(f=>f.endsWith('.sql')))for(const sql of (await readFile(join(sites,'drizzle',f),'utf8')).split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-offline-native84-jc057',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});report.browser=browser.version();
  for(let seat=0;seat<4;seat++) {
    const context=await browser.newContext({viewport:{width:1366,height:900}});
    await context.addInitScript(saved=>{localStorage.setItem('hegemony.session.v1',JSON.stringify(saved));localStorage.setItem('hegemony.seats.v1',JSON.stringify([saved]));localStorage.setItem('hegemony.screen.v1','table');localStorage.setItem('hegemony.autoPass.v1','false');},{roomId:id,inviteCode:initial.inviteCode,seat,token:tokens[seat]});
    const page=await context.newPage();pages.push(page);await page.clock.install({time:new Date('2026-10-18T01:00:00Z')});await page.clock.pauseAt(new Date('2026-10-18T01:00:00Z'));
    page.on('pageerror',e=>report.pageErrors.push(String(e)));page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')&&r.method()==='POST')report.clientPosts[seat]++;});
    await page.goto(origin);await expect(page.locator('.hg-connection-online')).toBeVisible();
  }
  const fetchRoom=(seat,operation,command)=>pages[seat].evaluate(async({id,operation,command})=>{const saved=JSON.parse(localStorage.getItem('hegemony.session.v1'));const r=await fetch(`/api/rooms/${id}/${operation}`,{method:command?'POST':'GET',headers:{Authorization:'Bearer '+saved.token,...(command?{'Content-Type':'application/json'}:{})},...(command?{body:JSON.stringify(command)}:{})});return {status:r.status,body:await r.json()};},{id,operation,command});
  const privateRead=async(seat,leaveOpen=false)=>{
    const page=pages[seat];await page.bringToFront();const before=await snapshot(),posts=report.clientPosts[seat],button=page.getByRole('button',{name:/^检视暗藏者 /}).first();
    await expect(button).toBeVisible();assert.equal(await button.locator('xpath=ancestor::section[@aria-label="监控员私密检视"]').evaluate(el=>getComputedStyle(el).position),'static');await hit(button);await button.click();
    const reading=page.locator('.hg-reading');await expect(reading).toContainText('终焉的诗寇蒂');await expect(reading).toContainText('该牌仍为暗藏者');
    assert.equal(report.clientPosts[seat],posts);assert.deepEqual(await snapshot(),before);
    await capture(page,`private-${scenario.name}-${report.privateReads.length}`);report.privateReads.push({seat,noCommandOrMutation:true,whilePendingChoice:!!JSON.parse(abi.view(before.room.state,seat)).pendingChoice});
    if(leaveOpen)heldReader=seat;else await page.getByRole('button',{name:'关闭放大阅读',exact:true}).click();
  };
  for(const [index,row]of scenario.steps.entries()) {
    report.activeStep={index,seat:row.seat,kind:row.command.action.kind,gameKind:row.command.action.action?.kind};
    assert.equal((await store().room(id)).state,row.state);const page=pages[row.seat],session=row.command.action,action=session.action;
    if(row.expected.errorCode){const saved=await snapshot(),r=await fetchRoom(row.seat,'commands',row.command);assert(r.status>=400);assert.equal(r.body.error,row.expected.errorCode);assert.deepEqual(await snapshot(),saved);report.commands.push({index,via:'browser-fetch-expected-rejection',status:r.status});continue;}
    if(heldReader===row.seat){await page.getByRole('button',{name:'关闭放大阅读',exact:true}).click();heldReader=null;}
    if(heldReader!==row.seat){await page.reload();await expect(page.locator('.hg-connection-online')).toBeVisible();}
    const current=JSON.parse(abi.view(row.state,row.seat));
    if(current.pendingChoice?.kind==='trigger'&&scenario.name==='reveal-p1')await privateRead(0);
    if(current.pendingChoice?.kind==='jc057-hand-inspect'&&!current.pause) {
      const dialog=page.getByRole('dialog',{name:'待完成的选择'});await expect(dialog).toBeVisible();
      const saved=await snapshot();await mf.dispose();await start();report.reopened++;assert.deepEqual(await snapshot(),saved);await page.reload();await expect(dialog).toBeVisible();
      for(let seat=1;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator('.hg-choice')).toHaveCount(0);}
      if(scenario.name==='empty')await expect(dialog).toContainText('该玩家当前没有手牌。');
      else {await expect(dialog.locator('.hg-choice-preview')).toContainText('威达的汀巴巴迪');const posts=report.clientPosts[0];await dialog.getByRole('button',{name:'放大阅读威达的汀巴巴迪',exact:true}).click();await expect(page.locator('.hg-reading')).toContainText('检视的手牌 · 仅你可见');await page.getByRole('button',{name:'关闭放大阅读'}).click();assert.equal(report.clientPosts[0],posts);await privateRead(0);}
      await capture(page,scenario.name+'-restored-hand-choice');
    }
    if(session.kind==='passResponse'&&scenario.name.startsWith('live-')&&heldReader===null) {
      const old=JSON.parse(abi.view(row.state,scenario.name==='live-control'?2:0));
      if(old.privateInspections?.length){const owner=scenario.name==='live-control'?2:0;if(owner!==row.seat){await pages[owner].reload();await privateRead(owner,true);}}
    }
    const responsePromise=page.waitForResponse(r=>new URL(r.url()).pathname===`/api/rooms/${id}/commands`&&r.request().method()==='POST');void responsePromise.catch(()=>{});
    const posts=report.clientPosts[row.seat];
    if(session.kind==='pauseRoom')await page.getByRole('button',{name:current.pendingChoice?'暂停并保存此桌':'暂停并保存',exact:true}).click();
    else if(session.kind==='resumeRoom')await page.getByRole('button',{name:'恢复对局',exact:true}).click();
    else if(session.kind==='passResponse')await page.getByRole('button',{name:'不连锁，让过',exact:true}).click();
    else if(session.kind==='game'&&action.kind==='choose') {
      const dialog=page.getByRole('dialog',{name:'待完成的选择'});await expect(dialog).toBeVisible();
      for(const id of action.selected)await dialog.locator(`[data-choice-option="${id}"]`).click();
      const button=dialog.getByRole('button',{name:current.pendingChoice.kind==='trigger'&&!action.selected.length?'跳过此选择':'确认选择',exact:true});await hit(button);await button.dblclick({delay:30});
    } else if(session.kind==='game'&&['reveal','play','deploy'].includes(action.kind)) {
      const legal=current.legalActions.find(a=>a.kind===action.kind&&a.cardId===action.cardId&&a.targetId===action.targetId&&a.option===action.option&&a.region===action.region);assert(legal,'Native legal action');
      await page.locator(`[data-card-instance="${action.cardId}"]`).first().click();
      if(action.targetId||action.region!==undefined){const groups=page.locator('button[data-object-action]');const matching=groups.filter({hasText:action.option==='hide'?'潜伏':action.option==='return'?'移回':undefined});await (await matching.count()===1?matching:groups.first()).click();
        if(action.region!==undefined)await page.locator(`button.hg-region-center[data-region-index="${action.region}"]`).click();
        if(action.targetId)await page.locator(`[data-card-instance="${action.targetId}"]`).first().click();}
      const control=page.locator('button[data-action-id]');const ids=await control.evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-action-id')));const pos=ids.indexOf(legal.id);assert(pos>=0,'Visible exact command control');await hit(control.nth(pos));await control.nth(pos).click();
    } else throw Error('Unhandled UI action '+JSON.stringify(session));
    const http=await responsePromise;assert.equal(http.status(),200);const sent=http.request().postDataJSON(),receipt=await http.json();
    // Native permits omitting the sole printed default ability. The real UI
    // names it explicitly; require a unique exact legal action, then still
    // compare the complete resulting opaque state and four views below.
    const comparable=structuredClone(sent.action);
    if(action&&!action.abilityId&&comparable.action?.abilityId){
      const matches=current.legalActions.filter(a=>a.kind===action.kind&&a.cardId===action.cardId&&a.targetId===action.targetId&&a.option===action.option&&a.region===action.region);
      assert.equal(new Set(matches.map(a=>a.abilityId)).size,1);assert.equal(matches[0].abilityId,comparable.action.abilityId);delete comparable.action.abilityId;
    }
    assert.deepEqual(comparable,session);assert.equal(sent.expectedVersion,row.command.expectedVersion);
    const expected=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify(sent),'0'));assert.equal(expected.state,row.expected.state);assertPublicView(receipt,expected.view);assert.equal((await store().room(id)).state,expected.state);
    const entry=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(entry.entry,expected.journal);
    for(let seat=0;seat<4;seat++){const r=await fetchRoom(seat,'state');assert.equal(r.status,200);assertPublicView(r.body,row.views[seat]);}
    if(action?.kind==='choose')assert.equal(report.clientPosts[row.seat]-posts,1,'Double click submits once');
    if(heldReader!==null&&!row.views[heldReader].privateInspections?.length){await pages[heldReader].bringToFront();await pages[heldReader].clock.runFor(2200);await expect(pages[heldReader].locator('.hg-reading')).toHaveCount(0);await expect(pages[heldReader].getByRole('button',{name:/^检视暗藏者 /})).toHaveCount(0);report.readerRevokedOnFreshView=true;heldReader=null;}
    report.commands.push({index,seat:row.seat,via:'production-ui',kind:session.kind,gameKind:action?.kind,version:expected.version,stateSha256:sha256(expected.state)});lastCommand={seat:row.seat,command:sent};lastReceipt=receipt;
  }
  const saved=await snapshot();await mf.dispose();await start();report.reopened++;assert.deepEqual(await snapshot(),saved);
  const retry=await fetchRoom(lastCommand.seat,'commands',lastCommand.command);assert.equal(retry.status,200);assert.deepEqual(retry.body,lastReceipt);assert.deepEqual(await snapshot(),saved);report.originalReceiptAfterReopen=true;
  for(let seat=0;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator('.hg-connection-online')).toBeVisible();}
  for(let seat=0;seat<4;seat++){const expected=scenario.steps.at(-1).views[seat].privateInspections||[];await expect(pages[seat].getByRole('button',{name:/^检视暗藏者 /})).toHaveCount(expected.reduce((n,g)=>n+g.cards.length,0));}
  if(scenario.name==='live-control')await privateRead(0);
  assert.deepEqual(report.pageErrors,[]);await capture(pages[0],scenario.name+'-final');report.passed=true;
} catch(error){report.failure={name:error.name,message:error.message,stack:error.stack};if(browser)for(const [seat,page]of pages.entries())try{await capture(page,'failure-'+seat);}catch{}process.exitCode=1;}
finally {if(browser)await browser.close();if(mf)await mf.dispose();await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({passed:report.passed,scenario:scenario.name,commands:report.commands.length,privateReads:report.privateReads.length,failure:report.failure,output}));}
