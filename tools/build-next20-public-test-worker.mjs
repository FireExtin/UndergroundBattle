// Local-only deterministic seed/clock harness. No admission override; never publish.
import {createRequire} from 'node:module';
import {existsSync,readFileSync,readdirSync,mkdirSync,writeFileSync,linkSync} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const root=process.cwd(),destination=path.resolve(process.argv[2]),production=path.join(root,'sites/dist/server');
const {buildEnvironment}=await import(pathToFileURL(path.join(root,'tools/build-storage.mjs')));buildEnvironment(root,[destination]);
if(existsSync(destination))throw Error('Preserve prior test output');
const sha=b=>createHash('sha256').update(b).digest('hex'),modules=readdirSync(production).filter(n=>n.endsWith('.wasm')).map(name=>({name,file:path.join(production,name),sha256:sha(readFileSync(path.join(production,name)))}));assertOne();function assertOne(){if(modules.length!==1)throw Error('One current production WASM required');}
const {build}=createRequire(path.join(root,'sites/package.json'))('esbuild');
const source=`import {kernel} from ${JSON.stringify(path.join(root,'sites/src/kernel.mjs'))};
import {createWorker} from ${JSON.stringify(path.join(root,'sites/src/http.mjs'))};
export default {fetch(request,env){const seed=env.LOCAL_TEST_SEED,now=Number(env.LOCAL_TEST_CLOCK_MS);if(!/^[0-9]+$/.test(seed)||!Number.isSafeInteger(now)||now<0)throw Error('Explicit local seed/clock required');const local={...kernel,newGameWithDeck:(id,invite,mode,name,draft,_randomSeed)=>kernel.newGameWithDeck(id,invite,mode,name,draft,seed)};return createWorker(local,()=>now).fetch(request,env);}};`;
mkdirSync(destination,{recursive:true});let seen=0;
const result=await build({stdin:{contents:source,sourcefile:'next20-local-public-test.mjs',resolveDir:root},outfile:path.join(destination,'worker.js'),bundle:true,format:'esm',platform:'browser',target:'es2022',conditions:['workerd','worker','browser'],sourcemap:'external',metafile:true,plugins:[{name:'exact-production-wasm',setup(b){b.onResolve({filter:/\.wasm$/},args=>{const f=path.resolve(args.resolveDir,args.path),m=modules.find(m=>m.sha256===sha(readFileSync(f)));if(!m)throw Error('WASM differs from production');seen++;return{path:'./'+m.name,external:true};});}}]});
if(seen!==1)throw Error('One exact WASM import required');for(const m of modules)linkSync(m.file,path.join(destination,m.name));
writeFileSync(path.join(destination,'local-inputs.json'),JSON.stringify({scope:'Local test only, never publish',seedOnlyAtNewGame:true,admissionOverride:false,roomStateInjection:false,source,modules,metafile:result.metafile},null,2)+'\n',{flag:'wx'});console.log(JSON.stringify({destination,modules,productionUnmodified:true}));
