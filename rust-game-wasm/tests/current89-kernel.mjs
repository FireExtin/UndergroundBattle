// JC035 candidateABI89; original77-88 remain independent frozen reducers.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
abi.initSync({module:moduleBytes});
export const kernel=abi,catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.89-jc035-preparation-candidate'||catalog.cardPoolVersion!=='limited-v2.81-jc035-preparation-candidate'||catalog.cards.length!==153||catalog.societies.length!==10)throw Error('Current ABI89 identity mismatch');
