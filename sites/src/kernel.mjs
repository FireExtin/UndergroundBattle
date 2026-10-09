import * as current from '../generated/hegemony_wasm.js';
import currentModule from '../generated/hegemony_wasm_bg.wasm';
import { routeKernels } from './kernel-router.mjs';
import { lazyKernel } from './lazy-kernel.mjs';
const kernel0 = lazyKernel(current, currentModule, {"rulesVersion":"hegemony-pdf-v1","cardPoolVersion":"limited-v2.46-deck-seal-search-candidate","engineVersion":"rust-v0.2.51-deck-seal-search-candidate"});
// Experimental Site: only the reviewed current tuple is supported.
export const kernel = routeKernels(kernel0);
