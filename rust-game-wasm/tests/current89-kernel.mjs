// Exact published v71 / ABI89; never substitute a candidate rebuild.
import {createRequire} from 'node:module';
import {frozenAbi89} from '../../tools/frozen-kernels.mjs';
const saved=frozenAbi89();
const abi=createRequire(import.meta.url)(saved.jsPath);
abi.initSync({module:saved.wasm});
export const kernel=abi,moduleBytes=saved.wasm,catalog=JSON.parse(abi.catalog());
export const identity=saved.identity.catalogIdentity;
if(catalog.engineVersion!==identity.engineVersion||catalog.cardPoolVersion!==identity.cardPoolVersion||catalog.cards.length!==153||catalog.societies.length!==10)throw Error('Retained ABI89 identity mismatch');
