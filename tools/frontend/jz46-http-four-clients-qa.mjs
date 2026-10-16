// Two real browser contexts, production GameApp/assets, listening HTTP,
// workerd/WASM and persistent local D1. Only the initial layout and clock are
// deterministic test inputs; no API mocking, opaque-state rewrite or live Site.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadFixture, sha256 } from '../jz46-oracle.mjs';
import { kernel as abi, moduleBytes } from '../../rust-game-wasm/tests/current79-kernel.mjs';
import { RoomStore } from '../../sites/src/store.mjs';
import { digest } from '../../sites/src/service.mjs';
import { assertPublicView, assertSessionJournal } from '../../sites/test/assert-session-journal.mjs';
const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const sites = join(repo, 'sites'), requireWeb = createRequire(join(repo, 'web/package.json'));
const requireSite = createRequire(join(sites, 'package.json'));
const { Miniflare, convertV4MiniflareOptions } = await import(pathToFileURL(requireSite.resolve('miniflare')).href);
const { chromium } = requireWeb('playwright'), { expect } = requireWeb('playwright/test');
assert(process.argv[2], 'Provide a new evidence directory');
const output = resolve(process.argv[2]); await mkdir(output);
const fixture = loadFixture(); assert.equal(sha256(moduleBytes), fixture.wasmSha256);
const scenario = fixture.scenarios.find(row => row.name === 'pause-inspection-resume'); assert(scenario);
const initial = scenario.initial.views[0], id = initial.roomId;
assert.equal(initial.players.length, 4);
const host = initial.hand.find(card => card.cardId === 'JZ46').instanceId;
const tokens = Array.from({length:4},(_,s)=>String(s+1).repeat(64)); // Local fixture seats, never real credentials.
const worker = join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'jz46-worker/server');
const retainedIdentity = JSON.parse(await readFile(join(repo, 'rust-game-wasm/frozen/abi77/identity.json'), 'utf8'));
const wasmModules = await Promise.all((await readdir(worker)).filter(file => file.endsWith('.wasm')).map(async file => {
  const bytes = await readFile(join(worker, file)); return { file, bytes: bytes.length, sha256: sha256(bytes) };
}));
assert.deepEqual(wasmModules.map(row => row.sha256).sort(), [fixture.wasmSha256, JSON.parse(await readFile(join(repo, 'rust-game-wasm/frozen/abi78/identity.json'), 'utf8')).wasmSha256, retainedIdentity.wasmSha256].sort());
const persist = await mkdtemp(join(process.env.TMPDIR, 'jz46-real-http-'));
const options = { name: 'jz46-real-http', rootPath: sites, modulesRoot: worker, host: '127.0.0.1', port: 0,
  cf: false, compatibilityDate: '2026-10-02', resourcePersistencePath: persist,
  modules: [{ type: 'ESModule', path: join(worker, 'worker.js') },
    ...(await readdir(worker)).filter(file => file.endsWith('.wasm')).map(file => ({ type: 'CompiledWasm', path: join(worker, file) }))],
  d1Databases: { DB: 'jz46-real-http' },
  assets: { directory: join(sites, 'dist/client'), binding: 'ASSETS', run_worker_first: ['/api/*'],
    routerConfig: { has_user_worker: true }, assetConfig: { not_found_handling: 'single-page-application' } },
};
const report = { passed: false, scope: 'Explicit prepared hidden target and funds, followed by real paid JZ46 play, action, four-client privacy, pause/restart and duplicate confirmation; local deterministic server clock, not online gameplay',
  engine: fixture.catalog.engineVersion, wasmSha256: fixture.wasmSha256, workerSha256: sha256(await readFile(join(worker, 'worker.js'))),
  wasmModules, productionWorkerSha256: sha256(await readFile(join(sites, 'dist/server/index.js'))),
  productionClientIndexSha256: sha256(await readFile(join(sites, 'dist/client/index.html'))),
  fixtureSha256: sha256(await readFile(join(repo, 'rust-game-wasm/tests/jz46Native79.fixture.json.gz'))),
  commands: [], clientRequests: [0, 0, 0, 0], clientPosts: [0, 0, 0, 0], pageErrors: [], screenshots: [], choiceLayoutChecks: [], reopened: 0 };
