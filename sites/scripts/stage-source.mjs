// Copy only the game, adapter and existing WASM into a standalone private Site checkout.
// Raw rule PDFs, the original card archive, runtime databases, traces and credentials stay out.
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildEnvironment } from '../../tools/build-storage.mjs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const project = fileURLToPath(new URL('../', import.meta.url));
if (existsSync(project + 'web/package.json')) throw new Error('Stage from the product Git checkout, not an already staged Site source.');
const repository = fileURLToPath(new URL('../../', import.meta.url));
const destination = process.argv[2];
if (!destination || !path.isAbsolute(destination) || path.relative(repository, destination).startsWith('..') === false) {
  throw new Error('Use a separate absolute Site checkout outside the original repository.');
}
const manifest = JSON.parse(readFileSync(project + '.openai/hosting.json', 'utf8'));
// Only inputs used by the current default tests travel with the standalone Site.
// The untouched historical originals remain in the product repository.
const testInputs = new Set(JSON.parse(readFileSync(project + 'scripts/test-inputs.json', 'utf8')));
for (const file of testInputs) if (!existsSync(repository + file)) throw new Error('Missing current test input: ' + file);
const include = source => {
  const relative = path.relative(repository, source).split(path.sep).join('/');
  if (relative.endsWith('.json') && (relative.startsWith('sites/test/fixtures/') || relative.startsWith('web/src/game/'))) return testInputs.has(relative);
  return true;
};
if (existsSync(destination) && readdirSync(destination).length) {
  const prior = JSON.parse(readFileSync(path.join(destination, '.openai/hosting.json'), 'utf8'));
  if (prior.project_id !== manifest.project_id) throw new Error('Preserve this destination; it belongs to a different project.');
}
// Current Native fixtures are shared gzip inputs; no historical restoration hook is needed.
// Plan this finite copy list before any write/remove, including nested parents.
const copies = [];
const copy = (from, relative, directory = false) => copies.push({ from, relative, directory });
for (const file of ['package.json', 'package-lock.json', 'wrangler.jsonc', 'drizzle.config.ts', 'README.md', '.openai/hosting.json']) copy(project + file, file);
for (const directory of ['src', 'db', 'drizzle', 'test', 'scripts']) copy(project + directory, directory, true);
copy(repository + 'tools/build-storage.mjs', 'tools/build-storage.mjs');
copy(repository + 'tools/current-native-source.mjs', 'tools/current-native-source.mjs');
copy(repository + 'tools/build-current-clock-worker.mjs', 'tools/build-current-clock-worker.mjs');
for (const file of readdirSync(repository + 'tools').filter(name => name.endsWith('-oracle.mjs'))) copy(repository + 'tools/' + file, 'tools/' + file);
for (const file of ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']) copy(repository + file, file);
copy(repository + 'rust-game-wasm/build.sh', 'rust-game-wasm/build.sh');
copy(repository + 'web/src', 'web/src', true);
copy(repository + 'web/public', 'web/public', true);
// Exact originals read by the current card-face tests; keep the archive out.
const scans = JSON.parse(readFileSync(repository + 'web/public/card-scans.json', 'utf8'));
for (const id of ['JC037', 'JC046', 'JC052', 'JC060', 'JC061', 'JC087', 'JC094', 'JC095',
  'JZ35', 'MSJC04', 'MSJC05', 'WM023', 'WM038', 'WM056', 'XQ25']) {
  const original = scans[id].source;
  if (!original.startsWith('resource/ymsj-fun.github.io/cards/') || original.includes('..')) throw new Error('Unexpected original card path: ' + id);
  copy(repository + original, original);
}
for (const file of ['package.json', 'package-lock.json', 'index.html', 'vite.config.ts', 'tsconfig.json', 'tsconfig.app.json', 'tsconfig.node.json']) copy(repository + 'web/' + file, 'web/' + file);
for (const crate of ['rust-game', 'rust-game-wasm']) {
  copy(repository + crate + '/src', crate + '/src', true);
  for (const directory of ['tests', 'examples']) if (existsSync(repository + crate + '/' + directory))
    copy(repository + crate + '/' + directory, crate + '/' + directory, true);
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
  cpSync(from, to, { recursive: directory, filter: include });
}
writeFileSync(path.join(destination, '.gitignore'), 'node_modules/\ndist/\n.wrangler/\ngenerated/\n/public/\nweb/dist/\nrust-game/target/\nrust-game-wasm/target/\n.env*\n.sites-runtime/\n');
// Sites flattens the adapter by one directory. Relocate only literal links
// from adapter tests/scripts to shared repository inputs; game code stays unchanged.
for (const directory of ['test', 'scripts']) {
  const relocate = folder => {
    for (const entry of readdirSync(folder, {withFileTypes: true})) {
      const file = path.join(folder, entry.name);
      if (entry.isDirectory()) relocate(file);
      else if (entry.name.endsWith('.mjs')) {
        const source = readFileSync(file, 'utf8');
        const relocated = source.replace(/(['"])(\.\.\/)+(?=(?:tools|rust-game-wasm|rust-game|web)\/)/g,
          (match, quote) => match.split('../').length > 2 ? quote + match.slice(quote.length + 3) : match);
        if (relocated !== source) writeFileSync(file, relocated);
      }
    }
  };
  relocate(path.join(destination, directory));
}
rmSync(path.join(destination, 'TEST_FIXTURE_STORAGE.md'), {force: true});
const wasmSha256 = createHash('sha256').update(readFileSync(repository + 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm')).digest('hex');
const originalCommit = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repository, encoding: 'utf8' }).trim();
writeFileSync(path.join(destination, 'source-provenance.json'), JSON.stringify({ originalRepository: 'https://github.com/FireExtin/UndergroundBattle.git', originalCommit, wasmSha256 }, null, 2) + '\n');
console.log(JSON.stringify({ checkout_path: destination, project_id: manifest.project_id, wasm_sha256: wasmSha256 }));
