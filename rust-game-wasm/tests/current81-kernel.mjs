// Fresh ABI81 only; historical rooms use their separately hash-verified loaders.
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const moduleUrl=import.meta.url;
export const moduleBytes=readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',moduleUrl));
export const kernel=createRequire(import.meta.url)(fileURLToPath(new URL('../pkg/hegemony_wasm.js',moduleUrl)));
kernel.initSync({module:moduleBytes});
export const catalog=JSON.parse(kernel.catalog());
if(catalog.engineVersion!=='rust-v0.2.81-jc054-complete-conceal-candidate'
  ||catalog.cardPoolVersion!=='limited-v2.74-jc054-complete-conceal-candidate'||catalog.cards.length!==147)
  throw Error('Build fresh ABI81 before JC054 checks; retained ABI77–80 cannot substitute for the candidate');
