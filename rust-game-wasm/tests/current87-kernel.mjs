// Current JC041 candidate. Published86 and every older core use exact retained bytes.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
abi.initSync({module:moduleBytes});
export const kernel=abi,catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.87-jc041-water-hidden-candidate'||catalog.cardPoolVersion!=='limited-v2.79-jc041-water-hidden-candidate'||catalog.cards.length!==151||catalog.societies.length!==10)throw Error('Current ABI87 identity mismatch');
