import * as current from '../generated/hegemony_wasm.js';
import currentModule from '../generated/hegemony_wasm_bg.wasm';
import * as published77 from '../generated/abi77/hegemony_wasm.js';
import published77Module from '../generated/abi77/hegemony_wasm_bg.wasm';
import { routeKernels } from './kernel-router.mjs';
import { lazyKernel } from './lazy-kernel.mjs';
const kernel0 = lazyKernel(current, currentModule, {"rulesVersion":"hegemony-pdf-v1","cardPoolVersion":"limited-v2.71-bq035-vampire-mobility-candidate","engineVersion":"rust-v0.2.78-bq035-vampire-mobility-candidate"});
const kernel77 = lazyKernel(published77, published77Module, {"rulesVersion":"hegemony-pdf-v1","cardPoolVersion":"limited-v2.70-jc046-mandatory-sacrifice-candidate","engineVersion":"rust-v0.2.77-jc046-mandatory-sacrifice-candidate"});
// Existing v58 rooms retain their original reducer, opaque state and receipts.
export const kernel = routeKernels(kernel0, [kernel77]);
