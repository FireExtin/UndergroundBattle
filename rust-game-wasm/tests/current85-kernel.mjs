// Current JC044 candidate. Historical84 uses its separately frozen module.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
abi.initSync({module:moduleBytes});
export const kernel=abi,catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.85-jc044-time-repress-candidate'||catalog.cardPoolVersion!=='limited-v2.77-jc044-time-repress-candidate'||catalog.cards.length!==149||catalog.societies.length!==10)throw Error('Current ABI85 identity mismatch');
