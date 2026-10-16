// Exact v58 runtime for the pass-history adapter review. No ABI78 rule changes.
import { existsSync, readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { gunzipSync } from 'node:zlib';
import { pathToFileURL, fileURLToPath } from 'node:url';
import path from 'node:path';

const project = fileURLToPath(new URL('../', import.meta.url));
const root = existsSync(project + 'web/package.json') ? project : fileURLToPath(new URL('../../', import.meta.url));
export const publishedIdentity = { rulesVersion: 'hegemony-pdf-v1', cardPoolVersion: 'limited-v2.70-jc046-mandatory-sacrifice-candidate', engineVersion: 'rust-v0.2.77-jc046-mandatory-sacrifice-candidate' };
export const wasmSha256 = 'b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c';
const jsSha256 = 'fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb';
export const sha256 = value => createHash('sha256').update(value).digest('hex');
export function published77Inputs() {
  const retained = path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build', 'frozen-kernels/abi77');
  const directory = process.env.HEGEMONY_PASS_HISTORY_ABI77_DIR || (existsSync(retained) ? retained : path.join(root, 'rust-game-wasm/pkg'));
  const jsPath = path.join(directory, 'hegemony_wasm.js');
  const js = readFileSync(jsPath);
  if (sha256(js) !== jsSha256) throw Error('Pass-history tests require the exact published v58 JS binding');
  const raw = path.join(directory, 'hegemony_wasm_bg.wasm');
  let wasm;
  if (existsSync(raw)) wasm = readFileSync(raw);
  else {
    const compressed = readFileSync(raw + '.gz');
    if (sha256(compressed) !== '2b041aa5daa089e00f3157cfdb4a54c10a6f531bc3527998a43e835c580438ff') throw Error('Published v58 compressed WASM mismatch');
    wasm = gunzipSync(compressed, { maxOutputLength: 2909006 });
  }
  if (sha256(wasm) !== wasmSha256 || wasm.length !== 2909006) throw Error('Pass-history tests require the exact published v58 ABI77 WASM');
  return { js, jsPath, wasm };
}
export async function published77Kernel() {
  const inputs = published77Inputs();
  const abi = await import(pathToFileURL(inputs.jsPath).href);
  abi.initSync({ module: inputs.wasm });
  const catalog = JSON.parse(abi.catalog());
  for (const key of Object.keys(publishedIdentity)) {
    if (catalog[key] !== publishedIdentity[key]) throw Error('Published v58 catalog identity mismatch');
  }
  return abi;
}
