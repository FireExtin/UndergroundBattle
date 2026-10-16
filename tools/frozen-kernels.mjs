// Exact retained runtime inputs live outside Git. Never rebuild, download or
// silently substitute a historical kernel when its published bytes are absent.
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
// Keep this a runtime Node URL; Vite must not rewrite the directory as a web asset.
const moduleUrl = import.meta.url;
const repository = fileURLToPath(new URL('../', moduleUrl));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export function frozenAbi77(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi77/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI77_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi77'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI77 inputs; set HEGEMONY_FROZEN_ABI77_DIR to the retained, hash-verified runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) {
    throw Error('Published ABI77 runtime hash mismatch; preserve the inputs and stop.');
  }
  const wasm = gunzipSync(compressed, { maxOutputLength: 2909006 });
  if (wasm.length !== 2909006 || digest(wasm) !== identity.wasmSha256) {
    throw Error('Published ABI77 runtime hash mismatch; preserve the inputs and stop.');
  }
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi77(destination, options = {}) {
  const saved = frozenAbi77(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// v60's precise ABI78 is an additional active-room dependency. Keep it
// separate from the freshly built candidate and from ABI77's original bytes.
export function frozenAbi78(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi78/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI78_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi78'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI78 inputs; set HEGEMONY_FROZEN_ABI78_DIR to the exact v60 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI78 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI78 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi78(destination, options = {}) {
  const saved = frozenAbi78(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// v61 is independently retained; its bytes are never rebuilt as candidate80.
export function frozenAbi79(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi79/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI79_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi79'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI79 inputs; set HEGEMONY_FROZEN_ABI79_DIR to the exact v61 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI79 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI79 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi79(destination, options = {}) {
  const saved = frozenAbi79(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}
