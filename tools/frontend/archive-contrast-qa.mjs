// Browser regression for the real Table/CardContent/ReadModal and CSS cascade.
// UI fixture only: this checks colours, never game rules or a real room.
// Run: node tools/frontend/archive-contrast-qa.mjs [evidence-directory]
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const web = join(repo, 'web'), require = createRequire(join(web, 'package.json'));
const { createServer } = await import(pathToFileURL(require.resolve('vite')).href);
const { chromium } = require('playwright'); // Use the available browser tool; no dependency changes.
const engine = await import(pathToFileURL(join(repo, 'rust-game-wasm/pkg/hegemony_wasm.js')).href);
engine.initSync({ module: await readFile(join(repo, 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(engine.catalog()), surgeon = catalog.cards.find(c => c.id === 'LC19');
assert.equal(surgeon.name, '外科医生');
const output = process.argv[2] ? resolve(process.argv[2]) : await mkdtemp(join(tmpdir(), 'archive-contrast-results-'));
await mkdir(output, { recursive: true });
const fixture = await mkdtemp(join(web, '.archive-contrast-qa-'));
let server, browser;
const rows = [], failures = [], pageErrors = []; let apiCalls = 0;
try {
  await writeFile(join(fixture, 'index.html'), '<!doctype html><html lang="zh"><meta charset="utf-8"><div id="root"></div><script type="module" src="./entry.tsx"></script></html>');
  await writeFile(join(fixture, 'entry.tsx'), `
import {createRoot} from 'react-dom/client';
import {Table} from '../src/game/Table';
import {testCard,testView} from '../src/game/testFixtures';
import '../src/game/game.css';
import '../src/styles/global.css';
const catalog=${JSON.stringify(catalog)}, definition=catalog.cards.find(c=>c.id==='LC19');
const card={...testCard,instanceId:'contrast-surgeon',cardId:definition.id,name:definition.name,cost:definition.cost,text:definition.text,icons:definition.permanentIcons,defense:definition.defense};
const view={...testView,status:'playing',players:[{...testView.players[0],handCount:1},{...testView.players[0],id:'p1',seat:1,name:'乙',team:1,handCount:0}],hand:[card],regions:catalog.cards.filter(c=>c.kind==='region').slice(0,3).map((c,index)=>({id:'contrast-region-'+index,index,cardId:c.id,name:c.name,threshold:c.threshold,points:c.points,influence:[0,0],characters:[]})),legalActions:[]};
createRoot(document.getElementById('root')!).render(<div className="hg-app"><header className="hg-header">UI 颜色回归 · 非对局</header><Table view={view} catalog={catalog} busy={false} connection="online" onAction={()=>{throw Error('No game actions in colour regression')}} /></div>);
`);
  server = await createServer({ root: web, logLevel: 'error', server: { host: '127.0.0.1', port: 0 } });
  await server.listen();
  browser = await chromium.launch({ headless: true, executablePath: process.env.CHROMIUM_EXECUTABLE || '/usr/bin/chromium', args: ['--no-sandbox'] });
  const url = `http://127.0.0.1:${server.httpServer.address().port}/${fixture.slice(web.length + 1)}/index.html`;
  for (const viewport of [{ width: 1440, height: 1000 }, { width: 1366, height: 768 }]) {
    const context = await browser.newContext({ viewport }), page = await context.newPage();
    page.on('pageerror', error => pageErrors.push(String(error)));
    await page.route('**/api/**', route => { apiCalls++; return route.abort(); });
    await page.goto(url);
    const capture = async (surface, selector, fields = ['.hg-card-name', '.hg-card-kind', '.hg-card-affiliation', '.hg-card-text', '.hg-cost', '.hg-icons > span', '.hg-icons b', '.hg-card-footer > span']) => {
      const panel = page.locator(selector); await panel.waitFor({ state: 'visible' });
      const values = await panel.evaluate((root, fields) => {
        const rgba = value => { const n = value.match(/[\d.]+/g).map(Number); return [...n.slice(0, 3), n[3] ?? 1]; };
        const blend = (front, back) => front.slice(0, 3).map((v, i) => v * front[3] + back[i] * (1 - front[3]));
        const backdrop = node => {
          const ancestors = []; for (let el = node; el; el = el.parentElement) ancestors.unshift(el);
          return ancestors.reduce((colour, el) => blend(rgba(getComputedStyle(el).backgroundColor), colour), [255, 255, 255]);
        };
        const luminance = rgb => rgb.map(v => { v /= 255; return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4; }).reduce((n, v, i) => n + v * [.2126, .7152, .0722][i], 0);
        return fields.flatMap(selector => [...root.querySelectorAll(selector)].filter(el => el.getBoundingClientRect().height > 0 && el.textContent.trim()).map(el => {
          const style = getComputedStyle(el), background = backdrop(el), foreground = blend(rgba(style.color), background);
          const a = luminance(foreground), b = luminance(background), contrast = (Math.max(a, b) + .05) / (Math.min(a, b) + .05);
          return { selector, text: el.textContent.trim(), foreground: style.color, background, fontSize: style.fontSize, contrast };
        }));
      }, fields);
      assert.ok(values.length, `No measured text on ${surface}`);
      for (const field of fields) assert.ok(values.some(row => row.selector === field), `Missing ${surface} ${field}`);
      const label = `${viewport.width}x${viewport.height}-${surface}`;
      rows.push({ label, background: await panel.evaluate(el => getComputedStyle(el).backgroundColor), values });
      failures.push(...values.filter(row => row.contrast < 4.5).map(row => ({ label, ...row })));
      await panel.screenshot({ path: join(output, label + '.png') });
    };
    const tile = page.locator('[data-card-instance="contrast-surgeon"]');
    await tile.hover(); await capture('hover', '.hg-card-hover');
    await tile.click(); await page.locator('.hg-inspected-card').hover();
    await capture('details', '.hg-inspected-card');
    await page.screenshot({ path: join(output, `${viewport.width}x${viewport.height}-table.png`), fullPage: true });
    await page.getByRole('button', { name: '放大文字与图标 ↗', exact: true }).click();
    await capture('reader-text', '.hg-reading-card');
    await page.getByRole('button', { name: '原始牌面', exact: true }).click();
    const scan = page.getByRole('img', { name: '外科医生原始牌面', exact: true });
    assert.equal(await scan.getAttribute('src'), '/cards/LC19.jpg');
    await scan.evaluate(img => img.complete && img.naturalWidth > 0 ? undefined : new Promise((resolve, reject) => { img.onload = resolve; img.onerror = reject; }));
    await capture('reader-original-caption', '.hg-source-reading', ['p']);
    await context.close();
  }
  await writeFile(join(output, 'computed-contrast.json'), JSON.stringify({ fixture: 'LC19 exact local kernel catalog + explicit UI test view; not gameplay', minimum: 4.5, rows, failures, apiCalls, pageErrors }, null, 2));
  assert.equal(apiCalls, 0); assert.deepEqual(pageErrors, []);
  assert.deepEqual(failures, [], 'Actual computed text/background contrast below 4.5:1; see computed-contrast.json');
  console.log(JSON.stringify({ output, surfaces: rows.length, minimumMeasured: Math.min(...rows.flatMap(row => row.values.map(value => value.contrast))), apiCalls, pageErrors }));
} finally {
  await browser?.close(); await server?.close(); await rm(fixture, { recursive: true, force: true });
}
