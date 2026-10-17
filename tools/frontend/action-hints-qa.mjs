// Local browser UI fixture. This does not establish game legality or online play.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const web = join(repo, 'web'), require = createRequire(join(web, 'package.json'));
const { createServer } = await import(pathToFileURL(require.resolve('vite')).href);
const { chromium } = require('playwright');
const engine = await import(pathToFileURL(join(repo, 'rust-game-wasm/pkg/hegemony_wasm.js')).href);
engine.initSync({ module: await readFile(join(repo, 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(engine.catalog());
const output = resolve(process.argv[2] || '/workspace/action-hints-evidence-20261017/browser');
await mkdir(output, { recursive: true });
const fixture = await mkdtemp(join(web, '.action-hints-qa-'));
let server, browser, apiCalls = 0;
const errors = [], rows = [];
try {
  await writeFile(join(fixture, 'index.html'), '<!doctype html><html lang="zh"><meta charset="utf-8"><div id="root"></div><script type="module" src="./entry.tsx"></script></html>');
  await writeFile(join(fixture, 'entry.tsx'), `
import {useState} from 'react';import {createRoot} from 'react-dom/client';
import {Table} from '../src/game/Table';import {testCard,testView} from '../src/game/testFixtures';
import '../src/game/game.css';import '../src/styles/global.css';
const catalog=${JSON.stringify(catalog)},def=catalog.cards.find(c=>c.id==='JC125');
const hand={...testCard,instanceId:'hint-hand',cardId:def.id,name:def.name,icons:def.permanentIcons,text:def.text,cost:def.cost,defense:def.defense};
const hidden={...hand,instanceId:'hint-hidden',faceDown:true,region:0};
const enemy={...hidden,instanceId:'hint-enemy',owner:'p1',controller:'p1',name:'不得显示的身份'};
const standard={id:'standard',kind:'deploy',cardId:hand.instanceId,region:0,label:'派遣 → 地区'};
const reveal={id:'reveal',kind:'reveal',cardId:hidden.instanceId,label:'现身'};
const initial={...testView,status:'playing',hand:[hand],players:[{...testView.players[0],handCount:1},{...testView.players[0],id:'p1',seat:1,name:'乙',team:1,handCount:0}],regions:catalog.cards.filter(c=>c.kind==='region').slice(0,3).map((c,index)=>({id:'hint-region-'+index,index,cardId:c.id,name:c.name,threshold:c.threshold,points:c.points,influence:[0,0],characters:index===0?[hidden,enemy]:[]})),legalActions:[standard,reveal,{id:'pass',kind:'pass',label:'让过'}]};
window.hintActions=[];
function App(){const [stage,setStage]=useState('action');window.hintStage=setStage;
const waiting=stage==='waiting';const current={...initial,version:waiting?2:1,phase:waiting?'end':'action',step:waiting?'end':'action',priorityTeam:waiting?1:0,legalActions:waiting?[]:initial.legalActions};
return <div className="hg-app"><header className="hg-header">候选行动提示 · 本地界面检查</header><Table view={current} catalog={catalog} busy={false} connection="online" onAction={a=>window.hintActions.push(a)}/></div>}
createRoot(document.getElementById('root')!).render(<App/>);`);
  server = await createServer({ root: web, logLevel: 'error', server: { host: '127.0.0.1', port: 0 } });
  await server.listen();
  browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  const url = `http://127.0.0.1:${server.httpServer.address().port}/${fixture.slice(web.length + 1)}/index.html`;
  for (const viewport of [{ width: 1440, height: 1000 }, { width: 1366, height: 768 }]) {
    const context = await browser.newContext({ viewport }), page = await context.newPage();
    page.on('pageerror', error => errors.push(String(error)));
    await page.route('**/api/**', route => { apiCalls++; return route.abort(); });
    await page.goto(url);
    const hand = page.locator('[data-card-instance="hint-hand"]');
    await hand.locator('.hg-card-action-hint').waitFor();
    assert.equal(await page.locator('[data-card-instance="hint-hidden"] .hg-card-action-hint').count(), 1);
    assert.equal(await page.locator('[data-card-instance="hint-enemy"] .hg-card-action-hint').count(), 0);
    assert.equal(await page.getByText('不得显示的身份', { exact: true }).count(), 0);
    const measurements = await page.locator('[data-card-actionable="true"]').evaluateAll(cards => cards.map(card => {
      const badge = card.querySelector('.hg-card-action-hint'), b = badge.getBoundingClientRect();
      const overlap = [...card.querySelectorAll('.hg-card-name, .hg-cost, .hg-icons, .hg-card-footer')].filter(el => {
        const r = el.getBoundingClientRect();return r.width && r.height && Math.min(b.right,r.right)>Math.max(b.left,r.left) && Math.min(b.bottom,r.bottom)>Math.max(b.top,r.top);
      }).map(el=>el.className);
      return { id:card.getAttribute('data-card-instance'),text:badge.textContent,badgeVisible:b.width>0&&b.height>0&&b.bottom<=innerHeight,overlap,
        geometry: [...card.querySelectorAll('.hg-card-name, .hg-archive-art, .hg-card-action-hint')].map(el=>({className:el.className,rect:el.getBoundingClientRect().toJSON(),position:getComputedStyle(el).position})) };
    }));
    await page.screenshot({ path: join(output, `${viewport.width}x${viewport.height}-action.png`), fullPage: true });
    await writeFile(join(output, `${viewport.width}x${viewport.height}-geometry.json`), JSON.stringify(measurements, null, 2));
    assert.ok(measurements.every(row => row.badgeVisible && row.overlap.length === 0), JSON.stringify(measurements));
    await page.screenshot({ path: join(output, `${viewport.width}x${viewport.height}-action.png`), fullPage: true });
    await page.getByRole('button', { name: '让过', exact: true }).click();
    assert.equal((await page.evaluate(() => window.hintActions)).length, 1);
    await page.evaluate(() => window.hintStage('waiting'));
    await page.waitForFunction(() => !document.querySelector('[data-card-actionable="true"]'));
    assert.ok((await page.getByRole('region', { name: '当前窗口与候选行动' }).textContent()).includes('结束阶段'));
    await page.screenshot({ path: join(output, `${viewport.width}x${viewport.height}-waiting.png`), fullPage: true });
    await page.reload();
    await hand.locator('.hg-card-action-hint').waitFor();
    rows.push({ viewport, measurements, waitingCleared: true, reloadRecomputed: true, passUnchanged: true });
    await context.close();
  }
  assert.deepEqual(errors, []); assert.equal(apiCalls, 0);
  await writeFile(join(output, 'result.json'), JSON.stringify({ scope: 'Local UI fixture, not gameplay or publication', rows, errors, apiCalls }, null, 2));
  console.log(JSON.stringify({ output, viewports: rows.length, errors, apiCalls }));
} finally {
  await browser?.close(); await server?.close(); await rm(fixture, { recursive: true, force: true });
}
