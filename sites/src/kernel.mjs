import * as current from '../generated/hegemony_wasm.js';
import currentModule from '../generated/hegemony_wasm_bg.wasm';
import { routeKernels } from './kernel-router.mjs';

// Only the current Rust core is shipped. Its catalog is the identity authority.
current.initSync({ module: currentModule });
export const kernel = routeKernels(current);
