// Current-kernel local clock wrapper for real HTTP/service/store/D1 tests.
// Never deploy this output or modify sites/generated, dist, or production code.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync,writeFileSync,mkdirSync,copyFileSync,existsSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {catalog,moduleBytes} from '../rust-game-wasm/tests/current-kernel.mjs';
const [deps,abi,output]=process.argv.slice(2);
assert(deps&&abi&&output,'deps abiDirectory NEWoutput required');
const root=fileURLToPath(new URL('../',import.meta.url)),dest=resolve(output),binding=resolve(abi);
assert(!existsSync(dest),'Preserve existing output');
const sha=b=>createHash('sha256').update(b).digest('hex');
const bytes=readFileSync(join(binding,'hegemony_wasm_bg.wasm'));
assert.deepEqual(bytes,moduleBytes,'Only the actual current production module may be wrapped');
assert.equal(catalog.engineVersion,'rust-v0.2.112-shared-keyword-rewards');
const {build}=createRequire(join(resolve(deps),'../package.json'))('esbuild');
mkdirSync(dest);
const source=`import {kernel} from ${JSON.stringify(join(root,'sites/src/kernel.mjs'))};
import {createWorker} from ${JSON.stringify(join(root,'sites/src/http.mjs'))};
export default {fetch(request,env){const now=Number(request.headers.get('X-Local-Clock')||'0');if(!Number.isSafeInteger(now)||now<0)throw Error('bad local clock');return createWorker(kernel,()=>now).fetch(request,env);}};`;
const result=await build({stdin:{contents:source,resolveDir:root,sourcefile:'current-local-only.mjs'},outfile:join(dest,'worker.js'),bundle:true,format:'esm',platform:'browser',target:'es2022',metafile:true,plugins:[{name:'current-only',setup(b){
 b.onResolve({filter:/hegemony_wasm\.js$/},()=>({path:join(binding,'hegemony_wasm.js')}));
 b.onResolve({filter:/\.wasm$/},()=>({path:'./candidate.wasm',external:true}));
}}]});
copyFileSync(join(binding,'hegemony_wasm_bg.wasm'),join(dest,'candidate.wasm'));
writeFileSync(join(dest,'inputs.json'),JSON.stringify({scope:'local clock wrapper; current real kernel/router/http/service/store/D1; never deploy',engineVersion:catalog.engineVersion,wasmSha256:sha(bytes),wrapper:source,metafile:result.metafile},null,2));
console.log(dest);
