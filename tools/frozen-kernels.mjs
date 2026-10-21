// Historical regression inputs only. Production staging/builds never import this file.
// Exact original bytes remain external evidence; do not rebuild or substitute them.
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
// Keep this a runtime Node URL; Vite must not rewrite the directory as a web asset.
const moduleUrl = import.meta.url;
const repository = fileURLToPath(new URL('../', moduleUrl));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function frozenAbi(version, options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, `rust-game-wasm/frozen/abi${version}/identity.json`), 'utf8'));
  const label = 'Published';
  const directory = path.resolve(options.directory || process.env[`HEGEMONY_FROZEN_ABI${version}_DIR`]
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', `frozen-kernels/abi${version}`));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error(`Missing ${label.toLowerCase()} ABI${version} inputs; set HEGEMONY_FROZEN_ABI${version}_DIR to the retained, hash-verified runtime. No old-room migration or substitute build is allowed.`); }
  const mismatch = () => Error(`${label} ABI${version} runtime hash mismatch; preserve the inputs and stop.`);
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw mismatch();
  const size = version === 77 ? 2909006 : identity.wasmBytes;
  const wasm = gunzipSync(compressed, { maxOutputLength: size });
  if (wasm.length !== size || digest(wasm) !== identity.wasmSha256) throw mismatch();
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export const frozenAbi77 = (options = {}) => frozenAbi(77, options);
export const frozenAbi78 = (options = {}) => frozenAbi(78, options);
export const frozenAbi79 = (options = {}) => frozenAbi(79, options);
export const frozenAbi80 = (options = {}) => frozenAbi(80, options);
export const frozenAbi81 = (options = {}) => frozenAbi(81, options);
export const frozenAbi82 = (options = {}) => frozenAbi(82, options);
export const frozenAbi83 = (options = {}) => frozenAbi(83, options);
export const frozenAbi84 = (options = {}) => frozenAbi(84, options);
export const frozenAbi85 = (options = {}) => frozenAbi(85, options);
export const frozenAbi86 = (options = {}) => frozenAbi(86, options);
export const frozenAbi87 = (options = {}) => frozenAbi(87, options);
export const frozenAbi88 = (options = {}) => frozenAbi(88, options);
export const frozenAbi89 = (options = {}) => frozenAbi(89, options);
