// Exact production modules, with a deterministic clock in this local test entry.
// This entry is outside the source tree and has no publishing path.
import { mkdirSync, writeFileSync, readFileSync, readdirSync, existsSync, renameSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
const project = fileURLToPath(new URL('../', import.meta.url));
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
const { buildEnvironment } = await import(pathToFileURL(root + 'tools/build-storage.mjs').href);
const { sha256, identity } = await import(pathToFileURL(root + 'tools/wm062-oracle.mjs').href);
const { catalog } = await import(pathToFileURL(root + 'rust-game-wasm/tests/current80-kernel.mjs').href);
const { frozenAbi77, frozenAbi78, frozenAbi79 } = await import(pathToFileURL(root + 'tools/frozen-kernels.mjs').href);
const output = path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'bq035-worker');
const server = path.join(output, 'server');
const previousServer = path.join(output, `server-prior-${Date.now()}-${process.pid}`);
const env = buildEnvironment(root, [output, server, previousServer]);
for (const key of Object.keys(identity)) if (catalog[key] !== identity[key]) throw Error('Wrong current candidate');
const current = readFileSync(root + 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
if (sha256(current) !== sha256(readFileSync(project + 'generated/hegemony_wasm_bg.wasm'))) throw Error('Build the candidate Site before the deterministic Worker test');
const retained = frozenAbi77();
const retained78 = frozenAbi78();
const retained79 = frozenAbi79();
if (sha256(readFileSync(project + 'generated/abi79/hegemony_wasm_bg.wasm')) !== retained79.identity.wasmSha256) throw Error('Wrong retained ABI79');
if (sha256(readFileSync(project + 'generated/abi78/hegemony_wasm_bg.wasm')) !== retained78.identity.wasmSha256) throw Error('Wrong retained ABI78');
if (sha256(readFileSync(project + 'generated/abi77/hegemony_wasm_bg.wasm')) !== retained.identity.wasmSha256) throw Error('Wrong retained ABI77');
mkdirSync(output, { recursive: true });
writeFileSync(path.join(output, 'worker.mjs'), [
  `import { kernel } from ${JSON.stringify(project + 'src/kernel.mjs')};`,
  `import { createWorker } from ${JSON.stringify(project + 'src/http.mjs')};`,
  `export default createWorker(kernel, () => 0);`,
].join('\n') + '\n');
const config = path.join(output, 'wrangler.json');
writeFileSync(config, JSON.stringify({ name: 'bq035-local-check', main: path.join(output, 'worker.mjs'), compatibility_date: '2026-10-02', d1_databases: [{ binding: 'DB', database_name: 'bq035-local' }] }));
// Wrangler retains unreferenced content-addressed modules in an existing
// output directory. Preserve the whole prior output before producing this
// batch, so a strict local harness cannot register an obsolete kernel.
const archivedPreviousServer = existsSync(server) ? previousServer : null;
if (archivedPreviousServer) renameSync(server, archivedPreviousServer);
execFileSync('npx', ['wrangler', 'deploy', '--dry-run', '--config', config, '--outdir', server], {
  cwd: project, stdio: 'inherit', env: { ...env, XDG_CONFIG_HOME: path.join(output, 'config'), WRANGLER_SEND_METRICS: 'false' },
});
const modules = readdirSync(server).filter(file => file.endsWith('.wasm'))
  .map(file => ({ file, sha256: sha256(readFileSync(path.join(server, file))) }));
if (JSON.stringify(modules.map(row => row.sha256).sort()) !== JSON.stringify([sha256(current), retained79.identity.wasmSha256, retained78.identity.wasmSha256, retained.identity.wasmSha256].sort())) throw Error('Local Worker must contain exactly candidate80 and retained79/78/77');
console.log(JSON.stringify({ localOnly: true, currentWasmSha256: sha256(current), retained77Sha256: retained.identity.wasmSha256, output, modules, archivedPreviousServer, injectedClockOnlyInLocalEntry: true }));
