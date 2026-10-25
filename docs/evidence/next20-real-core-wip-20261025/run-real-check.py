#!/usr/bin/env python3
import datetime,hashlib,json,os,subprocess,sys,time
from pathlib import Path
root=Path(__file__).resolve().parent
repo=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
tools=repo.parent/'.tools'
name,kind,*scopes=sys.argv[1:]
work=root/'next20-real-core-validation-20261025';work.mkdir(exist_ok=True)
log=work/(name+'.log');receipt=work/(name+'.json')
assert not log.exists() and not receipt.exists()
toolchain=tools/'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin'
build=root/'next20-validation-20261025'
env=dict(os.environ)
env.update(PATH=str(toolchain)+':'+str(tools/'node-v24.22.1-linux-x64/bin')+':'+env['PATH'],CARGO_HOME=str(build/'cargo'),CARGO_TARGET_DIR=str(build/'build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(build/'build'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',NPM_CONFIG_OFFLINE='true',WRANGLER_SEND_METRICS='false',CC=str(tools/'batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools/'batch3-zig-cc'),ZIG_LOCAL_CACHE_DIR=str(build/'zig-local'),ZIG_GLOBAL_CACHE_DIR=str(build/'zig-global'))
command=[str(tools/'node-v24.22.1-linux-x64/bin/node'),str(repo/'tools/build-storage.mjs'),'--',str(toolchain/'cargo')]
if kind=='test':command+=['test','--offline','--locked','-p','hegemony-server','--lib','--no-default-features',*scopes,'--','--test-threads=1','--nocapture']
else:command+=['check','--offline','--locked','--workspace','--all-targets']
os.sched_setaffinity(0,[31]);os.nice(19)
diff=subprocess.check_output(['git','diff','HEAD'],cwd=repo)
state={'state':'running','source_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'worktree_diff_sha256':hashlib.sha256(diff).hexdigest(),'command':command,'cwd':str(repo),'kind':'real checkout native engine','offline':True,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'log':str(log)}
changed=set(subprocess.check_output(['git','diff','--name-only','HEAD'],cwd=repo,text=True).splitlines())
changed.update(subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=repo,text=True).splitlines())
state['source_inputs']=[{'path':p,'sha256':hashlib.sha256((repo/p).read_bytes()).hexdigest()} for p in sorted(changed) if p.startswith(('rust-game/src/','rust-game/data/')) and (repo/p).is_file()]
receipt.write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n')
start=time.monotonic()
with log.open('xb') as out:q=subprocess.run(command,cwd=repo,env=env,stdout=out,stderr=subprocess.STDOUT)
state.update(state='complete',exit_code=q.returncode,seconds=round(time.monotonic()-start,3),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
receipt.write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:v for k,v in state.items() if k!='source_inputs'},ensure_ascii=False),flush=True)
print('\n'.join(log.read_text(errors='replace').splitlines()[-90:]),flush=True)
raise SystemExit(q.returncode)
