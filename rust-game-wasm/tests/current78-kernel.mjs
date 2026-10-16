// This loader is exclusively the freshly built ABI78, never a historical oracle.
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const moduleUrl = import.meta.url;
export const moduleBytes = readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm', moduleUrl));
export const kernel = createRequire(import.meta.url)(fileURLToPath(new URL('../pkg/hegemony_wasm.js', moduleUrl)));
kernel.initSync({ module: moduleBytes });
export const catalog = JSON.parse(kernel.catalog());
if (catalog.engineVersion !== 'rust-v0.2.78-bq035-vampire-mobility-candidate'
  || catalog.cardPoolVersion !== 'limited-v2.71-bq035-vampire-mobility-candidate' || catalog.cards.length !== 144) {
  throw Error('Build the current ABI78 before running BQ035 tests; a retained ABI77 cannot stand in for this candidate');
}
