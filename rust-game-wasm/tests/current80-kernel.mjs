// Exact published v62 ABI80; never read the newly built candidate package.
import { createRequire } from 'node:module';
import { frozenAbi80 } from '../../tools/frozen-kernels.mjs';
const saved = frozenAbi80();
export const moduleBytes = saved.wasm;
export const kernel = createRequire(import.meta.url)(saved.jsPath);
kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
for (const [key, value] of Object.entries(saved.identity.catalogIdentity)) {
  if (catalog[key] !== value) throw Error('Frozen ABI80 catalog identity mismatch');
}
if (catalog.cards.length !== 146) throw Error('Frozen ABI80 catalog count mismatch');
