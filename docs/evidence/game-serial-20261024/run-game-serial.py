import datetime, hashlib, json, os, resource, subprocess, sys, time
from pathlib import Path
root=Path(__file__).resolve().parent
repo=root/'UndergroundBattle-game-serial'
name,kind,*args=sys.argv[1:]
work=root/'serial-intake-20261024/verification';work.mkdir(parents=True,exist_ok=True)
log=work/(name+'.log');receipt=work/(name+'.json')
assert not log.exists() and not receipt.exists(), 'Inspect prior result instead of repeating'
env=dict(os.environ)
env.update(PATH=f"{root}/.tools/node-v24.22.1-linux-x64/bin:{root}/.tools/cargo/bin:"+env['PATH'],CARGO_HOME=str(root/'.tools/cargo'),RUSTUP_HOME=str(root/'.tools/rustup'),CC=str(root/'.tools/batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(root/'.tools/batch3-zig-cc'),ZIG_GLOBAL_CACHE_DIR=str(root/'.tools/zig-cache'),ZIG_LOCAL_CACHE_DIR=str(root/'.tools/zig-cache-local'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'batch4-build'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',WRANGLER_SEND_METRICS='false',NPM_CONFIG_OFFLINE='true')
env['INDEPENDENT_HISTORY_OUT']=str(work/(name+'-normal-traces'))
if kind=='native':command=['node','tools/build-storage.mjs','--','cargo','test','--offline','--locked','-p','hegemony-server','--lib','--',*args,'--nocapture']
else:command=args
run_cwd=repo/'web' if kind=='ui' else repo
os.sched_setaffinity(0,[31]);os.nice(19)
if kind=='native':resource.setrlimit(resource.RLIMIT_AS,(8*1024**3,8*1024**3))
tracked=subprocess.check_output(['git','ls-files','rust-game/src','rust-game-wasm/src','tools/cards','rust-game/data/cards.json','docs/factions/batch20-fourth-2026-10-22.json','web/src/game/AutoPass.tsx','web/src/game/ContinuousPass.test.tsx'],cwd=repo,text=True).splitlines()
data={'command':command,'cwd':str(run_cwd),'sourceHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'worktree':subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True),'startUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'state':'running','affinity':[31],'jobs':1,'testThreads':1,'kind':kind,'inputs':[{'path':n,'sha256':hashlib.sha256((repo/n).read_bytes()).hexdigest()} for n in tracked if (repo/n).is_file()]}
receipt.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
start=time.monotonic()
with log.open('x') as f:q=subprocess.run(command,cwd=run_cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
data.update(state='complete',exitCode=q.returncode,seconds=round(time.monotonic()-start,3),logSha256=hashlib.sha256(log.read_bytes()).hexdigest())
receipt.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:data[k] for k in ['sourceHead','state','exitCode','seconds','logSha256']}))
print(log.read_text()[-3200:])
sys.exit(q.returncode)
