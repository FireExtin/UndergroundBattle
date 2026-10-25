import os,sys,json,time,datetime,hashlib,subprocess
from pathlib import Path
base=Path('/home/ddd/Documents/Codex/2026-10-24/task-7');root=base/'candidate-cross-platform-20261025';name=sys.argv[1];args=sys.argv[2:];tools=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/.tools');toolchain=tools/'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin';build=base/'next20-validation-20261025'
main=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
if name.startswith('preflight-'):
 src=root/'preflight-source' if name.startswith('preflight-ui') or name.startswith('preflight-worker') or name.startswith('preflight-pack') else main;meta={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=main,text=True).strip(),'tracked_diff_sha256':hashlib.sha256(subprocess.check_output(['git','diff','HEAD'],cwd=main)).hexdigest(),'precommit_test':True}
 changed=subprocess.check_output(['git','diff','HEAD','--name-only'],cwd=main,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=main,text=True).splitlines()
 meta['edited_source_files']={f:hashlib.sha256((main/f).read_bytes()).hexdigest() for f in sorted(set(changed)) if (main/f).is_file() and Path(f).suffix in ['.rs','.mjs','.jsx','.js','.ts','.tsx','.py']}
else:
 meta=json.loads((root/'frozen-source.json').read_text());src=Path(meta['source'])
out=root/'logs';out.mkdir(exist_ok=True);log=out/(name+'.log');receipt=out/(name+'.json');assert not log.exists() and not receipt.exists()
env=dict(os.environ);env.update(PATH=str(toolchain)+':'+str(tools/'node-v24.22.1-linux-x64/bin')+':'+env['PATH'],CARGO_HOME=str(build/'cargo'),CARGO_TARGET_DIR=str(build/'build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(build/'build'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',NPM_CONFIG_OFFLINE='true',NPM_CONFIG_CACHE=str(root/'npm-cache'),WRANGLER_SEND_METRICS='false',CC=str(tools/'batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools/'batch3-zig-cc'),ZIG_LOCAL_CACHE_DIR=str(build/'zig-local'),ZIG_GLOBAL_CACHE_DIR=str(build/'zig-global'),WASM_BINDGEN=str(tools/'wasm-bindgen-0.2.104-from-crate/bin/wasm-bindgen'),XDG_CONFIG_HOME=str(root/'config'),CARGO_TERM_COLOR='never')

if name.startswith('wasm-'):env.update(CARGO_TARGET_DIR=str(root/'wasm-build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'wasm-build'))
if name.startswith('preflight-native'):env['NEXT20_CANDIDATE_NATIVE_OUT']=str(root/(name+'.native.json'))
if name=='full-native':env.update(json.loads((root/'native-env.json').read_text()))
if name=='full-native-baseline':
 for key in json.loads((root/'native-env.json').read_text()):env.pop(key,None)
 env.update(CARGO_TARGET_DIR=str(root/'baseline-build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'baseline-build'))
if name=='native-export-batch4':env['BATCH4_ABI95_NATIVE_OUT']=str(root/'native/single/batch4_abi95_native_out.json')
if name.startswith('ui-') or name.startswith('worker-') or name.startswith('pack-') or name.startswith('tool-'):
 env['HEGEMONY_CANDIDATE_NATIVE_FIXTURES']=str(root/'fixtures')
if name.startswith('worker-'):env['CURRENT_CLOCK_WORKER']=str(root/'current-worker/server/worker.js')
if name.startswith('preflight-ui') or name.startswith('preflight-worker') or name.startswith('preflight-pack'):
 env['HEGEMONY_CANDIDATE_NATIVE_FIXTURES']=str(root/'preflight-fixtures')
if name.startswith('preflight-worker'):env['CURRENT_CLOCK_WORKER']='/home/ddd/Documents/Codex/2026-10-24/task-7/validation-fixed-1bd0776-20261025/current-worker/server/worker.js'
if args[0]=='cargo':args[0]=str(toolchain/'cargo')
cmd=[str(tools/'node-v24.22.1-linux-x64/bin/node'),str(src/'tools/build-storage.mjs'),'--',*args]
os.sched_setaffinity(0,[27,28] if name.startswith('ui-') else [29] if name.startswith('worker-') else [30] if name.startswith('wasm-') else [26] if name.startswith('pack-') else [25] if name=='full-native-baseline' else [31]);os.nice(19)
j={'state':'running','name':name,**meta,'cwd':str(src),'command':cmd,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'log':str(log),'scope':'candidate evidence; no preserved fixture writes, no public admission'};receipt.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n');started=time.monotonic()
with log.open('xb') as stream:p=subprocess.run(cmd,cwd=src,env=env,stdout=stream,stderr=subprocess.STDOUT)
j.update(state='complete',exit_code=p.returncode,seconds=round(time.monotonic()-started,3),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest());receipt.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:j[k] for k in ['name','state','source_commit','exit_code','seconds','log_sha256','log']}),flush=True);print('\n'.join(log.read_text(errors='replace').splitlines()[-35:]));raise SystemExit(p.returncode)
