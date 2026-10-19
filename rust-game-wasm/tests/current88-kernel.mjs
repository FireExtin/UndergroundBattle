// Exact published v70 / ABI88; never substitute a candidate rebuild.
import {createRequire} from 'node:module';
import {frozenAbi88} from '../../tools/frozen-kernels.mjs';
const saved=frozenAbi88();
const abi=createRequire(import.meta.url)(saved.jsPath);
abi.initSync({module:saved.wasm});
export const kernel=abi,moduleBytes=saved.wasm,catalog=JSON.parse(abi.catalog());
export const identity=saved.identity.catalogIdentity;
if(catalog.engineVersion!==identity.engineVersion||catalog.cardPoolVersion!==identity.cardPoolVersion||catalog.cards.length!==152||catalog.societies.length!==10)throw Error('Retained ABI88 identity mismatch');
