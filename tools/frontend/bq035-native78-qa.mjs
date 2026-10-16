// Local browser rendering of exact Native78 projections. This is not online play.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadBq035Fixture, sha256 } from '../bq035-oracle.mjs';
import { kernel as abi, moduleBytes, catalog } from '../../rust-game-wasm/tests/current78-kernel.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const web = join(root, 'web'), require = createRequire(join(web, 'package.json'));
const { createServer } = await import(pathToFileURL(require.resolve('vite')).href);
const { chromium } = require('playwright'), { expect } = require('playwright/test');
assert(process.argv[2], 'Provide a new evidence directory');
const output = resolve(process.argv[2]); await mkdir(output);
const fixture = loadBq035Fixture(); assert.equal(sha256(moduleBytes), fixture.wasmSha256);
const scenario = fixture.scenarios.find(row => row.name === 'paid-attachments-mobility-glory-next-turn');
const index = scenario.steps.findIndex(step => !step.expected.errorCode && step.command.action.action?.kind === 'choose' && step.command.action.action.selected?.[0] === 'region:1');
const step = scenario.steps[index], view = index ? scenario.steps[index - 1].views[step.seat] : scenario.initial.views[step.seat];
assert.deepEqual(JSON.parse(abi.view(step.state, step.seat)), view);
assert(view.pendingChoice && view.pendingChoice.playerId === view.you, 'The browser fixture must use the actual frozen chooser');
const removed = fixture.scenarios.find(row => row.name === 'accepted-knight-real-bounce').steps.at(-1).views[0];
const knight = view.regions.flatMap(region => region.characters).find(card => card.cardId === 'BQ035'); assert(knight);
const temporary = await mkdtemp(join(web, '.bq035-browser-'));
const report = { passed: false, scope: 'Local exact Native78 projections and UI; not a published Site or online gameplay',
  engine: catalog.engineVersion, wasmSha256: fixture.wasmSha256, apiCalls: 0, pageErrors: [], checks: [] };
let server, browser;
try {
  await writeFile(join(temporary, 'index.html'), '<!doctype html><html lang="zh"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><div id="root"></div><script type="module" src="./entry.tsx"></script></html>');
  await writeFile(join(temporary, 'entry.tsx'), `
import {useState} from 'react'; import {createRoot} from 'react-dom/client';
import {Table} from '../src/game/Table'; import {ReadModal} from '../src/game/ReadModal'; import {actionForRoom} from '../src/game/api';
import '../src/game/game.css'; import '../src/styles/global.css';
const catalog=${JSON.stringify(catalog)}, view=${JSON.stringify(view)}, removed=${JSON.stringify(removed)}, knight=${JSON.stringify(knight)};
window.__bqActions=[];
function App(){ const [mode,setMode]=useState('choice'); window.__bqMode=setMode;
return <div className="hg-app"><header className="hg-header">BQ035 本地投影检查 · 非线上试玩</header>
{mode==='card'?<ReadModal card={knight} definition={catalog.cards.find(c=>c.id==='BQ035')} viewerId="p0" onClose={()=>setMode('choice')}/>
:<Table key={mode} view={mode==='removed'?removed:view} catalog={catalog} busy={false} connection="online" onAction={action=>window.__bqActions.push(JSON.parse(JSON.stringify(actionForRoom(view,action))))}/>}</div>; }
createRoot(document.getElementById('root')!).render(<App/>);
`);
  server = await createServer({ root: web, logLevel: 'error', server: { host: '127.0.0.1', port: 0 } }); await server.listen();
  const url = `http://127.0.0.1:${server.httpServer.address().port}/${temporary.slice(web.length + 1)}/index.html`;
  browser = await chromium.launch({ headless: true, executablePath: process.env.CHROMIUM_EXECUTABLE || '/usr/bin/chromium', args: ['--no-sandbox', '--disable-dev-shm-usage'] });
  report.browser = browser.version();
  for (const viewport of [{ width: 1366, height: 768 }, { width: 390, height: 844 }]) {
    const context = await browser.newContext({ viewport }), page = await context.newPage();
    page.on('pageerror', error => report.pageErrors.push(String(error)));
    await page.route('**/api/**', route => { report.apiCalls++; return route.abort(); });
    await page.goto(url);
    const setMode = mode => page.evaluate(mode => window.__bqMode(mode), mode);
    let dialog = page.getByRole('dialog', { name: '待完成的选择' });
    await expect(dialog).toBeVisible(); await expect(dialog.getByRole('button', { name: '跳过此选择' })).toBeEnabled();
    await expect(page.getByText('机动 · 幽冥骑士授予', { exact: true }).first()).toBeVisible();
    await dialog.locator('[data-choice-option="region:1"]').click();
    await page.reload(); dialog = page.getByRole('dialog', { name: '待完成的选择' });
    await expect(dialog.locator('[data-choice-option="region:1"]')).toHaveAttribute('aria-pressed', 'true');
    assert.deepEqual(await page.evaluate(() => window.__bqActions), []);
    const bounds = await dialog.evaluate(root => {
      const panel = root.getBoundingClientRect(), confirm = root.querySelector('.hg-choice-footer .hg-button-primary').getBoundingClientRect();
      return { top: panel.top, bottom: panel.bottom, confirmTop: confirm.top, confirmBottom: confirm.bottom, height: innerHeight };
    });
    assert(bounds.top >= 0 && bounds.bottom <= viewport.height && bounds.confirmTop >= 0 && bounds.confirmBottom <= viewport.height, JSON.stringify(bounds));
    const size = `${viewport.width}x${viewport.height}`;
    await page.screenshot({ path: join(output, `${size}-mobility-choice.png`) });
    await dialog.getByRole('button', { name: '确认选择' }).click();
    assert.deepEqual(await page.evaluate(() => window.__bqActions), [step.command.action]);
    await setMode('removed'); await expect(page.getByText('机动 · 幽冥骑士授予', { exact: true })).toHaveCount(0);
    await setMode('card'); await expect(page.getByRole('dialog')).toContainText('幽冥骑士');
    await expect(page.getByRole('dialog')).toContainText('所有有名为鲜血');
    await page.getByRole('button', { name: '原始牌面' }).click();
    const image = page.getByRole('img', { name: '幽冥骑士原始牌面' }); await expect(image).toBeVisible();
    await image.evaluate(image => image.complete && image.naturalWidth > 0 ? undefined : new Promise((resolve, reject) => {
      image.onload = () => resolve(undefined); image.onerror = () => reject(Error('Pinned BQ035 image failed to load'));
    }));
    await page.screenshot({ path: join(output, `${size}-original-card.png`) });
    report.checks.push({ viewport, bounds, optionalChoice: true, draftAfterReload: true, exactNativeAction: true, departedSourceBadgeRemoved: true, originalCardLoaded: true });
    await context.close();
  }
  assert.equal(report.apiCalls, 0); assert.deepEqual(report.pageErrors, []); report.passed = true;
} catch (error) { report.error = String(error.stack || error); throw error; }
finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser?.close(); await server?.close(); await rm(temporary, { recursive: true, force: true });
}
console.log(JSON.stringify(report));
