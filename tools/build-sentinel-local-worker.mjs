// Explicit local-only output; never run deploy or write sites/generated or dist.
import {createRequire} from 'node:module';
import {readFileSync,writeFileSync,mkdirSync,copyFileSync,existsSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [deps,abi,output]=process.argv.slice(2);if(!deps||!abi||!output)throw Error('deps abiDirectory NEWoutput required');
const root=fileURLToPath(new URL('../',import.meta.url)),dest=resolve(output),binding=resolve(abi);
if(existsSync(dest))throw Error('Preserve existing output');
const sha=b=>createHash('sha256').update(b).digest('hex'),bytes=readFileSync(join(binding,'hegemony_wasm_bg.wasm'));
if(sha(bytes)!=='0ac2657cb7ca930c910a909e728f62eb893d104f8f6dfe29856a7668beb4c2ce')throw Error('Unexpected isolated candidate WASM');
const {build}=createRequire(join(resolve(deps),'../package.json'))('esbuild');mkdirSync(dest);
const source=`import {kernel} from ${JSON.stringify(join(root,'sites/src/kernel.mjs'))};
import {createWorker} from ${JSON.stringify(join(root,'sites/src/http.mjs'))};
export default {fetch(request,env){const now=Number(request.headers.get('X-Local-Clock')||'0');if(!Number.isSafeInteger(now)||now<0)throw Error('bad local clock');return createWorker(kernel,()=>now).fetch(request,env);}};`;
const result=await build({stdin:{contents:source,resolveDir:root,sourcefile:'sentinel-local-only.mjs'},outfile:join(dest,'worker.js'),bundle:true,format:'esm',platform:'browser',target:'es2022',metafile:true,plugins:[{name:'isolated-candidate',setup(b){
 b.onResolve({filter:/hegemony_wasm\.js$/},()=>({path:join(binding,'hegemony_wasm.js')}));
 b.onResolve({filter:/\.wasm$/},()=>({path:'./candidate.wasm',external:true}));
}}]});
copyFileSync(join(binding,'hegemony_wasm_bg.wasm'),join(dest,'candidate.wasm'));
writeFileSync(join(dest,'inputs.json'),JSON.stringify({scope:'local clock wrapper; real kernel/router/http/service/store/D1; never deploy',wasmSha256:sha(bytes),wrapper:source,metafile:result.metafile},null,2));
console.log(dest);
