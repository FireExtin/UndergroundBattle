import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const project = fileURLToPath(new URL('../', import.meta.url));
// The private Site source is a bounded standalone checkout with web/core sources.
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
execFileSync('npm', ['run', 'build'], { cwd: root + 'web', stdio: 'inherit' });
rmSync(project + 'public', { recursive: true, force: true });
cpSync(root + 'web/dist', project + 'public', { recursive: true });
mkdirSync(project + 'generated', { recursive: true });
for (const name of ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']) cpSync(root + 'rust-game-wasm/pkg/' + name, project + 'generated/' + name);
cpSync(root + 'rust-game-wasm/legacy-v0.2.1', project + 'generated/legacy-v0.2.1', { recursive: true });
rmSync(project + 'dist', { recursive: true, force: true });
const config = project + '.wrangler/config';
mkdirSync(config, { recursive: true });
execFileSync('npx', ['wrangler', 'deploy', '--dry-run', '--outdir', 'dist/server'], {
  cwd: project, stdio: 'inherit', env: { ...process.env, XDG_CONFIG_HOME: config, WRANGLER_SEND_METRICS: 'false' },
});
cpSync(project + 'public', project + 'dist/client', { recursive: true });
mkdirSync(project + 'dist/.openai', { recursive: true });
cpSync(project + '.openai/hosting.json', project + 'dist/.openai/hosting.json');
