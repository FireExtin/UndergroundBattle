// XQ23 candidate ABI88. Original77-87 are separate immutable dependencies.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',import.meta.url));
abi.initSync({module:moduleBytes});
export const kernel=abi,catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.88-xq23-asset-return-candidate'||catalog.cardPoolVersion!=='limited-v2.80-xq23-asset-return-candidate'||catalog.cards.length!==152||catalog.societies.length!==10)throw Error('Current ABI88 identity mismatch');
