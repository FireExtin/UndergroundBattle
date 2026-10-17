// Real production GameApp, listening HTTP/workerd, all six exact WASM modules,
// and synthetic local D1. Client and server clocks are explicit test inputs.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadFixture, sha256 } from '../response-preview-oracle.mjs';
import { kernel as abi, moduleBytes } from '../../rust-game-wasm/tests/current82-kernel.mjs';
import { RoomStore } from '../../sites/src/store.mjs';
import { digest } from '../../sites/src/service.mjs';
import { assertPublicView, assertSessionJournal } from '../../sites/test/assert-session-journal.mjs';
const repo=resolve(dirname(fileURLToPath(import.meta.url)),'../..'),sites=join(repo,'sites');
const requireWeb=createRequire(join(repo,'web/package.json')),requireSite=createRequire(join(sites,'package.json'));
const {Miniflare,convertV4MiniflareOptions}=await import(pathToFileURL(requireSite.resolve('miniflare')).href);
const {chromium}=requireWeb('playwright'),{expect}=requireWeb('playwright/test');
assert(process.argv[2],'Provide a new evidence directory');const output=resolve(process.argv[2]);await mkdir(output);
const fixture=loadFixture();assert.equal(sha256(moduleBytes),fixture.wasmSha256);
const initial=fixture.scenarios.find(s=>s.name==='preview-then-real-reveal').initial;
const passReference=fixture.scenarios.find(s=>s.name==='preview-peer-pass-cancel').steps.find(s=>s.kind==='command');
const worker=process.env.HEGEMONY_CURRENT_WORKER_DIR||join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT||'/workspace/undergroundbattle-build','jc054-worker/server');
const client=join(sites,'dist/client'),id=initial.views[0].roomId;
const modules=await Promise.all((await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(async file=>({file,sha256:sha256(await readFile(join(worker,file)))})));
const oldHashes=await Promise.all([77,78,79,80,81].map(async n=>JSON.parse(await readFile(join(repo,`rust-game-wasm/frozen/abi${n}/identity.json`),'utf8')).wasmSha256));
assert.deepEqual(modules.map(m=>m.sha256).sort(),[fixture.wasmSha256,...oldHashes].sort());
const report={passed:false,scope:'Native82 initial layout; four real production clients at each viewport; local trusted server clock 0 and controlled browser timers; not online play',engine:fixture.catalog.engineVersion,
  wasmSha256:fixture.wasmSha256,fixtureSha256:sha256(await readFile(join(repo,'rust-game-wasm/tests/responsePreviewNative82.fixture.json.gz'))),
  workerSha256:sha256(await readFile(join(worker,'worker.js'))),productionWorkerSha256:sha256(await readFile(join(sites,'dist/server/index.js'))),
  clientIndexSha256:sha256(await readFile(join(client,'index.html'))),modules,viewports:[],pageErrors:[],screenshots:[]};
let browser,mf,pages=[];
try {
  browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});report.browser=browser.version();
  for(const viewport of [{width:1440,height:1000},{width:1366,height:768}]) {
    const label=`${viewport.width}x${viewport.height}`,run={viewport,posts:[0,0,0,0],requests:[0,0,0,0],commands:[],checks:[],geometry:[],hitChecks:[]};report.viewports.push(run);
    const persist=await mkdtemp(join(process.env.TMPDIR,'response-preview-browser-'));
    const options={name:'response-preview-browser',rootPath:sites,modulesRoot:worker,host:'127.0.0.1',port:0,cf:false,compatibilityDate:'2026-10-02',resourcePersistencePath:persist,
      modules:[{type:'ESModule',path:join(worker,'worker.js')},...(await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:join(worker,f)}))],
      d1Databases:{DB:'response-preview-browser'},bindings:{LOCAL_TEST_CLOCK_MS:'0'},
      assets:{directory:client,binding:'ASSETS',run_worker_first:['/api/*'],routerConfig:{has_user_worker:true},assetConfig:{not_found_handling:'single-page-application'}}};
    mf=new Miniflare(convertV4MiniflareOptions(options));const origin=(await mf.ready).origin,db=await mf.getD1Database('DB',options.name),store=new RoomStore(db);
    for(const file of (await readdir(join(sites,'drizzle'))).filter(f=>f.endsWith('.sql')))
      for(const sql of (await readFile(join(sites,'drizzle',file),'utf8')).split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
    const tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64)); // Synthetic local seats only.
    await store.create({id,invite:initial.views[0].inviteCode,state:initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'synthetic-local-preview-browser',response:'{}'});
    await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.views[0].version,id).run();
    for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
    const snapshot=async()=>({room:await store.room(id),commands:(await db.prepare('SELECT * FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results,
      journal:(await db.prepare('SELECT * FROM journal WHERE room_id=? ORDER BY version').bind(id).all()).results});
    const before=await snapshot();pages=[];
    for(let seat=0;seat<4;seat++) {
      const context=await browser.newContext({viewport});const page=await context.newPage();pages.push(page);
      await page.clock.install({time:new Date('2026-10-17T00:00:00Z')});await page.clock.pauseAt(new Date('2026-10-17T00:00:00Z'));
      await context.addInitScript(saved=>{localStorage.setItem('hegemony.session.v1',JSON.stringify(saved));localStorage.setItem('hegemony.seats.v1',JSON.stringify([saved]));localStorage.setItem('hegemony.screen.v1','table');localStorage.setItem('hegemony.autoPass.v1','false');},
        {roomId:id,inviteCode:initial.views[0].inviteCode,seat,token:tokens[seat]});
      page.on('pageerror',e=>report.pageErrors.push({viewport:label,seat,error:String(e)}));
      page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')){run.requests[seat]++;if(r.method()==='POST')run.posts[seat]++;}});
      await page.goto(origin);await expect(page.locator('.hg-connection-online')).toBeVisible();await expect(page.locator('.hg-response-window')).toBeVisible();
    }
    const own=initial.views[0].responseWindow.previewSources,hidden=own.find(s=>s.kind==='reveal').cardId,hand=own.find(s=>s.kind==='play').cardId;
    const tile=(seat,card)=>pages[seat].locator(`.hg-card[data-card-instance="${card}"]`);
    const dock=seat=>pages[seat].getByRole('region',{name:'当前窗口与候选行动'});
    const capture=async(name,seat=0)=>{const file=`${label}-${name}-seat${seat}.png`;await pages[seat].screenshot({path:join(output,file)});report.screenshots.push(file);};
    const hitCheck=async(locator,label)=>{
      await locator.scrollIntoViewIfNeeded();
      const hit=await locator.evaluate(element=>{const r=element.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2,top=document.elementFromPoint(x,y);
        return {rect:r.toJSON(),viewport:{width:innerWidth,height:innerHeight},centerVisible:x>=0&&x<innerWidth&&y>=0&&y<innerHeight,hit:!!top&&(element===top||element.contains(top))};});
      assert(hit.centerVisible&&hit.hit,label+' is reachable through ordinary scrolling and receives pointer hits');run.hitChecks.push({label,...hit});
    };
    const inspectGeometry=async seat=>{run.geometry.push(await pages[seat].evaluate(()=>({viewport:{width:innerWidth,height:innerHeight},documentWidth:document.documentElement.scrollWidth,
      dock:(()=>{const r=document.querySelector('.hg-action-availability').getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};})(),
      hintBadges:[...document.querySelectorAll('.hg-card-action-hint')].map(el=>({text:el.textContent,rect:el.getBoundingClientRect().toJSON()}))})));};
    await expect(tile(0,hidden)).toContainText('候选·快速');await expect(tile(0,hand)).toContainText('候选·行动阶段快速');await expect(dock(0)).toContainText('响应候选');
    await expect(tile(0,hidden)).toHaveAttribute('data-card-actionable','true');
    assert(initial.views[0].legalActions.every(a=>a.kind==='pass'),'Highlight is a source hint, never a submitted action');
    for(const seat of [1,2,3])await expect(tile(seat,hidden)).not.toContainText('候选');
    for(const seat of [2,3])await expect(pages[seat].locator('.hg-card-action-hint')).toHaveCount(0);
    await capture('pre-intent');await inspectGeometry(0);
    await hitCheck(tile(0,hand),'own hand preview');await expect(tile(0,hand).locator('.hg-card-action-hint')).toBeVisible();
    await capture('pre-intent-hand-visible');await inspectGeometry(0);
    await hitCheck(tile(0,hidden),'own hidden preview');
    await tile(0,hidden).click();await expect(pages[0].locator('[data-action-kind="reveal"]')).toHaveCount(0);
    assert.deepEqual(run.posts,[0,0,0,0]);assert.deepEqual(await snapshot(),before);run.checks.push('own hand/hidden highlighted; opponent source private; preview click produces no action or write');
    await pages[0].reload();await expect(tile(0,hidden)).toContainText('候选·快速');assert.deepEqual(run.posts,[0,0,0,0]);assert.deepEqual(await snapshot(),before);
    run.checks.push('reload restores pre-intent preview with unchanged state, deadline, assets, receipts and journal');
    // Expiry is a display-only local timer check. The trusted server test clock
    // stays 0, so this cannot silently persist a server Tick or extend a deadline.
    await pages[0].clock.runFor(5001);await expect(tile(0,hidden)).not.toContainText('候选');await expect(dock(0)).toContainText('响应决定时间已到');
    await expect(pages[0].getByRole('button',{name:'连锁',exact:true})).toBeDisabled();assert.deepEqual(run.posts,[0,0,0,0]);assert.deepEqual(await snapshot(),before);
    await capture('client-deadline');run.checks.push('client deadline hides hints/disables Begin with zero POST and no state or server deadline changes');
    await pages[0].reload();await expect(tile(0,hidden)).toContainText('候选·快速');
    const command=async(seat,kind,control,reference)=>{
      const state=(await store.room(id)).state,posts=run.posts[seat];
      await hitCheck(control,kind+' control');
      const pending=pages[seat].waitForResponse(r=>new URL(r.url()).pathname===`/api/rooms/${id}/commands`&&r.request().method()==='POST');
      // Two synchronous real DOM clicks test the command lock before React has
      // committed the busy state; the original server receipt is also retried.
      await control.evaluate(button=>{button.click();button.click();});const http=await pending;
      assert.equal(http.status(),200);const body=http.request().postDataJSON(),response=await http.json();assert.equal(body.action.kind,kind);
      const expected=JSON.parse(abi.applyRoom(state,seat,JSON.stringify(body),'0'));assert.equal(expected.outcome,'accepted');assertPublicView(response,expected.view);
      assert.equal((await store.room(id)).state,expected.state);if(reference){assert.equal(expected.state,reference.expected.state);assertPublicView(response,reference.expected.view);}
      const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(journal.entry,expected.journal);
      await expect.poll(()=>run.posts[seat]).toBe(posts+1);run.commands.push({seat,kind,action:body.action,version:expected.version,stateSha256:sha256(expected.state),posts:run.posts[seat]-posts,reference:reference?'exact Native82 plus same-body WASM':'same-body fresh82 WASM'});
      return {seat,body,response,state:expected.state};
    };
    await command(1,'passResponse',pages[1].getByRole('button',{name:'不连锁，让过',exact:true}),passReference);
    await expect(pages[1].locator('.hg-response-window')).toHaveAttribute('data-response-state','passed');run.checks.push('manual Pass submits original payload exactly once, matching Native82 and D1 journal');
    await pages[0].reload();await expect(tile(0,hidden)).toContainText('候选·快速');
    const begun=await command(0,'beginResponse',pages[0].getByRole('button',{name:'连锁',exact:true}));
    await expect(pages[0].locator('.hg-response-window')).toHaveAttribute('data-response-state','composing');
    assert.equal(JSON.parse(begun.state).pacing.window.members['0'].intentId,begun.body.action.intentId);
    await expect(tile(0,hidden)).toContainText('候选·快速');await expect(dock(0)).not.toContainText('先选择“连锁”');await capture('composing');
    run.checks.push('manual Begin creates one real intent; source hints now use legalActions, original composing controls remain');
    await pages[0].reload();await expect(pages[0].locator('.hg-response-window')).toHaveAttribute('data-response-state','composing');
    assert.equal((await store.room(id)).state,begun.state);assert.equal(run.posts[0],1);run.checks.push('reload preserves the real composing intent without reposting Begin');
    await tile(0,hidden).click();const reveal=pages[0].locator('[data-action-kind="reveal"]');await expect(reveal).toBeEnabled();
    const submitted=await command(0,'submitResponse',reveal);
    assert.equal(submitted.body.action.intentId,begun.body.action.intentId);assert.deepEqual(submitted.body.action.action,{kind:'reveal',cardId:hidden});
    assert.equal(JSON.parse(submitted.state).game.players[0].assets.filter(c=>c.exhausted).length,4);run.checks.push('real reveal pays four assets only after explicit Begin and Submit; repeated click submits once');
    const saved=await snapshot();const retry=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/commands`,{method:'POST',headers:{Authorization:'Bearer '+tokens[0],'Content-Type':'application/json'},body:JSON.stringify(submitted.body)});
    assert.equal(retry.status,200);assert.deepEqual(await retry.json(),submitted.response);assert.deepEqual(await snapshot(),saved);run.checks.push('duplicate original command returns exact receipt with no extra state/journal/command write');
    await capture('submitted');await inspectGeometry(0);for(const geo of run.geometry)assert(geo.documentWidth<=viewport.width,'No page-level horizontal overflow');
    for(const page of pages)await page.context().close();pages=[];await mf.dispose();mf=null;
  }
  assert.deepEqual(report.pageErrors,[]);report.passed=true;report.onlineGameplay=false;report.siteOrMainChanged=false;
}catch(error){report.failure={name:error.name,message:error.message,stack:error.stack};process.exitCode=1;for(const [seat,page]of pages.entries())try{await page.screenshot({path:join(output,`failure-seat${seat}.png`)});}catch{}}
finally {if(browser)await browser.close();if(mf)await mf.dispose();await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({passed:report.passed,viewports:report.viewports.map(v=>({viewport:v.viewport,posts:v.posts,checks:v.checks.length})),failure:report.failure,output}));}
