import * as current from '../generated/hegemony_wasm.js';
import currentModule from '../generated/hegemony_wasm_bg.wasm';
import { routeKernels } from './kernel-router.mjs';
import { lazyKernel } from './lazy-kernel.mjs';
const kernel0 = lazyKernel(current, currentModule, {"rulesVersion":"hegemony-pdf-v1","cardPoolVersion":"limited-v2.68-jc060-occult-adviser-candidate","engineVersion":"rust-v0.2.75-jc060-occult-adviser-candidate"});
// Experimental Site: only the current candidate tuple is supported.
export const kernel = routeKernels(kernel0);