let mf, db, browser, pages = [], origin;
const start = async () => {
  mf = new Miniflare(convertV4MiniflareOptions(options));
  const url = await mf.ready; origin = url.origin; options.port = Number(url.port);
  db = await mf.getD1Database('DB', options.name);
};
try {
  await start();
  for(const file of (await readdir(join(sites,'drizzle'))).filter(f=>f.endsWith('.sql')))
    for(const sql of (await readFile(join(sites,'drizzle',file),'utf8')).split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  await store().create({id,invite:initial.inviteCode,state:scenario.initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-offline-jz46-initial-layout',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});report.browser=browser.version();
  for(let seat=0;seat<4;seat++){
    const context=await browser.newContext({viewport:{width:1366,height:900}});
    await context.addInitScript(saved=>{
      localStorage.setItem('hegemony.session.v1',JSON.stringify(saved));localStorage.setItem('hegemony.seats.v1',JSON.stringify([saved]));
      localStorage.setItem('hegemony.screen.v1','table');localStorage.setItem('hegemony.autoPass.v1','false');
    },{roomId:id,inviteCode:initial.inviteCode,seat,token:tokens[seat]});
    const page=await context.newPage();pages.push(page);
    page.on('pageerror',e=>report.pageErrors.push(String(e)));
    page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')){report.clientRequests[seat]++;if(r.method()==='POST')report.clientPosts[seat]++;}});
    await page.goto(origin);await expect(page.locator('.hg-app')).toBeVisible();
  }
  const fetchRoom=(seat,operation,command)=>pages[seat].evaluate(async({id,operation,command})=>{
    const saved=JSON.parse(localStorage.getItem('hegemony.session.v1'));
    const r=await fetch(`/api/rooms/${id}/${operation}`,{method:command?'POST':'GET',headers:{Authorization:'Bearer '+saved.token,...(command?{'Content-Type':'application/json'}:{})},...(command?{body:JSON.stringify(command)}:{})});
    return {status:r.status,body:await r.json()};
  },{id,operation,command});
  const capture=async label=>{for(let seat=0;seat<4;seat++){const name=`${label}-seat${seat}.png`;await pages[seat].screenshot({path:join(output,name)});report.screenshots.push(name);}};
  let lastCommand,lastReceipt;
  for(const [index,step] of scenario.steps.entries()){
    const page=pages[step.seat],kind=step.command.action.kind,gameAction=step.command.action.action;
    assert.equal((await store().room(id)).state,step.state);
    await page.reload();await expect(page.locator('.hg-app')).toBeVisible();
    report.activeStep={index,seat:step.seat,kind,gameKind:gameAction?.kind};
    await writeFile(join(output,'progress.json'),JSON.stringify({...report,commands:report.commands.length},null,2)+'\n');
    if(step.expected.errorCode){
      const r=await fetchRoom(step.seat,'commands',step.command);assert(r.status>=400);assert.equal(r.body.error,step.expected.errorCode);
      assert.equal((await store().room(id)).state,step.state);report.commands.push({index,seat:step.seat,via:'browser-fetch-expected-rejection',status:r.status});continue;
    }
    const responsePromise=page.waitForResponse(r=>new URL(r.url()).pathname===`/api/rooms/${id}/commands`&&r.request().method()==='POST'&&r.request().postDataJSON().expectedVersion===step.command.expectedVersion);
    void responsePromise.catch(()=>{});
    if(kind==='pauseRoom'){
      const dialog=page.getByRole('dialog',{name:'待完成的选择'});
      if(await dialog.count())await dialog.getByRole('button',{name:'暂停并保存此桌',exact:true}).click();
      else await page.getByRole('button',{name:'暂停并保存',exact:true}).click();
    }else if(kind==='resumeRoom')await page.getByRole('button',{name:'恢复对局',exact:true}).click();
    else if(kind==='passResponse')await page.getByRole('button',{name:'不连锁，让过',exact:true}).click();
    else if(kind==='game'&&gameAction.kind==='choose'){
      let dialog=page.getByRole('dialog',{name:'待完成的选择'});await expect(dialog).toBeVisible();
      await expect(dialog.getByRole('heading',{name:'你检视的暗藏者'})).toBeVisible();
      for(let s=1;s<4;s++)await expect(pages[s].getByRole('heading',{name:'你检视的暗藏者'})).toHaveCount(0);
      await capture('private-before-confirm');
      const before=await snapshot();await mf.dispose();await start();report.reopened++;
      assert.deepEqual(await snapshot(),before);await page.reload();
      dialog=page.getByRole('dialog',{name:'待完成的选择'});await expect(dialog.getByRole('heading',{name:'你检视的暗藏者'})).toBeVisible();
      report.privateInspectionRestoredAfterWorkerAndBrowserRestart=true;
      const posts=report.clientPosts[step.seat];
      await dialog.getByRole('button',{name:'确认选择',exact:true}).dblclick({delay:30});
      report.confirmPostsBefore=posts;
    }else if(kind==='game'&&['play','activate'].includes(gameAction.kind)){
      const legal=step.state&&JSON.parse(abi.view(step.state,step.seat)).legalActions.find(a=>
        a.kind===gameAction.kind&&a.cardId===gameAction.cardId&&a.abilityId===gameAction.abilityId&&a.targetId===gameAction.targetId&&a.region===gameAction.region);
      assert(legal,'Native legal action');
      await page.locator(`[data-card-instance="${gameAction.cardId}"]`).first().click();
      const buttons=page.locator('button[data-action-id]');
      const ids=await buttons.evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-action-id')));
      const match=ids.indexOf(legal.id);assert(match>=0,`No real UI control for ${gameAction.kind}`);
      await buttons.nth(match).click();
    }else throw Error('Unexpected UI step '+JSON.stringify(step.command.action));
    const http=await responsePromise;assert.equal(http.status(),200);const command=http.request().postDataJSON(),response=await http.json();
    assert.deepEqual(command.action,step.command.action);assert.equal(command.expectedVersion,step.command.expectedVersion);
    const expected=JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(command),'0'));assert.equal(expected.state,step.expected.state);
    assertPublicView(response,expected.view);assert.equal((await store().room(id)).state,expected.state);
    const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(journal.entry,expected.journal);
    for(let seat=0;seat<4;seat++){const r=await fetchRoom(seat,'state');assert.equal(r.status,200);assertPublicView(r.body,step.views[seat]);}
    if(gameAction?.kind==='choose'){
      assert.equal(report.clientPosts[step.seat]-report.confirmPostsBefore,1,'Double click must submit one UI command');
      report.doubleClickProducedOneSubmission=true;
    }
    report.commands.push({index,seat:step.seat,kind,gameKind:gameAction?.kind,via:'production-ui',status:200,version:expected.version,stateSha256:sha256(expected.state)});
    lastCommand={seat:step.seat,command};lastReceipt=response;
    if(kind==='pauseRoom'){const before=await snapshot();await mf.dispose();await start();report.reopened++;assert.deepEqual(await snapshot(),before);}
  }
  const saved=await snapshot();const duplicate=await fetchRoom(lastCommand.seat,'commands',lastCommand.command);
  assert.equal(duplicate.status,200);assert.deepEqual(duplicate.body,lastReceipt);assert.deepEqual(await snapshot(),saved);
  report.originalConfirmationReceiptAfterRestart=true;
  await capture('confirmed-once');assert.deepEqual(report.pageErrors,[]);
  report.passed=true;report.onlineGameplay=false;report.mainOrSiteChanged=false;
}catch(error){
  report.passed=false;report.failure={name:error.name,message:error.message,stack:error.stack};
  if(browser)for(const [seat,page]of pages.entries()){try{await page.screenshot({path:join(output,`failure-seat${seat}.png`)});}catch{}}
  process.exitCode=1;
}finally{
  if(browser)await browser.close();if(mf)await mf.dispose();
  await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({passed:report.passed,commands:report.commands.length,reopened:report.reopened,clientPosts:report.clientPosts,failure:report.failure,output}));
}
const store = () => new RoomStore(db);
const snapshot = async () => ({ room: await store().room(id), commands: (await db.prepare('SELECT command_id,intent_hash,response,version FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results });
