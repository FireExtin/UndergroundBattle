// Two real browser contexts, production GameApp/assets, listening HTTP,
// workerd/WASM and persistent local D1. Only the initial layout and clock are
// deterministic test inputs; no API mocking, opaque-state rewrite or live Site.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadBq035Fixture, sha256 } from '../bq035-oracle.mjs';
import { kernel as abi, moduleBytes } from '../../rust-game-wasm/tests/current78-kernel.mjs';
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
const fixture = loadBq035Fixture(); assert.equal(sha256(moduleBytes), fixture.wasmSha256);
const scenario = fixture.scenarios.find(row => row.name === 'wm023-duel-paid-conversion-blood-knight-move'); assert(scenario);
const initial = scenario.initial.views[0], id = initial.roomId;
assert.equal(initial.players.length, 2);
const host = initial.regions.flatMap(region => region.characters).find(card => card.cardId === 'WM023').instanceId;
const tokens = ['1'.repeat(64), '2'.repeat(64)]; // Local fixture seats, never real credentials.
const worker = join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'bq035-worker/server');
const persist = await mkdtemp(join(process.env.TMPDIR, 'bq035-p2-real-http-'));
const options = { name: 'bq035-p2-real-http', rootPath: sites, modulesRoot: worker, host: '127.0.0.1', port: 0,
  cf: false, compatibilityDate: '2026-10-02', resourcePersistencePath: persist,
  modules: [{ type: 'ESModule', path: join(worker, 'worker.js') },
    ...(await readdir(worker)).filter(file => file.endsWith('.wasm')).map(file => ({ type: 'CompiledWasm', path: join(worker, file) }))],
  d1Databases: { DB: 'bq035-p2-real-http' },
  assets: { directory: join(sites, 'dist/client'), binding: 'ASSETS', run_worker_first: ['/api/*'],
    routerConfig: { has_user_worker: true }, assetConfig: { not_found_handling: 'single-page-application' } },
};
const report = { passed: false, scope: 'Prepared existing WM023, followed by paid cards and real two-client HTTP mechanism chain; local deterministic server clock, not online gameplay',
  engine: fixture.catalog.engineVersion, wasmSha256: fixture.wasmSha256, workerSha256: sha256(await readFile(join(worker, 'worker.js'))),
  fixtureSha256: sha256(await readFile(join(repo, 'rust-game-wasm/tests/bq035Native78.fixture.json.gz'))),
  commands: [], clientRequests: [0, 0], clientPosts: [0, 0], pageErrors: [], screenshots: [], reopened: 0 };
