// Local-only clock wrapper; share immutable production WASM bytes on ordinary disk.
// New hard links avoid duplicate runtime bytes without deleting old files or using tmpfs.
import {createRequire} from 'node:module';
import {existsSync,readFileSync,readdirSync,mkdirSync,writeFileSync,linkSync} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {buildEnvironment} from './build-storage.mjs';
const repo=fileURLToPath(new URL('../',import.meta.url)),destination=process.argv[2];
if(!destination)throw Error('Provide a NEW local-only Worker directory');
const output=path.resolve(destination),production=path.join(repo,'sites/dist/server');
buildEnvironment(repo,[output]);if(existsSync(output))throw Error('Preserve existing output; choose a new directory');
const sha=b=>createHash('sha256').update(b).digest('hex');
const modules=readdirSync(production).filter(n=>n.endsWith('.wasm')).map(name=>{const file=path.join(production,name),b=readFileSync(file);return{name,file,bytes:b.length,sha256:sha(b)};});
if(modules.length!==1)throw Error('Exactly one current production WASM module required');
const byHash=new Map(modules.map(m=>[m.sha256,m]));if(byHash.size!==1)throw Error('Duplicate production module identity');
const seen=new Set(),{build}=createRequire(path.join(repo,'sites/package.json'))('esbuild');
const source=`import {kernel} from ${JSON.stringify(path.join(repo,'sites/src/kernel.mjs'))};
import {createWorker} from ${JSON.stringify(path.join(repo,'sites/src/http.mjs'))};
export default {fetch(request,env){const now=Number(env.LOCAL_TEST_CLOCK_MS||'0');if(!Number.isSafeInteger(now)||now<0)throw Error('Invalid local test clock');return createWorker(kernel,()=>now).fetch(request,env);}};`;
mkdirSync(output,{recursive:true});
const result=await build({stdin:{contents:source,sourcefile:'current-local-clock.mjs',resolveDir:repo},outfile:path.join(output,'worker.js'),bundle:true,format:'esm',platform:'browser',target:'es2022',conditions:['workerd','worker','browser'],sourcemap:'external',metafile:true,
 plugins:[{name:'exact-production-wasm-links',setup(b){b.onResolve({filter:/\.wasm$/},args=>{const original=path.resolve(args.resolveDir,args.path),hash=sha(readFileSync(original)),m=byHash.get(hash);if(!m)throw Error('Source WASM not present in production manifest: '+original);seen.add(hash);return{path:'./'+m.name,external:true};});}}]});
if(seen.size!==1)throw Error('The exact current source module must be linked');
for(const m of modules)linkSync(m.file,path.join(output,m.name));
writeFileSync(path.join(output,'local-inputs.json'),JSON.stringify({scope:'Local fixed-clock test only, never publish; no production bytes edited.',buildToolSha256:sha(readFileSync(fileURLToPath(import.meta.url))),wrapperSource:source,modules,currentModuleHardLinkedToProduction:true,metafile:result.metafile},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({output,files:readdirSync(output).length,exactWasmModules:1,hardLinksOnlyNoDuplicateWasm:true,productionUnmodified:true}));
