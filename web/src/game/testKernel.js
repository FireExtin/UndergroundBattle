// Test-only: the one current WASM kernel (built by rust-game-wasm/build.sh).
// Historical engines are not kept; recorded fixtures carry their own views.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';

kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
export { kernel };
export const catalog = JSON.parse(kernel.catalog());
export const definitions = new Map(catalog.cards.map(c => [c.id, c]));
