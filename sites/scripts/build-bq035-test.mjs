// Exact production modules, with a deterministic clock in this local test entry.
// This entry is outside the source tree and has no publishing path.
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { buildEnvironment } from '../../tools/build-storage.mjs';
import { sha256, bq035Identity } from '../../tools/bq035-oracle.mjs';
import { catalog } from '../../rust-game-wasm/tests/current78-kernel.mjs';
import { frozenAbi77 } from '../../tools/frozen-kernels.mjs';
const project = fileURLToPath(new URL('../', import.meta.url));
const root = fileURLToPath(new URL('../../', import.meta.url));
const output = path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'bq035-worker');
const env = buildEnvironment(root, [output]);
for (const key of Object.keys(bq035Identity)) if (catalog[key] !== bq035Identity[key]) throw Error('Wrong current candidate');
const current = readFileSync(root + 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
if (sha256(current) !== sha256(readFileSync(project + 'generated/hegemony_wasm_bg.wasm'))) throw Error('Build the candidate Site before the deterministic Worker test');
const retained = frozenAbi77();
if (sha256(readFileSync(project + 'generated/abi77/hegemony_wasm_bg.wasm')) !== retained.identity.wasmSha256) throw Error('Wrong retained ABI77');
mkdirSync(output, { recursive: true });
writeFileSync(path.join(output, 'worker.mjs'), [
  `import { kernel } from ${JSON.stringify(project + 'src/kernel.mjs')};`,
  `import { createWorker } from ${JSON.stringify(project + 'src/http.mjs')};`,
  `export default createWorker(kernel, () => 0);`,
].join('\n') + '\n');
const config = path.join(output, 'wrangler.json');
writeFileSync(config, JSON.stringify({ name: 'bq035-local-check', main: path.join(output, 'worker.mjs'), compatibility_date: '2026-10-02', d1_databases: [{ binding: 'DB', database_name: 'bq035-local' }] }));
execFileSync('npx', ['wrangler', 'deploy', '--dry-run', '--config', config, '--outdir', path.join(output, 'server')], {
  cwd: project, stdio: 'inherit', env: { ...env, XDG_CONFIG_HOME: path.join(output, 'config'), WRANGLER_SEND_METRICS: 'false' },
});
console.log(JSON.stringify({ localOnly: true, currentWasmSha256: sha256(current), retained77Sha256: retained.identity.wasmSha256, output, injectedClockOnlyInLocalEntry: true }));
