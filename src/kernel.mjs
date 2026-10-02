import * as current from '../generated/hegemony_wasm.js';
import * as previous from '../generated/legacy-v0.2.1/hegemony_wasm.js';
import currentModule from '../generated/hegemony_wasm_bg.wasm';
import previousModule from '../generated/legacy-v0.2.1/hegemony_wasm_bg.wasm';
import { routeKernels } from './kernel-router.mjs';
current.initSync({ module: currentModule });
previous.initSync({ module: previousModule });
export const kernel = routeKernels(current, previous);
