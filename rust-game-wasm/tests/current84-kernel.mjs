// Current candidate only. Identity checked before any fixture or route use.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
abi.initSync({module:moduleBytes});
export const kernel=abi,catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.84-jc057-live-inspection-candidate'||catalog.cardPoolVersion!=='limited-v2.76-jc057-live-inspection-candidate'||catalog.cards.length!==148||catalog.societies.length!==10)throw Error('Current ABI84 identity mismatch');
