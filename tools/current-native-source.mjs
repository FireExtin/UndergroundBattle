// Test evidence for the current kernel. This does not migrate persisted rooms.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync} from 'node:fs';
import {gzipSync, gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {candidateNativePath} from './candidate-native-path.mjs';
import {catalog, moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';

const moduleUrl = import.meta.url;
const root = fileURLToPath(new URL('../', moduleUrl));
export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
export const currentIdentity = Object.fromEntries(
  ['rulesVersion', 'cardPoolVersion', 'engineVersion'].map(key => [key, catalog[key]]));
export const currentWasmSha256 = sha256(moduleBytes);
const inputs = ['Cargo.lock', 'Cargo.toml', 'rust-toolchain.toml',
  'rust-game/Cargo.toml', 'rust-game-wasm/Cargo.toml', 'rust-game-wasm/build.sh'];
function collect(directory) {
  if (!existsSync(path.join(root, directory))) return;
  for (const item of readdirSync(path.join(root, directory), {withFileTypes: true})) {
    const name = `${directory}/${item.name}`;
    if (item.isDirectory()) collect(name);
    else if (item.isFile() && (name.endsWith('.rs') || name === 'rust-game/data/cards.json')) inputs.push(name);
  }
}
for (const directory of ['rust-game/src', 'rust-game/tests', 'rust-game/examples',
  'rust-game/data', 'rust-game-wasm/src', 'rust-game-wasm/examples']) collect(directory);
const files = Object.fromEntries([...new Set(inputs)].filter(name => existsSync(path.join(root, name)))
  .sort().map(name => [name, sha256(readFileSync(path.join(root, name)))]));
// The complete local Rust source inventory complements the per-export compiler
// bindings below, whose older hand-written file lists are only subsets.
export const currentSourceManifest = {algorithm: 'sha256', identity: currentIdentity,
  wasmSha256: currentWasmSha256, files};
export const currentSourceManifestSha256 = sha256(JSON.stringify(currentSourceManifest));

export function assertCurrentCatalog(actual) {
  assert.deepEqual(actual, catalog, 'Native catalog must equal the current WASM catalog');
}
export function assertCurrentSourceBindings(binding) {
  assert.equal(binding.algorithm, 'sha256');
  assert(Object.keys(binding.files).length > 0, 'Missing Native compiler inputs');
  for (const [name, hash] of Object.entries(binding.files)) {
    assert(Object.hasOwn(files, name), `Native input outside current source inventory: ${name}`);
    assert.equal(hash, files[name], `Native compiler input differs from current source: ${name}`);
  }
}

export const currentFixturePath = relativePath => candidateNativePath(relativePath, root);

export function loadCurrentFixture(relativePath) {
  const bundle = JSON.parse(gunzipSync(readFileSync(currentFixturePath(relativePath))));
  assert.equal(bundle.format, 'current-native-fixture-v1');
  assert.equal(bundle.currentSourceManifestSha256, currentSourceManifestSha256);
  assert.equal(bundle.wasmSha256, currentWasmSha256);
  assert.equal(Buffer.byteLength(bundle.raw.json), bundle.raw.bytes);
  assert.equal(sha256(bundle.raw.json), bundle.raw.sha256);
  return JSON.parse(bundle.raw.json);
}

export function writeCurrentFixture(relativePath, fixture) {
  const json = JSON.stringify(fixture);
  const destination = currentFixturePath(relativePath);
  const bundle = {format: 'current-native-fixture-v1', currentSourceManifestSha256,
    wasmSha256: currentWasmSha256, raw: {json, bytes: Buffer.byteLength(json), sha256: sha256(json)}};
  mkdirSync(path.dirname(destination), {recursive: true});
  writeFileSync(destination, gzipSync(JSON.stringify(bundle), {level: 9}), {flag: 'wx'});
}
