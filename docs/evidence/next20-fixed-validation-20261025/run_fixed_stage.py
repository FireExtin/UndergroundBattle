import os,sys,json,time,datetime,hashlib,subprocess
from pathlib import Path
base=Path('/home/ddd/Documents/Codex/2026-10-24/task-7');root=base/'validation-fixed-1bd0776-20261025';name=sys.argv[1];variant='post-worker-e853d8df' if name.startswith('post-worker-') else 'baseline-ff3b1c03' if name.startswith('baseline-ff-') else 'post-inventory-ed6d72a0' if name.startswith('post-inventory-') else None;src=(root/variant/'source') if variant else (root/'source');meta=json.loads(((root/variant/'inputs.json') if variant else (root/'inputs.json')).read_text());tools=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/.tools');toolchain=tools/'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin';build=base/'next20-validation-20261025';args=sys.argv[2:];out=root/'logs';out.mkdir(exist_ok=True);log=out/(name+'.log');receipt=out/(name+'.json');assert not log.exists() and not receipt.exists()
if name.startswith('post-final-native-'):
 src=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
 sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=src,text=True).strip()
 assert sha.startswith('927e1819')
 assert not subprocess.check_output(['git','diff','HEAD','--name-only'],cwd=src,text=True).strip()
 meta={'source_commit':sha}
env=dict(os.environ);env.update(PATH=str(toolchain)+':'+str(tools/'node-v24.22.1-linux-x64/bin')+':'+env['PATH'],CARGO_HOME=str(build/'cargo'),CARGO_TARGET_DIR=str(build/'build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(build/'build'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',NPM_CONFIG_OFFLINE='true',NPM_CONFIG_CACHE=str(root/'npm-cache'),WRANGLER_SEND_METRICS='false',CC=str(tools/'batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools/'batch3-zig-cc'),ZIG_LOCAL_CACHE_DIR=str(build/'zig-local'),ZIG_GLOBAL_CACHE_DIR=str(build/'zig-global'),WASM_BINDGEN=str(tools/'wasm-bindgen-0.2.104-from-crate/bin/wasm-bindgen'),XDG_CONFIG_HOME=str(root/'config'),CARGO_TERM_COLOR='never')
if name.startswith('wasm-'):
 env.update(CARGO_TARGET_DIR=str(root/'wasm-build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'wasm-build'))
if name.startswith('baseline-ff-wasm-'):
 env.update(CARGO_TARGET_DIR=str(root/'baseline-ff3b1c03/wasm-build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'baseline-ff3b1c03/wasm-build'))
if name == 'full-worker' or name.startswith('post-worker-'):env['CURRENT_CLOCK_WORKER']=str(root/'current-worker/server/worker.js')
if args[0]=='cargo':args[0]=str(toolchain/'cargo')
cmd=[str(tools/'node-v24.22.1-linux-x64/bin/node'),str(src/'tools/build-storage.mjs'),'--',*args]
os.sched_setaffinity(0,[27,28] if name in ['full-ui','post-inventory-ui'] else [29] if name.startswith('worker-') or name.startswith('post-worker-') or name == 'full-worker' else [30] if name.startswith('wasm-') else [31]);os.nice(19);j={'state':'running','name':name,'source_commit':meta['source_commit'],'cwd':str(src),'command':cmd,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'log':str(log),'scope':('read-only fixed main checkout targeted tests; all compiler outputs in task-7' if name.startswith('post-final-native-') else 'isolated fixed Git archive; offline; no original artifact or table writes')};receipt.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n');start=time.monotonic()
with log.open('xb') as f:p=subprocess.run(cmd,cwd=src,env=env,stdout=f,stderr=subprocess.STDOUT)
j.update(state='complete',exit_code=p.returncode,seconds=round(time.monotonic()-start,3),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest());receipt.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n');print(json.dumps(j,ensure_ascii=False),flush=True);print('\n'.join(log.read_text(errors='replace').splitlines()[-35:]));raise SystemExit(p.returncode)
