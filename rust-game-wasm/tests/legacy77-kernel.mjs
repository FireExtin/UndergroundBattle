// Historical ABI77 oracles remain byte-exact; they are not relabelled as ABI78.
import { pathToFileURL } from 'node:url';
import { frozenAbi77 } from '../../tools/frozen-kernels.mjs';
const retained = frozenAbi77();
export const moduleBytes = retained.wasm;
export const kernel = await import(/* @vite-ignore */ pathToFileURL(retained.jsPath).href);
kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
for (const key of ['rulesVersion', 'cardPoolVersion', 'engineVersion']) {
  if (catalog[key] !== retained.identity.catalogIdentity[key]) throw Error('Frozen ABI77 catalog identity mismatch');
}
