// Backend-only local compilation; no front-end build and no publishing path.
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
const repository = existsSync(root + 'web/package.json') ? root : fileURLToPath(new URL('../../', import.meta.url));
const fixture = process.argv[2] === '--society-fixtures';
const historical = process.argv[2] === '--legacy-v0.2.11';
if (process.argv.length > (fixture || historical ? 3 : 2)) throw Error('Usage: build-backend-test.mjs [--society-fixtures | --legacy-v0.2.11]');
const outputRoot = process.env.HEGEMONY_BACKEND_TEST_ROOT || path.resolve(root, '../../.private-validation/society-foundation-20261003');
const base = path.join(outputRoot, fixture ? 'fixture-worker' : historical ? 'legacy-v0.2.11-worker' : 'candidate-worker');
mkdirSync(base, { recursive: true });
rmSync(base + '/src', { recursive: true, force: true });
cpSync(root + 'src', base + '/src', { recursive: true });
mkdirSync(base + '/generated', { recursive: true });
const current = repository + 'rust-game-wasm/' + (fixture ? 'pkg-society-fixtures/' : historical ? 'legacy-v0.2.11/' : 'pkg/');
for (const name of ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']) cpSync(current + name, base + '/generated/' + name);
for (const name of readdirSync(repository + 'rust-game-wasm').filter(name => /^legacy-v0\.2\.\d+(?:-resource-policy)?$/.test(name))) cpSync(repository + 'rust-game-wasm/' + name, base + '/generated/' + name, { recursive: true });
writeFileSync(base + '/wrangler.json', JSON.stringify({ name: fixture ? 'hegemony-society-fixture-test' : 'hegemony-society-candidate-test', main: 'src/index.mjs', compatibility_date: '2026-10-02', d1_databases: [{ binding: 'DB', database_name: 'hegemony-test' }] }));
writeFileSync(base + '/package.json', '{"type":"module","private":true}\n');
// Only the isolated test copy selects a fixture or the original printed-society core.
// The normal source, generated modules and production build retain their approved identity.
if (fixture || historical) {
  const abi = await import(pathToFileURL(current + 'hegemony_wasm.js'));
  abi.initSync({ module: readFileSync(current + 'hegemony_wasm_bg.wasm') });
  const catalog = JSON.parse(abi.catalog());
  if (historical ? catalog.engineVersion !== 'rust-v0.2.11' : !catalog.engineVersion.endsWith('-fixture') || !catalog.societies.every(s => s.id.startsWith('FIXTURE_'))) throw Error('Wrong isolated test core');
  const identity = { rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion, engineVersion: catalog.engineVersion };
  const source = readFileSync(base + '/src/kernel.mjs', 'utf8');
  const declaration = /^const kernel0 = lazyKernel\(current, currentModule, \{[^\n]+\}\);$/m;
  if (!declaration.test(source)) throw Error('Expected the existing current-core declaration');
  writeFileSync(base + '/src/kernel.mjs', source.replace(declaration, `const kernel0 = lazyKernel(current, currentModule, ${JSON.stringify(identity)});`));
}
execFileSync(root + 'node_modules/.bin/wrangler', ['deploy', '--dry-run', '--config', base + '/wrangler.json', '--outdir', base + '/dist'], { cwd: base, stdio: 'inherit', env: { ...process.env, XDG_CONFIG_HOME: base + '/.wrangler-config', WRANGLER_SEND_METRICS: 'false' } });
if (!fixture && !historical) {
  cpSync(base + '/generated', root + 'generated', { recursive: true });
  rmSync(root + 'dist/server', { recursive: true, force: true });
  cpSync(base + '/dist', root + 'dist/server', { recursive: true });
}
console.log(JSON.stringify({ fixtureRegistry: fixture, historicalEngine: historical ? 'rust-v0.2.11' : null, dist: base + '/dist', frontendBuilt: false, deployed: false }));
