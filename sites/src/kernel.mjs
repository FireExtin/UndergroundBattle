import * as current from '../generated/hegemony_wasm.js';
import currentModule from '../generated/hegemony_wasm_bg.wasm';
import { routeKernels } from './kernel-router.mjs';
import { lazyKernel } from './lazy-kernel.mjs';
const kernel0 = lazyKernel(current, currentModule, {"rulesVersion":"hegemony-pdf-v1","cardPoolVersion":"limited-v2.57-xq24-riot-police-candidate","engineVersion":"rust-v0.2.64-xq24-riot-police-candidate"});
// Experimental Site: only the current candidate tuple is supported.
export const kernel = routeKernels(kernel0);
