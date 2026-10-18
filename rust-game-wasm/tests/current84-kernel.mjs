// Exact published v66 ABI84, independent of the newer candidate package.
import {createRequire} from 'node:module';
import {frozenAbi84} from '../../tools/frozen-kernels.mjs';
const saved=frozenAbi84();
export const moduleBytes=saved.wasm;
export const kernel=createRequire(import.meta.url)(saved.jsPath);
kernel.initSync({module:moduleBytes});
export const catalog=JSON.parse(kernel.catalog());
export const identity=saved.identity.catalogIdentity;
for(const [key,value]of Object.entries(identity))if(catalog[key]!==value)throw Error('Frozen ABI84 identity mismatch');
if(catalog.cards.length!==148||catalog.societies.length!==10)throw Error('Frozen ABI84 count mismatch');
