// Exact published v63 ABI81. Never read the newer candidate package.
import { createRequire } from 'node:module';
import { frozenAbi81 } from '../../tools/frozen-kernels.mjs';
const saved=frozenAbi81();
export const moduleBytes=saved.wasm;
export const kernel=createRequire(import.meta.url)(saved.jsPath);
kernel.initSync({module:moduleBytes});
export const catalog=JSON.parse(kernel.catalog());
for(const [key,value]of Object.entries(saved.identity.catalogIdentity))
  if(catalog[key]!==value)throw Error('Frozen ABI81 catalog identity mismatch');
if(catalog.cards.length!==147)throw Error('Frozen ABI81 catalog count mismatch');
