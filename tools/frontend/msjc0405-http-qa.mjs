// Production GameApp + actual local HTTP/workerd/WASM/D1. No live service writes.
// Entry flows use only visible UI. Ability flows disclose Native93 initial layouts.
import assert from 'node:assert/strict';
import {mkdir,mkdtemp,readFile,readdir,writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {dirname,join,resolve} from 'node:path';
import {loadFixture,sha256} from '../msjc0405-oracle.mjs';
import {kernel as abi,moduleBytes} from '../../rust-game-wasm/tests/current-kernel.mjs';
const moduleUrl=import.meta.url,repo=fileURLToPath(new URL('../../',moduleUrl));
const webRequire=createRequire(join(repo,'web/package.json')),siteRequire=createRequire(join(repo,'sites/package.json'));
const {chromium}=webRequire('playwright'),{expect}=webRequire('playwright/test');
const {Miniflare,convertV4MiniflareOptions}=await import(pathToFileURL(siteRequire.resolve('miniflare')).href);
const {RoomStore}=await import('../../sites/src/store.mjs');
const {digest}=await import('../../sites/src/service.mjs');
const {assertPublicView,assertSessionJournal}=await import('../../sites/test/assert-session-journal.mjs');
const output=resolve(process.argv[2]||'/workspace/msjc0405-evidence-20261017/browser-first');
await mkdir(output); // Preserve previous browser evidence; use a fresh directory.
const sites=join(repo,'sites'),client=join(sites,'dist/client');
const worker=dirname(process.env.CURRENT_CLOCK_WORKER||join(repo,'sites/dist/current-clock/worker.js'));
const fixture=loadFixture();assert.equal(sha256(moduleBytes),fixture.wasmSha256);
const modules=await Promise.all((await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(async file=>({file,sha256:sha256(await readFile(join(worker,file)))})));
assert.deepEqual(modules.map(m=>m.sha256).sort(),[fixture.wasmSha256]);
const report={passed:false,engine:fixture.catalog.engineVersion,wasmSha256:fixture.wasmSha256,modules,
  workerSha256:sha256(await readFile(join(worker,'worker.js'))),productionWorkerSha256:sha256(await readFile(join(sites,'dist/server/index.js'))),
  clientIndexSha256:sha256(await readFile(join(client,'index.html'))),scope:'Local production client. Natural UI deck construction/entry/start; separately disclosed Native93 ability layouts. No online gameplay.',
  natural:[],abilities:[],pageErrors:[],screenshots:[]};
let browser,mf,pages=[];
const capture=async(page,name)=>{const file=name+'.png';await page.screenshot({path:join(output,file),fullPage:true});report.screenshots.push(file);};
async function startServer(name) {
  const persist=await mkdtemp(join(process.env.TMPDIR,'msjc0405-browser-'));
  const options={name,rootPath:sites,modulesRoot:worker,host:'127.0.0.1',port:0,cf:false,compatibilityDate:'2026-10-02',resourcePersistencePath:persist,
    modules:[{type:'ESModule',path:join(worker,'worker.js')},...(await readdir(worker)).filter(f=>f.endsWith('.wasm')).map(f=>({type:'CompiledWasm',path:join(worker,f)}))],
    d1Databases:{DB:name},bindings:{LOCAL_TEST_CLOCK_MS:'0'},assets:{directory:client,binding:'ASSETS',run_worker_first:['/api/*'],routerConfig:{has_user_worker:true},assetConfig:{not_found_handling:'single-page-application'}}};
  mf=new Miniflare(convertV4MiniflareOptions(options));const origin=(await mf.ready).origin,db=await mf.getD1Database('DB',name);
  for(const file of (await readdir(join(sites,'drizzle'))).filter(f=>f.endsWith('.sql')))
    for(const sql of (await readFile(join(sites,'drizzle',file),'utf8')).split('--> statement-breakpoint').filter(s=>s.trim()))await db.prepare(sql).run();
  return {origin,db,store:new RoomStore(db),persist};
}
async function hit(control) {
  await control.scrollIntoViewIfNeeded();const box=await control.boundingBox();assert(box&&box.width>0&&box.height>0);
  assert(await control.evaluate(el=>{const r=el.getBoundingClientRect(),top=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return top===el||el.contains(top);}), 'Control must be the actual visible hit target');
}
async function context(viewport,saved) {
  const context=await browser.newContext({viewport}),page=await context.newPage();pages.push(page);
  page.on('pageerror',e=>report.pageErrors.push(String(e)));
  if(saved) {
    await page.clock.install({time:new Date('2026-10-17T00:00:00Z')});await page.clock.pauseAt(new Date('2026-10-17T00:00:00Z'));
    await context.addInitScript(s=>{localStorage.setItem('hegemony.session.v1',JSON.stringify(s));localStorage.setItem('hegemony.seats.v1',JSON.stringify([s]));localStorage.setItem('hegemony.screen.v1','table');},saved);
  }
  return page;
}
const waitPost=(page,path)=>page.waitForResponse(r=>new URL(r.url()).pathname===path&&r.request().method()==='POST');
async function natural(viewport) {
  const {origin,store}=await startServer('msjc0405-natural-'+viewport.height),run={viewport,checks:[],decks:[]};report.natural.push(run);pages=[];
  const selected=[];
  for(const [index,societyId]of ['MSJC04','MSJC05'].entries()) {
    const society=fixture.catalog.societies.find(c=>c.id===societyId),page=await context(viewport);
    let posts=0;page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')&&r.method()==='POST')posts++;});
    await page.goto(origin+'/#play');await expect(page.getByRole('combobox',{name:'入席秘社（可选）'})).toBeEnabled();
    await page.getByRole('combobox',{name:'入席秘社（可选）'}).selectOption(societyId);
    await expect(page.getByRole('button',{name:'创建牌桌 →'})).toBeDisabled();
    await expect(page.getByRole('alert')).toContainText('25');
    await page.reload();await expect(page.getByRole('combobox',{name:'入席秘社（可选）'})).toHaveValue(societyId);assert.equal(posts,0);
    await page.getByRole('button',{name:'收藏与组牌 →'}).click();
    await page.getByRole('button',{name:'载入当前入席快照'}).click();
    await page.getByRole('button',{name:'牌组设置与模板',exact:true}).click();
    await expect(page.getByRole('combobox',{name:'秘社（可选）'})).toHaveValue(societyId);
    await page.getByRole('button',{name:'新建空白牌组'}).click();
    const name=`${society.name}·浏览器命名`,renamed=`${name}·改名后`;
    await page.getByRole('textbox',{name:'牌组名称'}).fill(name);
    await page.getByRole('combobox',{name:'秘社（可选）'}).selectOption(societyId);
    let left=25;const entries=[];
    for(const card of fixture.catalog.cards.filter(c=>c.color===society.color&&['character','spell','attachment'].includes(c.kind)).sort((a,b)=>Number(b.unique)-Number(a.unique))) {
      const count=Math.min(left,card.deckCopyLimit??left);if(!count)continue;left-=count;entries.push({cardId:card.id,count});
      await page.getByRole('searchbox',{name:'检索卡牌'}).fill(card.id);
      await page.getByRole('button',{name:`添加 ${card.name}（${card.id}）`,exact:true}).click();
      await page.getByRole('spinbutton',{name:`${card.name}（${card.id}）张数`}).fill(String(count));
    }
    assert.equal(left,0);
    await page.getByRole('searchbox',{name:'检索卡牌'}).fill('JC125');
    await page.getByRole('button',{name:'添加 无知路人（JC125）',exact:true}).click();
    await page.getByRole('spinbutton',{name:'无知路人（JC125）张数'}).fill('25');entries.push({cardId:'JC125',count:25});
    await expect(page.getByRole('button',{name:'保存并选择此牌组'})).toBeEnabled();
    await page.getByRole('button',{name:'保存牌组',exact:true}).click();
    await page.reload();await expect(page.getByRole('textbox',{name:'牌组名称'})).toHaveValue(name);
    await page.getByRole('textbox',{name:'牌组名称'}).fill(renamed);await page.getByRole('button',{name:'保存牌组',exact:true}).click();
    await page.reload();await expect(page.getByRole('textbox',{name:'牌组名称'})).toHaveValue(renamed);
    await page.getByRole('button',{name:'牌组设置与模板',exact:true}).click();
    await expect(page.getByRole('combobox',{name:'秘社（可选）'})).toHaveValue(societyId);
    const stored=await page.evaluate(()=>Object.keys(localStorage).filter(k=>k.startsWith('hegemony.deckDraft.v2.')).map(k=>JSON.parse(localStorage.getItem(k))));
    assert.equal(stored.length,1);assert.equal(stored[0].name,renamed);assert.equal(stored[0].societyId,societyId);assert.deepEqual(stored[0].cards,entries);assert.equal(posts,0);
    await capture(page,`${viewport.height}-${societyId}-named-saved-reloaded`);
    await page.getByRole('button',{name:'新建空白牌组'}).click();
    await page.getByRole('list',{name:'已保存牌组'}).getByRole('button',{name:new RegExp('^'+renamed+' ')}).click();
    await expect(page.getByRole('textbox',{name:'牌组名称'})).toHaveValue(renamed);
    await page.getByRole('button',{name:'保存并选择此牌组'}).click();
    await expect(page.getByRole('combobox',{name:'入席秘社（可选）'})).toHaveValue(societyId);
    selected.push({page,societyId,name:renamed});run.decks.push({societyId,namedSavedRenamedReloadedLoaded:true,cardCount:50,printedColorCount:25,postsBeforeEntry:posts});
  }
  const host=selected[0].page,guest=selected[1].page;
  await host.getByRole('textbox',{name:'你的称呼'}).fill('红色本地玩家');
  let pending=waitPost(host,'/api/rooms');await hit(host.getByRole('button',{name:'创建牌桌 →'}));await host.getByRole('button',{name:'创建牌桌 →'}).click();let http=await pending;assert.equal(http.status(),200);const a=await http.json();
  await guest.goto(origin+'/?invite='+a.inviteCode+'#play');await expect(guest.getByRole('combobox',{name:'入席秘社（可选）'})).toHaveValue('MSJC05');
  await guest.getByRole('textbox',{name:'你的称呼'}).fill('灰色本地玩家');pending=waitPost(guest,'/api/rooms/join');await guest.getByRole('button',{name:'加入牌桌 →'}).click();http=await pending;assert.equal(http.status(),200);const b=await http.json();
  const sessions=[a,b];
  const state=async seat=>JSON.parse(abi.view((await store.room(a.roomId)).state,seat));
  for(let seat=0;seat<2;seat++){const v=await state(seat);assert.equal(v.yourDeck.name,selected[seat].name);assert.equal(v.yourDeck.societyId,selected[seat].societyId);}
  for(let seat=0;seat<2;seat++) {
    const page=selected[seat].page;await page.reload();await expect(page.locator('.hg-connection-online')).toBeVisible();
    const v=await state(seat),ready=v.legalActions.find(a=>a.kind==='ready');assert(ready);
    pending=waitPost(page,`/api/rooms/${a.roomId}/commands`);await page.getByRole('button',{name:ready.label,exact:true}).click();assert.equal((await pending).status(),200);
  }
  await host.reload();const before=await state(0),start=before.legalActions.find(a=>a.kind==='start');assert(start);
  pending=waitPost(host,`/api/rooms/${a.roomId}/commands`);await host.getByRole('button',{name:start.label,exact:true}).click();assert.equal((await pending).status(),200);
  let views=await Promise.all([0,1].map(state));assert.deepEqual(views[0].players.map(p=>p.handCount),[6,6]);
  assert.deepEqual(views[0].societyZones.map(z=>z.card.cardId),['MSJC04','MSJC05']);
  for(let seat=0;seat<2;seat++){await selected[seat].page.reload();await expect(selected[seat].page.locator('[data-society-zone]')).toHaveCount(2);assert.equal((await state(seat)).yourDeck.name,selected[seat].name);}
  await capture(host,`${viewport.height}-natural-start-red-gray`);
  // Finish mandatory mulligans only through each player's real choice control.
  for(let n=0;n<4;n++) {views=await Promise.all([0,1].map(state));const seat=views.findIndex(v=>v.pendingChoice);if(seat<0)break;
    const page=selected[seat].page;await page.reload();pending=waitPost(page,`/api/rooms/${a.roomId}/commands`);await page.getByRole('button',{name:'保留全部手牌',exact:true}).click();assert.equal((await pending).status(),200);}
  run.checks.push('initial selection and real25-color validation','no API POST before entry','name/save/rename/reload/load through UI','complete named snapshots create/join without injected room state','two real six-card starts, visible societies, refresh and mandatory choices');
  for(const {page}of selected)await page.context().close();await mf.dispose();mf=null;pages=[];
}
async function ability(viewport,societyId) {
  const scenario=fixture.scenarios.find(s=>s.name===societyId.toLowerCase()+'-search'),initial=scenario.initial;
  const {origin,db,store}=await startServer(`msjc0405-${societyId.toLowerCase()}-${viewport.height}`),id=initial.views[0].roomId;
  const tokens=Array.from({length:4},(_,i)=>String(i+1).repeat(64));
  await store.create({id,invite:initial.views[0].inviteCode,state:initial.state,nonce:crypto.randomUUID(),tokenHash:await digest(tokens[0]),requestHash:crypto.randomUUID(),intentHash:'explicit-native93-browser-layout',response:'{}'});
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.views[0].version,id).run();
  for(let seat=1;seat<4;seat++)await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id,seat,await digest(tokens[seat])).run();
  const run={viewport,societyId,scope:'explicit Native93 starting layout',posts:[0,0,0,0],commands:[],checks:[]};report.abilities.push(run);pages=[];
  for(let seat=0;seat<4;seat++) {const page=await context(viewport,{roomId:id,inviteCode:initial.views[0].inviteCode,seat,token:tokens[seat]});page.on('request',r=>{if(new URL(r.url()).pathname.startsWith('/api/')&&r.method()==='POST')run.posts[seat]++;});await page.goto(origin);await expect(page.locator('.hg-connection-online')).toBeVisible();}
  assert.equal((await store.room(id)).state,initial.state);assert.deepEqual(run.posts,[0,0,0,0]);
  const view=async seat=>JSON.parse(abi.view((await store.room(id)).state,seat));
  const command=async(seat,control)=>{
    const before=(await store.room(id)).state,count=run.posts[seat],page=pages[seat];await hit(control);
    const pending=waitPost(page,`/api/rooms/${id}/commands`);await control.click();const http=await pending;assert.equal(http.status(),200);
    const body=http.request().postDataJSON(),result=await http.json(),expected=JSON.parse(abi.applyRoom(before,seat,JSON.stringify(body),'0'));
    assert.equal(expected.outcome,'accepted');assertPublicView(result,expected.view);assert.equal((await store.room(id)).state,expected.state);
    const journal=await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id,expected.version).first();assertSessionJournal(journal.entry,expected.journal);
    await expect.poll(()=>run.posts[seat]).toBe(count+1);run.commands.push({seat,kind:body.action.kind,gameKind:body.action.action?.kind,version:expected.version,stateSha256:sha256(expected.state)});
  };
  const source=initial.views[0].societyZones[0].card,search=initial.views[0].legalActions.find(a=>a.abilityId===(societyId==='MSJC04'?'search-red-unique':'search-gray-unique'));
  assert(search);await pages[0].locator(`[data-card-instance="${source.instanceId}"]`).click();
  await command(0,pages[0].locator('[data-action-kind="activate"]').filter({hasText:'色独有检索'}));
  let current=await view(0);assert.equal(current.assets.filter(c=>c.exhausted).length,4);assert.equal(current.societyZones[0].card.exhausted,true);
  await pages[0].reload();assert.equal(run.posts[0],1);
  for(let n=0;n<16;n++) {
    const all=await Promise.all([0,1,2,3].map(view));const seat=all.findIndex(v=>v.responseWindow?.members.some(m=>m.playerId===v.you&&m.status==='undecided'));
    if(seat<0)break;await pages[seat].reload();await command(seat,pages[seat].getByRole('button',{name:'不连锁，让过',exact:true}));
  }
  current=await view(0);
  if(societyId==='MSJC04') {
    assert.equal(current.pendingChoice.options.length,2);assert(current.pendingChoice.options.every(o=>o.card.cardId==='JC052'));
    for(let seat=1;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator('.hg-choice')).toHaveCount(0);assert.equal((await view(seat)).pendingChoice,null);}
    await pages[0].reload();await pages[0].locator('.hg-choice-option').first().click();await capture(pages[0],`${viewport.height}-${societyId}-private-choice`);
    await command(0,pages[0].getByRole('button',{name:'确认选择',exact:true}));
    assert.deepEqual((await view(0)).hand.map(c=>c.cardId),['JC052']);run.checks.push('only red gold JC052 selectable, own private choice, public reveal and real new hand instance');
  } else {assert.equal(current.pendingChoice,null);assert.equal(current.hand.length,0);run.checks.push('no fake gray unique or private choice; real empty search resolves and shuffles');}
  const before=(await store.room(id)).state;
  for(let seat=0;seat<4;seat++){await pages[seat].reload();await expect(pages[seat].locator(`[data-card-instance="${source.instanceId}"]`)).toContainText('本局已使用');}
  assert.equal((await store.room(id)).state,before);current=await view(0);assert(current.societyZones[0].card.usedOncePerGame.includes(search.abilityId));
  assert(!current.legalActions.some(a=>a.abilityId===search.abilityId));run.checks.push('four real clients, payment4/exhaustion/quota survive refresh without extra POST, actual journals and same-command WASM equality');
  await capture(pages[0],`${viewport.height}-${societyId}-resolved-used`);
  for(const page of pages)await page.context().close();await mf.dispose();mf=null;pages=[];
}
try {
  browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium',args:['--no-sandbox','--disable-dev-shm-usage']});report.browser=browser.version();
  for(const viewport of [{width:1440,height:1000},{width:1366,height:768}]) {await natural(viewport);await ability(viewport,'MSJC04');await ability(viewport,'MSJC05');}
  assert.deepEqual(report.pageErrors,[]);report.passed=true;
} catch(error) {report.failure=String(error.stack||error);for(const [i,page]of pages.entries())try{await capture(page,'failure-'+i);}catch{}process.exitCode=1;}
finally {if(browser)await browser.close();if(mf)await mf.dispose();await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({passed:report.passed,naturalRuns:report.natural.length,abilityRuns:report.abilities.length,failure:report.failure,output}));}
