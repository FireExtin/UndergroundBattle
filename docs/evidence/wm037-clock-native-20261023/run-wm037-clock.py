import datetime, hashlib, json, os, resource, subprocess, sys, time
from pathlib import Path
root=Path(__file__).resolve().parent
repo=root/'UndergroundBattle-wm037-clock'
name,*args=sys.argv[1:]
work=root/'wm037-clock-20261023'
work.mkdir(exist_ok=True)
log=work/(name+'.log'); receipt=work/(name+'.json')
assert not log.exists() and not receipt.exists()
env=dict(os.environ)
env.update(PATH=f"{root}/.tools/node-v24.22.1-linux-x64/bin:{root}/.tools/cargo/bin:"+env['PATH'],CARGO_HOME=str(root/'.tools/cargo'),RUSTUP_HOME=str(root/'.tools/rustup'),CC=str(root/'.tools/batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(root/'.tools/batch3-zig-cc'),ZIG_GLOBAL_CACHE_DIR=str(root/'.tools/zig-cache'),ZIG_LOCAL_CACHE_DIR=str(root/'.tools/zig-cache-local'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'batch4-build'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',WRANGLER_SEND_METRICS='false',NPM_CONFIG_OFFLINE='true',XDG_CONFIG_HOME=str(work/'wrangler-config'))
env['HISTORY_SOURCE_COMMIT']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
env['BATCH4_ABI95_NORMAL_OUT']=str(work/'normal-game-result.json')
env['BATCH4_ABI95_NATIVE_OUT']=str(work/'native-witness-abi95.json')
env['HEGEMONY_WASM_TEST_ABI']=(repo/'rust-game-wasm/pkg/hegemony_wasm.js').as_uri()
env['HEGEMONY_BACKEND_TEST_ROOT']=str(work/'worker-backend')
env['CURRENT_CLOCK_WORKER']=str(work/'clock-worker/worker.js')
env['TMPDIR']=str(work/'tmp'); Path(env['TMPDIR']).mkdir(exist_ok=True)
if args[0]=='native':
 command=['node','tools/build-storage.mjs','--','cargo','test','--offline','--locked','-p','hegemony-server',*args[1:]]
 env['INDEPENDENT_HISTORY_OUT']=str(work/(name+'-traces'))
else: command=args
if args[0]=='wasm':
 env['WASM_BINDGEN']=str(root/'.tools/wasm-bindgen-0.2.104-from-crate/bin/wasm-bindgen')
 env['RUSTFLAGS']=f'--remap-path-prefix={root}/.tools/cargo=/__undergroundbattle/cargo --remap-path-prefix={repo}=/__undergroundbattle/source'
 command=['bash','rust-game-wasm/build.sh']
if False:
 recipe=json.loads((repo/'docs/evidence/abi94-recovery-validation-2026-10-22/wasm-build.json').read_text())['recipe']
 for key in ['CARGO_HOME','RUSTUP_HOME','UNDERGROUNDBATTLE_BUILD_ROOT','WASM_BINDGEN','CC','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER','RUSTFLAGS']: env[key]=recipe[key]
 backup=work/('prior-guest-'+name); backup.mkdir(exist_ok=False)
 import shutil
 for p in (repo/'rust-game-wasm/pkg').iterdir():
  if p.is_file(): shutil.copyfile(p,backup/p.name)
run_cwd=repo/'sites' if args[0]=='sites' else repo
if args[0]=='sites': command=args[1:]
os.sched_setaffinity(0,[max(os.sched_getaffinity(0))]); os.nice(19)
if args[0]=='native': resource.setrlimit(resource.RLIMIT_AS,(8*1024**3,8*1024**3))
data=dict(command=command,cwd=str(run_cwd),sourceHead=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),worktree=subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True),startUtc=datetime.datetime.now(datetime.timezone.utc).isoformat(),state='running',affinity=list(os.sched_getaffinity(0)),nativeJobs=1,testThreads=1,virtualAddressLimitGiB=8 if args[0]=='native' else None)
data['sourceFiles']=[dict(path=str(p.relative_to(repo)),sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted([*(repo/'rust-game/src').rglob('*.rs'), *(repo/'rust-game-wasm/src').rglob('*.rs')])]
data['catalogueInputs']=[dict(path=n,sha256=hashlib.sha256((repo/n).read_bytes()).hexdigest()) for n in ['rust-game/data/cards.json','docs/factions/card-specifications.json','docs/factions/batch20-fourth-2026-10-22.json','tools/cards/batch4-source.test.mjs','web/public/cards/JZ56.jpg','web/public/cards/WM037.jpg','web/public/cards/MSJC13.jpg','tools/cards/clockmakers-source.test.mjs']]
receipt.write_text(json.dumps(data,indent=2)+'\n')
start=time.monotonic()
with log.open('x') as out: result=subprocess.run(command,cwd=run_cwd,env=env,stdout=out,stderr=subprocess.STDOUT)
data.update(state='complete',exitCode=result.returncode,seconds=round(time.monotonic()-start,3),logSha256=hashlib.sha256(log.read_bytes()).hexdigest())
receipt.write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps({k:data[k] for k in ['state','exitCode','seconds','logSha256']}))
sys.exit(result.returncode)
