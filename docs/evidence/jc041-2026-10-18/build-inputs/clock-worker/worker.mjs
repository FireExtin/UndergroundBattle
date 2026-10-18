import {kernel} from "./../../UndergroundBattle/sites/src/kernel.mjs";
import {createWorker} from "./../../UndergroundBattle/sites/src/http.mjs";
// Trusted local test binding only; production index always uses Date.now.
export default {fetch(request,env){const now=Number(env.LOCAL_TEST_CLOCK_MS||'0');if(!Number.isSafeInteger(now)||now<0)throw Error('Invalid local test clock');return createWorker(kernel,()=>now).fetch(request,env);}};
