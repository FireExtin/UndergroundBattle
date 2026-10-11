// Local compiled Worker + persistent D1 SQLite. No Cloudflare account required.
// First run the current WASM and Sites builds; this starts their exact output.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';

const root = fileURLToPath(new URL('../', import.meta.url));
const name = 'hegemony-qa63';
const port = Number(process.env.PORT || 8113);
assert(Number.isInteger(port) && port >= 0 && port <= 65535, 'PORT must be 0..65535');
const stateDir = path.resolve(process.env.HEGEMONY_QA_STATE_DIR || path.join(tmpdir(), 'undergroundbattle-engine63-preview'));
mkdirSync(stateDir, { recursive: true });
const server = path.join(root, 'dist/server');
const mf = new Miniflare(convertV4MiniflareOptions({
  name, rootPath: root, host: process.env.HOST || '127.0.0.1', port,
  cf: false, compatibilityDate: '2026-10-02', resourcePersistencePath: stateDir,
  modules: [
    { type: 'ESModule', path: path.join(server, 'index.js') },
    ...readdirSync(server).filter(file => file.endsWith('.wasm'))
      .map(file => ({ type: 'CompiledWasm', path: path.join(server, file) })),
  ],
  d1Databases: { DB: 'hegemony-qa63-db' },
  assets: { directory: path.join(root, 'dist/client'), binding: 'ASSETS', run_worker_first: ['/api/*'],
    routerConfig: { has_user_worker: true }, assetConfig: { not_found_handling: 'single-page-application' } },
}));

try {
  const url = await mf.ready, db = await mf.getD1Database('DB', name);
  const tables = ['commands', 'entry_receipts', 'journal', 'rooms', 'seats'];
  const existing = await db.prepare("SELECT name FROM sqlite_master WHERE type='table'").all();
  const present = tables.filter(table => existing.results.some(row => row.name === table));
  if (!present.length) {
    const statements = readdirSync(path.join(root, 'drizzle')).filter(file => file.endsWith('.sql')).sort()
      .flatMap(file => readFileSync(path.join(root, 'drizzle', file), 'utf8')
        .split('--> statement-breakpoint').filter(sql => sql.trim()));
    await db.batch(statements.map(sql => db.prepare(sql)));
  } else assert.equal(present.length, tables.length, 'Incomplete local QA schema');
  const response = await fetch(new URL('/api/health', url));
  assert.equal(response.status, 200);
  const health = await response.json();
  assert.equal(health.engineVersion, 'rust-v0.2.63-bq030-fixed-slots-candidate');
  const catalogResponse = await fetch(new URL('/api/catalog', url));
  assert.equal(catalogResponse.status, 200);
  const catalog = await catalogResponse.json();
  assert.equal(catalog.cardPoolVersion, 'limited-v2.56-bq030-attachment-candidate');
  assert.equal(catalog.engineVersion, health.engineVersion);
  assert.equal(catalog.cards.length, 129);
  assert(catalog.cards.some(card => card.id === 'BQ030'));
  console.log(`READY ${url.href}`);
  console.log(`ENGINE ${health.engineVersion} | local workerd and persistent D1 SQLite`);
  console.log('Parent browser gameplay acceptance remains pending. Stop with Ctrl+C.');
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, async () => {
    await mf.dispose(); process.exit(0);
  });
} catch (error) {
  await mf.dispose(); throw error;
}
