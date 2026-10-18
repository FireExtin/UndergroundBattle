// Exact published v65 ABI83. Never read the newer candidate package.
import { createRequire } from 'node:module';
import { frozenAbi83 } from '../../tools/frozen-kernels.mjs';
const saved=frozenAbi83();
export const moduleBytes=saved.wasm;
export const kernel=createRequire(import.meta.url)(saved.jsPath);
kernel.initSync({module:moduleBytes});
export const catalog=JSON.parse(kernel.catalog());
for(const [key,value]of Object.entries(saved.identity.catalogIdentity))
  if(catalog[key]!==value)throw Error('Frozen ABI83 catalog identity mismatch');
if(catalog.cards.length!==147||catalog.societies.length!==10)throw Error('Frozen ABI83 catalog count mismatch');

export const identity=saved.identity.catalogIdentity;
