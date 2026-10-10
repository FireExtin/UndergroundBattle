// These comparisons execute the current ABI against native evidence from the
// same identity. Older recorded states stay unchanged and cannot be relabelled.
import { readFileSync } from 'node:fs';
const abi = new URL(process.env.HEGEMONY_WASM_TEST_ABI || '../pkg/hegemony_wasm.js', import.meta.url);
export const kernel = await import(abi.href);
export const moduleBytes = readFileSync(new URL('./hegemony_wasm_bg.wasm', abi));
export const instance = kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
