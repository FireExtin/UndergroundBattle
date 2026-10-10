import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
const project = fileURLToPath(new URL('../', import.meta.url));
// The private Site source is a bounded standalone checkout with web/core sources.
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
const { buildEnvironment } = await import(pathToFileURL(root + 'tools/build-storage.mjs').href);
const buildEnv = buildEnvironment(root, [root + 'web/dist', project + 'public', project + 'generated', project + 'dist', project + '.wrangler', project + '.wrangler/config']);
execFileSync('npm', ['run', 'build'], { cwd: root + 'web', stdio: 'inherit', env: buildEnv });
rmSync(project + 'public', { recursive: true, force: true });
cpSync(root + 'web/dist', project + 'public', { recursive: true });
// Remove stale deployment modules; frozen sources remain recoverable in Git.
rmSync(project + 'generated', { recursive: true, force: true });
mkdirSync(project + 'generated', { recursive: true });
for (const name of ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']) cpSync(root + 'rust-game-wasm/pkg/' + name, project + 'generated/' + name);
rmSync(project + 'dist', { recursive: true, force: true });
const config = project + '.wrangler/config';
mkdirSync(config, { recursive: true });
execFileSync('npx', ['wrangler', 'deploy', '--dry-run', '--outdir', 'dist/server'], {
  cwd: project, stdio: 'inherit', env: { ...buildEnv, XDG_CONFIG_HOME: config, WRANGLER_SEND_METRICS: 'false' },
});
cpSync(project + 'public', project + 'dist/client', { recursive: true });
mkdirSync(project + 'dist/.openai', { recursive: true });
cpSync(project + '.openai/hosting.json', project + 'dist/.openai/hosting.json');
