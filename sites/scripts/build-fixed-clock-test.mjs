// Local HTTP test bundle only. Production index keeps the real server clock.
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import { buildEnvironment } from '../../tools/build-storage.mjs';
import { frozenAbi77, frozenAbi78, frozenAbi79, frozenAbi80, frozenAbi81, frozenAbi82 } from '../../tools/frozen-kernels.mjs';
const repository=fileURLToPath(new URL('../../',import.meta.url));
const sites=path.join(repository,'sites');
const output=path.resolve(process.env.HEGEMONY_FIXED_CLOCK_TEST_ROOT || path.join(process.env.UNDERGROUNDBATTLE_BUILD_ROOT || '/workspace/undergroundbattle-build','jc054-worker'));
const environment=buildEnvironment(repository,[output]);
for(const name of ['hegemony_wasm.js','hegemony_wasm_bg.wasm'])
  assert.deepEqual(readFileSync(path.join(sites,'generated',name)),readFileSync(path.join(repository,'rust-game-wasm/pkg',name)),'Build the fresh production backend before the test wrapper');
for(const [i,load]of [frozenAbi77,frozenAbi78,frozenAbi79,frozenAbi80,frozenAbi81,frozenAbi82].entries()) {
  const saved=load(),directory=path.join(sites,'generated',`abi${77+i}`);
  assert.deepEqual(readFileSync(path.join(directory,'hegemony_wasm.js')),saved.js);
  assert.deepEqual(readFileSync(path.join(directory,'hegemony_wasm_bg.wasm')),saved.wasm);
}
const pkg=await import(pathToFileURL(path.join(repository,'rust-game-wasm/pkg/hegemony_wasm.js')));
pkg.initSync({module:readFileSync(path.join(repository,'rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
assert.equal(JSON.parse(pkg.catalog()).engineVersion,'rust-v0.2.83-msjc04-msjc05-societies-candidate');
mkdirSync(output,{recursive:true});
const relative=file=>'./'+path.relative(output,path.join(sites,'src',file)).split(path.sep).join('/');
writeFileSync(path.join(output,'worker.mjs'),`import {kernel} from ${JSON.stringify(relative('kernel.mjs'))};\nimport {createWorker} from ${JSON.stringify(relative('http.mjs'))};\n// Trusted local test binding only; production index always uses Date.now.\nexport default {fetch(request,env){const now=Number(env.LOCAL_TEST_CLOCK_MS||'0');if(!Number.isSafeInteger(now)||now<0)throw Error('Invalid local test clock');return createWorker(kernel,()=>now).fetch(request,env);}};\n`);
writeFileSync(path.join(output,'package.json'),'{"type":"module","private":true}\n');
writeFileSync(path.join(output,'wrangler.json'),JSON.stringify({name:'jc054-local-clock-test',main:'worker.mjs',compatibility_date:'2026-10-02',d1_databases:[{binding:'DB',database_name:'jc054-http'}]}));
// Wrangler retains unreferenced modules in an existing outdir. Preserve the
// previous bundle whole, then emit into an empty directory for exact inventory.
if(existsSync(path.join(output,'server')))
  renameSync(path.join(output,'server'),path.join(output,'retained-server-'+new Date().toISOString().replaceAll(':','-')));
execFileSync(path.join(sites,'node_modules/.bin/wrangler'),['deploy','--dry-run','--config',path.join(output,'wrangler.json'),'--outdir',path.join(output,'server')],
  {cwd:sites,stdio:'inherit',env:{...environment,XDG_CONFIG_HOME:path.join(output,'.wrangler-config'),WRANGLER_SEND_METRICS:'false'}});
console.log(JSON.stringify({output,clock:0,deployed:false,productionClockUnchanged:true,exactProductionRuntimeInputs:true}));
