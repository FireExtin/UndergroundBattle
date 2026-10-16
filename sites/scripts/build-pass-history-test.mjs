// Local dry-run test bundle only; never publishes or changes the provider checkout.
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import { published77Inputs, publishedIdentity, wasmSha256 } from '../test/published77.mjs';

const project = fileURLToPath(new URL('../', import.meta.url));
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
const { buildEnvironment } = await import(pathToFileURL(root + 'tools/build-storage.mjs').href);
const output = path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'pass-history-worker');
const environment = buildEnvironment(root, [output]);
const input = published77Inputs();
mkdirSync(output, { recursive: true });
writeFileSync(path.join(output, 'hegemony_wasm.js'), input.js);
writeFileSync(path.join(output, 'hegemony_wasm_bg.wasm'), input.wasm);
writeFileSync(path.join(output, 'worker.mjs'), [
  `import * as abi from './hegemony_wasm.js';`,
  `import module from './hegemony_wasm_bg.wasm';`,
  `import { createWorker } from ${JSON.stringify(project + 'src/http.mjs')};`,
  `import { lazyKernel } from ${JSON.stringify(project + 'src/lazy-kernel.mjs')};`,
  `import { routeKernels } from ${JSON.stringify(project + 'src/kernel-router.mjs')};`,
  `const kernel = routeKernels(lazyKernel(abi, module, ${JSON.stringify(publishedIdentity)}));`,
  // This header exists solely in the generated local test entry, never in src/index.mjs.
  `export default { fetch(request, env) { const now = Number(request.headers.get('X-Test-Server-Now') || 0); return createWorker(kernel, () => now).fetch(request, env); } };`,
].join('\n') + '\n');
const config = path.join(output, 'wrangler.json');
writeFileSync(config, JSON.stringify({ name: 'pass-history-local-check', main: path.join(output, 'worker.mjs'), compatibility_date: '2026-10-02', d1_databases: [{ binding: 'DB', database_name: 'pass-history-local' }] }));
execFileSync('npx', ['wrangler', 'deploy', '--dry-run', '--config', config, '--outdir', path.join(output, 'server')], {
  cwd: project, stdio: 'inherit', env: { ...environment, XDG_CONFIG_HOME: path.join(output, 'config'), WRANGLER_SEND_METRICS: 'false' },
});
console.log(JSON.stringify({ localOnly: true, wasmSha256, output }));
