// Current JC066 candidate; published85 uses its separately frozen module.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
abi.initSync({module:moduleBytes});
export const kernel=abi,catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.86-jc066-sealed-icons-candidate'||catalog.cardPoolVersion!=='limited-v2.78-jc066-sealed-icons-candidate'||catalog.cards.length!==150||catalog.societies.length!==10)throw Error('Current ABI86 identity mismatch');
