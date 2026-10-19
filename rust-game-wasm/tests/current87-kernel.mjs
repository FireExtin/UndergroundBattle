// Exact published v69 / ABI87; never substitute a candidate rebuild.
import {frozenAbi87} from '../../tools/frozen-kernels.mjs';
const saved=frozenAbi87();
const abi=await import(new URL('file://'+saved.jsPath));
abi.initSync({module:saved.wasm});
export const kernel=abi,moduleBytes=saved.wasm,catalog=JSON.parse(abi.catalog());
export const identity=saved.identity.catalogIdentity;
if(catalog.engineVersion!==identity.engineVersion||catalog.cardPoolVersion!==identity.cardPoolVersion||catalog.cards.length!==151||catalog.societies.length!==10)throw Error('Retained ABI87 identity mismatch');
