// Test-only ABI77 oracle for unchanged historical recorded inputs.
// Candidate ABI78 has separate fresh Native/WASM integration coverage.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { kernel as kernel, moduleBytes as frozen77Bytes } from '../../../rust-game-wasm/tests/legacy77-kernel.mjs';

kernel.initSync({ module: frozen77Bytes });
export { kernel };
export const catalog = JSON.parse(kernel.catalog());
export const definitions = new Map(catalog.cards.map(c => [c.id, c]));
