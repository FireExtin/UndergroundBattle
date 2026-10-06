// Backend-only local compilation; no front-end build and no publishing path.
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
const repository = existsSync(root + 'web/package.json') ? root : fileURLToPath(new URL('../../', import.meta.url));
const fixture = process.argv[2] === '--society-fixtures';
if (process.argv.length > (fixture ? 3 : 2)) throw Error('Usage: build-backend-test.mjs [--society-fixtures]');
const outputRoot = process.env.HEGEMONY_BACKEND_TEST_ROOT || path.resolve(root, '../../.private-validation/society-foundation-20261003');
const base = path.join(outputRoot, fixture ? 'fixture-worker' : 'candidate-worker');
mkdirSync(base, { recursive: true });
rmSync(base + '/src', { recursive: true, force: true });
cpSync(root + 'src', base + '/src', { recursive: true });
mkdirSync(base + '/generated', { recursive: true });
const current = repository + 'rust-game-wasm/' + (fixture ? 'pkg-society-fixtures/' : 'pkg/');
for (const name of ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']) cpSync(current + name, base + '/generated/' + name);
for (const name of readdirSync(repository + 'rust-game-wasm').filter(name => /^legacy-v0\.2\.\d+(?:-resource-policy)?$/.test(name))) cpSync(repository + 'rust-game-wasm/' + name, base + '/generated/' + name, { recursive: true });
writeFileSync(base + '/wrangler.json', JSON.stringify({ name: fixture ? 'hegemony-society-fixture-test' : 'hegemony-society-candidate-test', main: 'src/index.mjs', compatibility_date: '2026-10-02', d1_databases: [{ binding: 'DB', database_name: 'hegemony-test' }] }));
writeFileSync(base + '/package.json', '{"type":"module","private":true}\n');
execFileSync(root + 'node_modules/.bin/wrangler', ['deploy', '--dry-run', '--config', base + '/wrangler.json', '--outdir', base + '/dist'], { cwd: base, stdio: 'inherit', env: { ...process.env, XDG_CONFIG_HOME: base + '/.wrangler-config', WRANGLER_SEND_METRICS: 'false' } });
if (!fixture) {
  cpSync(base + '/generated', root + 'generated', { recursive: true });
  rmSync(root + 'dist/server', { recursive: true, force: true });
  cpSync(base + '/dist', root + 'dist/server', { recursive: true });
}
console.log(JSON.stringify({ fixtureRegistry: fixture, dist: base + '/dist', frontendBuilt: false, deployed: false }));
