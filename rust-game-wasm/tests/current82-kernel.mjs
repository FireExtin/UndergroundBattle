// Fresh source-built ABI82 only. Historical rooms use their frozen loaders.
import { readFileSync } from 'node:fs';
import * as abi from '../pkg/hegemony_wasm.js';
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',import.meta.url));
abi.initSync({module:moduleBytes});
export const kernel=abi;
export const catalog=JSON.parse(abi.catalog());
if(catalog.engineVersion!=='rust-v0.2.82-response-source-preview-candidate'
  ||catalog.cardPoolVersion!=='limited-v2.74-jc054-complete-conceal-candidate'
  ||catalog.cards.length!==147)throw Error('Build the fresh ABI82 candidate before checking it');
