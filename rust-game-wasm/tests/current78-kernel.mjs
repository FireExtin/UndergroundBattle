// Historical exact v60 ABI78 oracle. Fresh candidate coverage uses current79-kernel.
// Historical ABI78 observations retain their exact original runtime.
import { createRequire } from 'node:module';
import { frozenAbi78 } from '../../tools/frozen-kernels.mjs';
const retained = frozenAbi78();
export const moduleBytes = retained.wasm;
// Let Node load the verified ESM binding directly. Vitest's module runner must
// not reinterpret this external file URL as a Vite virtual module.
export const kernel = createRequire(import.meta.url)(retained.jsPath);
kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
for (const key of ['rulesVersion', 'cardPoolVersion', 'engineVersion']) {
  if (catalog[key] !== retained.identity.catalogIdentity[key]) throw Error('Frozen ABI78 catalog identity mismatch');
}
