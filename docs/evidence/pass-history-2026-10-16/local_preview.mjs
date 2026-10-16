// Isolated local production-bundle check. No remote bindings or user rooms.
import { createServer } from 'node:http';
import { readFileSync, readdirSync, existsSync, mkdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
const root = '/workspace/pass-context-abi77-review-20261016';
const require = createRequire(root + '/sites/package.json');
const { Miniflare, convertV4MiniflareOptions } = require('miniflare');
const persistence = '/workspace/undergroundbattle-build/pass-history-browser-20261016';
mkdirSync(persistence, { recursive: false });
const directory = root + '/sites/dist/server';
const mf = new Miniflare(convertV4MiniflareOptions({ name: 'public-pass-browser', resourcePersistencePath: persistence, modulesRoot: directory,
  modules: [{ type: 'ESModule', path: directory + '/index.js' }, ...readdirSync(directory).filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: directory + '/' + name }))],
  compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: 'public-pass-browser-only' } }));
const db = await mf.getD1Database('DB');
for (const file of readdirSync(root + '/sites/drizzle').filter(name => name.endsWith('.sql'))) {
  for (const sql of readFileSync(root + '/sites/drizzle/' + file, 'utf8').split('--> statement-breakpoint').filter(sql => sql.trim())) await db.prepare(sql).run();
}
const assets = root + '/sites/dist/client';
const mime = { '.html':'text/html; charset=utf-8', '.js':'text/javascript', '.css':'text/css', '.jpg':'image/jpeg', '.png':'image/png', '.svg':'image/svg+xml', '.webp':'image/webp', '.ico':'image/x-icon' };
const server = createServer(async (incoming, outgoing) => {
  try {
    const url = new URL(incoming.url, 'http://127.0.0.1:8129');
    if (url.pathname.startsWith('/api/')) {
      const chunks = []; for await (const chunk of incoming) chunks.push(chunk);
      const body = Buffer.concat(chunks);
      const response = await mf.dispatchFetch(url.href, { method: incoming.method, headers: incoming.headers,
        ...(body.length ? { body } : {}) });
      outgoing.writeHead(response.status, Object.fromEntries(response.headers));
      outgoing.end(Buffer.from(await response.arrayBuffer())); return;
    }
    const candidate = path.resolve(assets, '.' + decodeURIComponent(url.pathname));
    if (candidate !== assets && !candidate.startsWith(assets + '/')) { outgoing.writeHead(400); outgoing.end(); return; }
    const file = existsSync(candidate) && statSync(candidate).isFile() ? candidate : assets + '/index.html';
    outgoing.writeHead(200, { 'Content-Type': mime[path.extname(file)] || 'application/octet-stream', 'Cache-Control':'no-store' });
    outgoing.end(readFileSync(file));
  } catch { outgoing.writeHead(503); outgoing.end('Local test service unavailable'); }
});
server.listen(8129, '127.0.0.1', () => process.stdout.write('Local production-bundle test ready at http://127.0.0.1:8129\n'));
async function close() { server.close(); await mf.dispose(); process.exit(0); }
process.on('SIGTERM', close); process.on('SIGINT', close);
