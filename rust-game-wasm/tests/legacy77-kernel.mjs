// Historical ABI77 oracles remain byte-exact; they are not relabelled as ABI78.
import { createRequire } from 'node:module';
import { frozenAbi77 } from '../../tools/frozen-kernels.mjs';
const retained = frozenAbi77();
export const moduleBytes = retained.wasm;
// Let Node load the verified ESM binding directly. Vitest's module runner must
// not reinterpret this external file URL as a Vite virtual module.
export const kernel = createRequire(import.meta.url)(retained.jsPath);
kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
for (const key of ['rulesVersion', 'cardPoolVersion', 'engineVersion']) {
  if (catalog[key] !== retained.identity.catalogIdentity[key]) throw Error('Frozen ABI77 catalog identity mismatch');
}
