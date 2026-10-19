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

// v62's ABI80 is immutable while JC054 builds a distinct ABI81.
export function frozenAbi80(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi80/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI80_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi80'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI80 inputs; set HEGEMONY_FROZEN_ABI80_DIR to the exact v62 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI80 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI80 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi80(destination, options = {}) {
  const saved = frozenAbi80(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Published v63 includes the independently reviewed JC054 P2 fix. Preserve it
// before adding the separate read-only response preview to a newer engine.
export function frozenAbi81(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi81/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI81_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi81'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI81 inputs; set HEGEMONY_FROZEN_ABI81_DIR to the exact v63 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI81 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI81 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi81(destination, options = {}) {
  const saved = frozenAbi81(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Published v64 ABI82 is retained before adding the red and gray societies.
export function frozenAbi82(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi82/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI82_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi82'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI82 inputs; set HEGEMONY_FROZEN_ABI82_DIR to the exact v64 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI82 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI82 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi82(destination, options = {}) {
  const saved = frozenAbi82(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Published v65 ABI83 is retained before the separate JC057 candidate.
export function frozenAbi83(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi83/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI83_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi83'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI83 inputs; set HEGEMONY_FROZEN_ABI83_DIR to the exact v65 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI83 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI83 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi83(destination, options = {}) {
  const saved = frozenAbi83(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Published v66 retains the reviewed JC057 P2 fix while JC044 uses ABI85.
export function frozenAbi84(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi84/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI84_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi84'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI84 inputs; set HEGEMONY_FROZEN_ABI84_DIR to the exact v66 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI84 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI84 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi84(destination, options = {}) {
  const saved = frozenAbi84(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Published v67 retains the reviewed JC044 P2 fix while JC066 uses ABI86.
export function frozenAbi85(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi85/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI85_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi85'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI85 inputs; set HEGEMONY_FROZEN_ABI85_DIR to the exact v67 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI85 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI85 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi85(destination, options = {}) {
  const saved = frozenAbi85(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Published v68 retains the reviewed JC066 P2 fix while JC041 uses ABI87.
export function frozenAbi86(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi86/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI86_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi86'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI86 inputs; set HEGEMONY_FROZEN_ABI86_DIR to the exact v68 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI86 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI86 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi86(destination, options = {}) {
  const saved = frozenAbi86(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}

// Preserve exact v69 / ABI87 while XQ23 builds a distinct ABI88.
export function frozenAbi87(options = {}) {
  const identity = JSON.parse(readFileSync(path.join(repository, 'rust-game-wasm/frozen/abi87/identity.json'), 'utf8'));
  const directory = path.resolve(options.directory || process.env.HEGEMONY_FROZEN_ABI87_DIR
    || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi87'));
  let js, compressed;
  try { js = readFileSync(path.join(directory, 'hegemony_wasm.js')); compressed = readFileSync(path.join(directory, 'hegemony_wasm_bg.wasm.gz')); }
  catch { throw Error('Missing published ABI87 inputs; set HEGEMONY_FROZEN_ABI87_DIR to the exact v69 runtime. No old-room migration or substitute build is allowed.'); }
  if (digest(js) !== identity.jsSha256 || digest(compressed) !== identity.gzipSha256) throw Error('Published ABI87 runtime hash mismatch; preserve the inputs and stop.');
  const wasm = gunzipSync(compressed, { maxOutputLength: identity.wasmBytes });
  if (wasm.length !== identity.wasmBytes || digest(wasm) !== identity.wasmSha256) throw Error('Published ABI87 runtime hash mismatch; preserve the inputs and stop.');
  return { identity, directory, jsPath: path.join(directory, 'hegemony_wasm.js'), js, wasm };
}
export function stageFrozenAbi87(destination, options = {}) {
  const saved = frozenAbi87(options);
  mkdirSync(destination, { recursive: true });
  writeFileSync(path.join(destination, 'hegemony_wasm.js'), saved.js);
  writeFileSync(path.join(destination, 'hegemony_wasm_bg.wasm'), saved.wasm);
  return saved.identity;
}
