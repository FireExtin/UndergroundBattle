// These comparisons execute the current ABI against native evidence from the
// same identity. Older recorded states stay unchanged and cannot be relabelled.
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
const abi = new URL(process.env.HEGEMONY_WASM_TEST_ABI || '../pkg/hegemony_wasm.js', import.meta.url);
// Load the ESM binding with Node so Vitest does not rewrite its file URL.
export const kernel = createRequire(import.meta.url)(fileURLToPath(abi));
export const moduleBytes = readFileSync(new URL('./hegemony_wasm_bg.wasm', abi));
export const instance = kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
