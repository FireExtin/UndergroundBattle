// Fresh source-built ABI83 only. Published rooms use their exact frozen loaders.
import {readFileSync} from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
// Keep this as a runtime Node URL when Vitest transforms the module.
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
abi.initSync({module:moduleBytes});
export const kernel=abi;
export const catalog=JSON.parse(abi.catalog());
export const identity=Object.fromEntries(['rulesVersion','cardPoolVersion','engineVersion'].map(k=>[k,catalog[k]]));
if(catalog.engineVersion!=='rust-v0.2.83-msjc04-msjc05-societies-candidate'
  ||catalog.cardPoolVersion!=='limited-v2.75-msjc04-msjc05-societies-candidate'
  ||catalog.cards.length!==147||catalog.societies.length!==10)throw Error('Build the fresh ABI83 candidate before checking it');
