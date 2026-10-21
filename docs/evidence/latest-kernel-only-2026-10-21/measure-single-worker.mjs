import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync, readdirSync, mkdtempSync } from 'node:fs';
import path from 'node:path';
const [repo, worker, persistenceParent, label] = process.argv.slice(2);
const { Miniflare, convertV4MiniflareOptions } = createRequire(path.join(repo, 'sites/package.json'))('miniflare');
const wasm = readdirSync(worker).filter(f => f.endsWith('.wasm'));
const start = performance.now();
const mf = new Miniflare(convertV4MiniflareOptions({
  name: 'latest-kernel-measurement', resourcePersistencePath: mkdtempSync(path.join(persistenceParent, label + '-')),
  modulesRoot: worker, modules: [{ type: 'ESModule', path: path.join(worker, 'index.js') },
    ...wasm.map(f => ({ type: 'CompiledWasm', path: path.join(worker, f) }))],
  compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'latest-kernel-measurement' },
}));
try {
  const health = await mf.dispatchFetch('http://localhost/api/health');
  assert.equal(health.status, 200);
  const healthBody = await health.json();
  assert.equal(healthBody.engineVersion, 'rust-v0.2.93-batch20-next');
  const healthMs = performance.now() - start;
  const db = await mf.getD1Database('DB');
  for (const file of readdirSync(path.join(repo, 'sites/drizzle')).filter(f => f.endsWith('.sql'))) {
    for (const sql of readFileSync(path.join(repo, 'sites/drizzle', file), 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const createStart = performance.now();
  const request = { requestId: crypto.randomUUID(), mode: 'duel', name: 'measurement', deckId: 'watchers' };
  const response = await mf.dispatchFetch('http://localhost/api/rooms', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(request) });
  const created = await response.json();
  assert.equal(response.status, 200, JSON.stringify(created));
  assert.equal(created.view.versions.engine, healthBody.engineVersion);
  const createMs = performance.now() - createStart;
  const read = await mf.dispatchFetch(`http://localhost/api/rooms/${created.roomId}/state`, { headers: { Authorization: 'Bearer ' + created.token } });
  assert.equal(read.status, 200);
  assert.equal((await read.json()).versions.engine, healthBody.engineVersion);
  console.log(JSON.stringify({ label, wasmModules: wasm.length, healthMs, createMs, readyAndNewRoomMs: performance.now() - start, nodeMaxRssKiB: process.resourceUsage().maxRSS, nodeRssBytes: process.memoryUsage().rss, currentRoomPassed: true }));
} finally { await mf.dispose(); }
