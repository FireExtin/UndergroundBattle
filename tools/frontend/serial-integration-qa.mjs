// Local browser display checks of exact current Native projections; never live gameplay.
// Run after generating ABI77 inputs: node tools/frontend/serial-integration-qa.mjs <new-evidence-directory>
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const web = join(repo, 'web'), require = createRequire(join(web, 'package.json'));
const { createServer } = await import(pathToFileURL(require.resolve('vite')).href);
const { chromium } = require('playwright');
const { expect } = require('playwright/test');
const output = resolve(process.argv[2] || '');
assert(process.argv[2], 'Provide a new evidence directory.');
await mkdir(output); // Preserve earlier evidence, including failed runs.
const bytes = await readFile(join(repo, 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
const wasmSha256 = createHash('sha256').update(bytes).digest('hex');
const abi = await import(pathToFileURL(join(repo, 'rust-game-wasm/pkg/hegemony_wasm.js')).href);
abi.initSync({ module: bytes });
const catalog = JSON.parse(abi.catalog());
assert.equal(catalog.engineVersion, 'rust-v0.2.77-jc046-mandatory-sacrifice-candidate');
const domains = JSON.parse(await readFile(join(web, 'src/game/jc060Native77.fixture.json')));
const sacrifice = JSON.parse(await readFile(join(web, 'src/game/jc046Native77.fixture.json')));
assert.deepEqual(domains.catalog, catalog); assert.deepEqual(sacrifice.catalog, catalog);
assert.equal(domains.wasmSha256, wasmSha256); assert.equal(sacrifice.wasmSha256, wasmSha256);
const characters = view => view.regions.flatMap(region => region.characters);
const stacking = domains.scenarios.find(row => row.name === 'stacking-return');
const two = characters(stacking.initial.views[0]).find(card => card.currentDomains?.some(domain => domain.count === 2));
const one = characters(stacking.steps.at(-1).views[0]).find(card => card.instanceId === two.instanceId);
const hidden = domains.scenarios.find(row => row.name === 'source-hide');
const granted = characters(hidden.initial.views[0]).find(card => card.currentDomains?.length);
const removed = characters(hidden.steps.at(-1).views[0]).find(card => card.instanceId === granted.instanceId);
assert.deepEqual(one.currentDomains, [{ count: 1, magic: '心灵' }]); assert.equal(removed.currentDomains, undefined);
const step = sacrifice.scenarios.find(row => row.name === 'choose-self-among-others').steps
  .find(row => row.command.action.action?.kind === 'choose' && !row.expected.errorCode);
const view = JSON.parse(abi.view(step.state, step.seat));
const fixture = await mkdtemp(join(web, '.serial-integration-qa-'));
const report = { passed: false, scope: 'Local browser + exact Native77 projections; fixture rendering, not live gameplay',
  engine: catalog.engineVersion, wasmSha256, apiCalls: 0, pageErrors: [], checks: [] };
let server, browser;
try {
  await writeFile(join(fixture, 'index.html'), '<!doctype html><html lang="zh"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><div id="root"></div><script type="module" src="./entry.tsx"></script></html>');
  await writeFile(join(fixture, 'entry.tsx'), `
import {useState} from 'react';
import {createRoot} from 'react-dom/client';
import {ReadModal} from '../src/game/ReadModal';
import {Table} from '../src/game/Table';
import {DeckLibrary} from '../src/game/DeckLibraryPanel';
import {actionForRoom} from '../src/game/api';
import '../src/game/game.css';
import '../src/styles/global.css';
const catalog=${JSON.stringify(catalog)}, cards=${JSON.stringify({ two, one, removed })}, view=${JSON.stringify(view)};
window.__reviewActions=[];
function App(){
 const [mode,setMode]=useState('two'); window.__setReviewCase=setMode;
 return <div className="hg-app"><header className="hg-header">本地 ABI77 投影检查 · 非线上试玩</header>
 {mode==='deck'?<DeckLibrary catalog={catalog}/>:mode==='choice'?<Table view={view} catalog={catalog} busy={false} connection="online" onAction={action=>window.__reviewActions.push(actionForRoom(view,action))}/>:<ReadModal key={mode} card={cards[mode]} definition={catalog.cards.find(card=>card.id===cards[mode].cardId)} viewerId="p0" onClose={()=>{}}/>}</div>;
}
createRoot(document.getElementById('root')!).render(<App/>);
`);
  server = await createServer({ root: web, logLevel: 'error', server: { host: '127.0.0.1', port: 0 } });
  await server.listen();
  const url = `http://127.0.0.1:${server.httpServer.address().port}/${fixture.slice(web.length + 1)}/index.html`;
  browser = await chromium.launch({ headless: true, executablePath: process.env.CHROMIUM_EXECUTABLE || '/usr/bin/chromium', args: ['--no-sandbox', '--disable-dev-shm-usage'] });
  for (const viewport of [{ width: 1366, height: 900 }, { width: 390, height: 844 }]) {
    const context = await browser.newContext({ viewport }), page = await context.newPage();
    page.on('pageerror', error => report.pageErrors.push(String(error)));
    await page.route('**/api/**', route => { report.apiCalls++; return route.abort(); });
    await page.goto(url);
    const setCase = async mode => { await page.evaluate(mode => window.__setReviewCase(mode), mode); };
    const group = page.getByRole('group', { name: '领域对照' });
    await expect(group).toHaveText('当前领域心灵 ×2印刷领域：无');
    await page.screenshot({ path: join(output, `${viewport.width}-domain-two.png`) });
    await setCase('one'); await expect(group).toHaveText('当前领域心灵 ×1印刷领域：无');
    await setCase('removed'); await expect(group).toHaveCount(0);
    await setCase('choice');
    let dialog = page.getByRole('dialog', { name: '待完成的选择' });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole('button', { name: '跳过此选择' })).toHaveCount(0);
    await expect(dialog.getByRole('button', { name: '确认选择' })).toBeDisabled();
    const selected = step.command.action.action.selected[0];
    const other = view.pendingChoice.options.find(option => option.id !== selected).id;
    await dialog.locator(`[data-choice-option="${other}"]`).click();
    await dialog.locator(`[data-choice-option="${selected}"]`).click();
    await expect(dialog.locator(`[data-choice-option="${other}"]`)).toHaveAttribute('aria-pressed', 'false');
    await page.reload(); await setCase('choice');
    dialog = page.getByRole('dialog', { name: '待完成的选择' });
    await expect(dialog.locator(`[data-choice-option="${selected}"]`)).toHaveAttribute('aria-pressed', 'true');
    assert.deepEqual(await page.evaluate(() => window.__reviewActions), []);
    await page.screenshot({ path: join(output, `${viewport.width}-mandatory-choice.png`) });
    await dialog.getByRole('button', { name: '确认选择' }).click();
    assert.deepEqual(await page.evaluate(() => window.__reviewActions), [step.command.action]);
    await setCase('deck');
    await page.getByRole('button', { name: '展开组卡工作台' }).click();
    await page.getByRole('searchbox', { name: '检索卡牌' }).fill('JC046');
    const entry = page.locator('[data-catalog-card="JC046"]');
    await expect(entry).toBeVisible();
    await expect(page.locator('[data-catalog-card]')).toHaveCount(1);
    await entry.getByRole('button', { name: /^添加 / }).click();
    await expect(entry).toHaveAttribute('data-selected-count', '1');
    await page.screenshot({ path: join(output, `${viewport.width}-deck.png`) });
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth);
    assert.equal(overflow, false, 'Deck workbench overflows horizontally');
    report.checks.push({ viewport, domainCounts: [2, 1, 0], mandatoryChoiceRestored: true,
      exactNativeAction: true, deckCard: 'JC046', deckCount: 1, horizontalOverflow: overflow });
    await context.close();
  }
  assert.equal(report.apiCalls, 0); assert.deepEqual(report.pageErrors, []);
  report.passed = true;
} catch (error) {
  report.failure = String(error); throw error;
} finally {
  await writeFile(join(output, 'summary.json'), JSON.stringify(report, null, 2) + '\n');
  await browser?.close(); await server?.close(); await rm(fixture, { recursive: true });
  console.log(JSON.stringify(report));
}
