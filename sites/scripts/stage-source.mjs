// Copy only the game, adapter and existing WASM into a standalone private Site checkout.
// Raw rule PDFs, the original card archive, runtime databases, traces and credentials stay out.
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildEnvironment } from '../../tools/build-storage.mjs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { gunzipSync } from 'node:zlib';

const project = fileURLToPath(new URL('../', import.meta.url));
const repository = fileURLToPath(new URL('../../', import.meta.url));
const destination = process.argv[2];
if (!destination || !path.isAbsolute(destination) || path.relative(repository, destination).startsWith('..') === false) {
  throw new Error('Use a separate absolute Site checkout outside the original repository.');
}
const manifest = JSON.parse(readFileSync(project + '.openai/hosting.json', 'utf8'));
if (existsSync(destination) && readdirSync(destination).length) {
  const prior = JSON.parse(readFileSync(path.join(destination, '.openai/hosting.json'), 'utf8'));
  if (prior.project_id !== manifest.project_id) throw new Error('Preserve this destination; it belongs to a different project.');
}
// The existing Site stores the large JC046 oracle as gzip and restores its exact
// reviewed bytes for build/test. Keep that source-storage contract when copying
// the GitHub tree, whose two JSON copies are intentionally uncompressed.
const storedFixture = 'scripts/test-fixture-storage.json';
const retainedStorage = new Map();
const retainedHooks = new Map();
if (existsSync(path.join(destination, storedFixture))) {
  const storage = JSON.parse(readFileSync(path.join(destination, storedFixture), 'utf8'));
  const targets = ['test/fixtures/jc046-native-v077.json', 'web/src/game/jc046Native77.fixture.json'];
  if (storage.archive !== 'test/fixtures/jc046-native-v077.json.gz'
      || JSON.stringify(storage.targets) !== JSON.stringify(targets)) throw new Error('Review the changed Site fixture-storage contract before staging.');
  const bytes = gunzipSync(readFileSync(path.join(destination, storage.archive)));
  const digest = value => createHash('sha256').update(value).digest('hex');
  if (bytes.length !== storage.bytes || digest(bytes) !== storage.sha256
      || targets.some(target => digest(readFileSync(repository + (target.startsWith('test/') ? 'sites/' : '') + target)) !== storage.sha256)) {
    throw new Error('The retained Site fixture must match both current reviewed JSON inputs.');
  }
  for (const name of [storedFixture, storage.archive, 'scripts/restore-test-fixtures.mjs', 'TEST_FIXTURE_STORAGE.md', '.gitignore']) {
    retainedStorage.set(name, readFileSync(path.join(destination, name)));
  }
  for (const name of ['package.json', 'web/package.json']) {
    const scripts = JSON.parse(readFileSync(path.join(destination, name), 'utf8')).scripts;
    const expected = name === 'package.json' ? 'node scripts/restore-test-fixtures.mjs' : 'node ../scripts/restore-test-fixtures.mjs';
    if (scripts.prebuild !== expected || scripts.pretest !== expected) throw new Error('Review the changed Site fixture restoration hooks before staging.');
    retainedHooks.set(name, { prebuild: scripts.prebuild, pretest: scripts.pretest });
  }
}
// Plan this finite copy list before any write/remove, including nested parents.
const copies = [];
const copy = (from, relative, directory = false) => copies.push({ from, relative, directory });
for (const file of ['package.json', 'package-lock.json', 'wrangler.jsonc', 'drizzle.config.ts', 'README.md', '.openai/hosting.json']) copy(project + file, file);
for (const directory of ['src', 'db', 'drizzle', 'test', 'scripts']) copy(project + directory, directory, true);
copy(repository + 'tools/build-storage.mjs', 'tools/build-storage.mjs');
for (const file of readdirSync(repository + 'tools').filter(name => name.endsWith('-oracle.mjs'))) copy(repository + 'tools/' + file, 'tools/' + file);
for (const file of ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']) copy(repository + file, file);
copy(repository + 'rust-game-wasm/build.sh', 'rust-game-wasm/build.sh');
copy(repository + 'rust-game-wasm/tests', 'rust-game-wasm/tests', true);
copy(repository + 'web/src', 'web/src', true);
copy(repository + 'web/public', 'web/public', true);
for (const file of ['package.json', 'package-lock.json', 'index.html', 'vite.config.ts', 'tsconfig.json', 'tsconfig.app.json', 'tsconfig.node.json']) copy(repository + 'web/' + file, 'web/' + file);
for (const crate of ['rust-game', 'rust-game-wasm']) {
  copy(repository + crate + '/src', crate + '/src', true);
  for (const file of ['Cargo.toml', 'Cargo.lock']) if (existsSync(repository + crate + '/' + file)) copy(repository + crate + '/' + file, crate + '/' + file);
}
// Only executable card definitions belong in the bounded Site source.
// Research/evidence indexes can contain every archived record and stay private in the original repository.
const removedDirectories = ['rust-game/data', 'rust-game-wasm/frozen', 'tools/frozen-kernels.mjs'];
copy(repository + 'rust-game/data/cards.json', 'rust-game/data/cards.json');
for (const file of ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm']) copy(repository + 'rust-game-wasm/pkg/' + file, 'rust-game-wasm/pkg/' + file);
// The standalone Site ships only the current core. Historical evidence stays in Git.
const wasmDestination = path.join(destination, 'rust-game-wasm');
if (existsSync(wasmDestination)) {
  for (const name of readdirSync(wasmDestination).filter(name => /^legacy-v0\.2\.\d+(?:-resource-policy)?$/.test(name))) removedDirectories.push('rust-game-wasm/' + name);
}
buildEnvironment(repository, [destination,
  ...copies.map(({ relative }) => path.join(destination, relative)),
  ...removedDirectories.map(relative => path.join(destination, relative)),
  path.join(destination, '.gitignore'), path.join(destination, 'source-provenance.json'),
]);
mkdirSync(destination, { recursive: true });
for (const relative of removedDirectories) rmSync(path.join(destination, relative), { recursive: true, force: true });
for (const { from, relative, directory } of copies) {
  const to = path.join(destination, relative);
  if (directory) rmSync(to, { recursive: true, force: true });
  mkdirSync(path.dirname(to), { recursive: true });
  cpSync(from, to, { recursive: directory });
}
writeFileSync(path.join(destination, '.gitignore'), 'node_modules/\ndist/\n.wrangler/\ngenerated/\n/public/\nweb/dist/\nrust-game/target/\nrust-game-wasm/target/\n.env*\n.sites-runtime/\n');
for (const [name, bytes] of retainedStorage) writeFileSync(path.join(destination, name), bytes);
for (const [name, hooks] of retainedHooks) {
  const file = path.join(destination, name);
  const manifest = JSON.parse(readFileSync(file, 'utf8'));
  Object.assign(manifest.scripts, hooks);
  writeFileSync(file, JSON.stringify(manifest, null, 2) + '\n');
}
const wasmSha256 = createHash('sha256').update(readFileSync(repository + 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm')).digest('hex');
const originalCommit = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repository, encoding: 'utf8' }).trim();
writeFileSync(path.join(destination, 'source-provenance.json'), JSON.stringify({ originalRepository: 'https://github.com/FireExtin/UndergroundBattle.git', originalCommit, wasmSha256 }, null, 2) + '\n');
console.log(JSON.stringify({ checkout_path: destination, project_id: manifest.project_id, wasm_sha256: wasmSha256 }));
