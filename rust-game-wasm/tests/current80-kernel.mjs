// Fresh ABI80 candidate only. Historical ABI77/78/79 have separate strict loaders.
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const moduleUrl = import.meta.url;
export const moduleBytes = readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm', moduleUrl));
export const kernel = createRequire(import.meta.url)(fileURLToPath(new URL('../pkg/hegemony_wasm.js', moduleUrl)));
kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
if (catalog.engineVersion !== 'rust-v0.2.80-wm062-conceal-adjacent-move-candidate'
  || catalog.cardPoolVersion !== 'limited-v2.73-wm062-conceal-adjacent-move-candidate' || catalog.cards.length !== 146) {
  throw Error('Build the fresh ABI80 before WM062 checks; retained ABI77/78/79 cannot stand in for this candidate');
}
