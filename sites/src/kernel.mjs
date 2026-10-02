import { initSync, catalog, newGame, joinGame, apply, view } from '../generated/hegemony_wasm.js';
import wasmModule from '../generated/hegemony_wasm_bg.wasm';
initSync({ module: wasmModule });
export const kernel = { catalog, newGame, joinGame, apply, view };