let mf, db, browser, pages = [], origin;
const start = async () => {
  mf = new Miniflare(convertV4MiniflareOptions(options));
  const url = await mf.ready; origin = url.origin; options.port = Number(url.port);
  db = await mf.getD1Database('DB', options.name);
};
const store = () => new RoomStore(db);
const snapshot = async () => ({ room: await store().room(id), commands: (await db.prepare('SELECT command_id,intent_hash,response,version FROM commands WHERE room_id=? ORDER BY version').bind(id).all()).results });
try {
  await start();
  for (const file of (await readdir(join(sites, 'drizzle'))).filter(file => file.endsWith('.sql'))) {
    for (const sql of (await readFile(join(sites, 'drizzle', file), 'utf8')).split('--> statement-breakpoint').filter(sql => sql.trim())) await db.prepare(sql).run();
  }
  await store().create({ id, invite: initial.inviteCode, state: scenario.initial.state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]), requestHash: crypto.randomUUID(), intentHash: 'offline-p2-initial-layout', response: '{}' });
  await db.prepare('UPDATE rooms SET version=? WHERE id=?').bind(initial.version, id).run();
  await db.prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(id, 1, await digest(tokens[1])).run();
  browser = await chromium.launch({ headless: true, executablePath: process.env.CHROMIUM_EXECUTABLE || '/usr/bin/chromium', args: ['--no-sandbox', '--disable-dev-shm-usage'] });
  report.browser = browser.version();
  for (let seat = 0; seat < 2; seat++) {
    const context = await browser.newContext({ viewport: { width: 1366, height: 900 } });
    await context.addInitScript(saved => {
      localStorage.setItem('hegemony.session.v1', JSON.stringify(saved));
      localStorage.setItem('hegemony.seats.v1', JSON.stringify([saved]));
      localStorage.setItem('hegemony.screen.v1', 'table');
      localStorage.setItem('hegemony.autoPass.v1', 'false');
    }, { roomId: id, inviteCode: initial.inviteCode, seat, token: tokens[seat] });
    const page = await context.newPage(); pages.push(page);
    page.on('pageerror', error => report.pageErrors.push(String(error)));
    page.on('request', request => { if (new URL(request.url()).pathname.startsWith('/api/')) { report.clientRequests[seat]++; if (request.method() === 'POST') report.clientPosts[seat]++; } });
    await page.goto(origin);
    await expect(page.locator(`[data-card-instance="${host}"]`)).toBeVisible();
  }
  const apiView = async seat => pages[seat].evaluate(async id => {
    const saved = JSON.parse(localStorage.getItem('hegemony.session.v1'));
    const response = await fetch(`/api/rooms/${id}/state`, { headers: { Authorization: 'Bearer ' + saved.token } });
    if (!response.ok) throw Error('View HTTP ' + response.status); return response.json();
  }, id);
  const capture = async label => {
    for (let seat = 0; seat < 2; seat++) { const file = `${label}-seat${seat}.png`; await pages[seat].screenshot({ path: join(output, file) }); report.screenshots.push(file); }
  };
  let lastCommand, lastReceipt;
  for (const [index, step] of scenario.steps.entries()) {
    const before = (await store().room(id)).state; assert.equal(before, step.state);
    const page = pages[step.seat], kind = step.command.action.kind;
    // Reload restores the actual saved seat, obtains an authoritative view and
    // resets only the display clock in this deterministic-clock test harness.
    await page.reload(); await expect(page.locator(`[data-card-instance="${host}"]`)).toBeVisible();
    let response, command, via = 'browser-fetch';
    if (kind === 'pauseRoom' || kind === 'resumeRoom' || kind === 'passResponse' || kind === 'game' && step.command.action.action.kind === 'choose') {
      via = 'production-ui';
      const pending = page.waitForResponse(response => new URL(response.url()).pathname === `/api/rooms/${id}/commands` && response.request().method() === 'POST');
      if (kind === 'pauseRoom') {
        const dialog = page.getByRole('dialog', { name: '待完成的选择' });
        if (await dialog.count()) await dialog.getByRole('button', { name: '暂停并保存此桌', exact: true }).click();
        else await page.getByRole('button', { name: '暂停并保存', exact: true }).click();
      } else if (kind === 'resumeRoom') await page.getByRole('button', { name: '恢复对局', exact: true }).click();
      else if (kind === 'passResponse') await page.getByRole('button', { name: '不连锁，让过', exact: true }).click();
      else {
        const dialog = page.getByRole('dialog', { name: '待完成的选择' }); await expect(dialog).toBeVisible();
        await expect(pages[1].getByRole('dialog', { name: '待完成的选择' })).toHaveCount(0);
        await capture('wm023-before-choice');
        await dialog.locator('[data-choice-option="region:1"]').click();
        await dialog.getByRole('button', { name: '确认选择', exact: true }).click();
      }
      const http = await pending; assert.equal(http.status(), 200); command = http.request().postDataJSON(); response = await http.json();
    } else {
      command = step.command;
      response = await page.evaluate(async ({ id, command }) => {
        const saved = JSON.parse(localStorage.getItem('hegemony.session.v1'));
        const response = await fetch(`/api/rooms/${id}/commands`, { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: 'Bearer ' + saved.token }, body: JSON.stringify(command) });
        if (!response.ok) throw Error('Command HTTP ' + response.status); return response.json();
      }, { id, command });
    }
    assert.deepEqual(command.action, step.command.action); assert.equal(command.expectedVersion, step.command.expectedVersion);
    const expected = JSON.parse(abi.applyRoom(before, step.seat, JSON.stringify(command), '0'));
    assert.equal(expected.state, step.expected.state); assertPublicView(response, expected.view);
    assert.equal((await store().room(id)).state, step.expected.state);
    const journal = await db.prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(id, expected.version).first(); assertSessionJournal(journal.entry, expected.journal);
    for (let seat = 0; seat < 2; seat++) assertPublicView(await apiView(seat), step.views[seat]);
    report.commands.push({ index, seat: step.seat, kind, gameKind: step.command.action.action?.kind, via, status: 200, version: expected.version, stateSha256: sha256(expected.state) });
    lastCommand = { seat: step.seat, command }; lastReceipt = response;
    if (kind === 'pauseRoom') {
      const saved = await snapshot(); await mf.dispose(); await start(); assert.deepEqual(await snapshot(), saved); report.reopened++;
      for (let seat = 0; seat < 2; seat++) { await pages[seat].reload(); assertPublicView(await apiView(seat), step.views[seat]); }
      await capture('paused-' + report.reopened);
    }
  }
  for (let seat = 0; seat < 2; seat++) {
    await pages[seat].reload(); const view = await apiView(seat);
    assert(view.regions[1].characters.some(card => card.instanceId === host && card.owner === 'p1' && card.controller === 'p0'));
    assert(!view.regions[0].characters.some(card => card.instanceId === host));
    const attached = view.attachments.filter(item => item.hostId === host); assert.equal(attached.length, 2); assert(attached.every(item => item.region === 1));
    await expect(pages[seat].locator(`#hg-region-1 [data-card-instance="${host}"]`)).toBeVisible();
  }
  await capture('wm023-moved-with-attachments');
  const frozen = await snapshot();
  const duplicate = await pages[lastCommand.seat].evaluate(async ({ id, command }) => {
    const saved = JSON.parse(localStorage.getItem('hegemony.session.v1'));
    const response = await fetch(`/api/rooms/${id}/commands`, { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: 'Bearer ' + saved.token }, body: JSON.stringify(command) });
    if (!response.ok) throw Error('Duplicate HTTP ' + response.status); return response.json();
  }, { id, command: lastCommand.command });
  assert.deepEqual(duplicate, lastReceipt); assert.deepEqual(await snapshot(), frozen);
  assert.equal(report.commands.length, scenario.steps.length); assert(report.clientPosts.every(n => n > 0)); assert.equal(report.reopened, 2); assert.deepEqual(report.pageErrors, []);
  report.actualMovement = { instanceId: host, from: 0, to: 1, owner: 'p1', controller: 'p0', attachedCards: 2 };
  report.exactNativeStatesAndCurrentWasmJournals = true; report.originalReceiptAfterReopen = true; report.passed = true;
} catch (error) { report.error = String(error.stack || error); throw error; }
finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser?.close(); await mf?.dispose();
}
console.log(JSON.stringify(report));
