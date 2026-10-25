import datetime, hashlib, json, os, signal, subprocess, sys, time
from pathlib import Path
root=Path(__file__).resolve().parent
repo=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
tools=repo.parent/'.tools';build=root.parent/'next20-validation-20261025';name=sys.argv[1];test=sys.argv[2]
run=root/'runs'/name;run.mkdir(parents=True,exist_ok=False)
toolchain=tools/'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin'
env=dict(os.environ);env.update(PATH=str(toolchain)+':'+str(tools/'node-v24.22.1-linux-x64/bin')+':'+env['PATH'],CARGO_HOME=str(build/'cargo'),CARGO_TARGET_DIR=str(build/'build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(build/'build'),CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true',CARGO_TERM_COLOR='never',RUST_TEST_THREADS='1',CC=str(tools/'batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools/'batch3-zig-cc'),ZIG_LOCAL_CACHE_DIR=str(build/'zig-local'),ZIG_GLOBAL_CACHE_DIR=str(build/'zig-global'),NEXT20_NATURAL_OUT_DIR=str(run/'native'))
cmd=[str(tools/'node-v24.22.1-linux-x64/bin/node'),str(repo/'tools/build-storage.mjs'),'--',str(toolchain/'cargo'),'test','--offline','--locked','-p','hegemony-server','--lib',test,'--','--nocapture','--test-threads=1']
sha=lambda b:hashlib.sha256(b).hexdigest()
files=['rust-game/src/deck.rs','rust-game/src/lib.rs','rust-game/src/next20_natural_room_tests.rs']
receipt={'state':'running','at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'HEAD':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'tracked_diff_sha256':sha(subprocess.check_output(['git','diff','HEAD'],cwd=repo)),'source_files':{f:sha((repo/f).read_bytes()) for f in files},'cwd':str(repo),'command':cmd,'test_filter':test,'timeout_seconds':600,'log':str(run/'output.log'),'scope':'Candidate-only three-card construction; genuine deck-to-game commands, no runtime fixture injection'}
for f in files:
 p=run/'source-files'/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes((repo/f).read_bytes())
(run/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');os.sched_setaffinity(0,[30]);os.nice(19);start=time.monotonic()
with (run/'output.log').open('xb') as output:
 process=subprocess.Popen(cmd,cwd=repo,env=env,stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
 try:code=process.wait(timeout=600)
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGTERM)
  try:process.wait(timeout=10)
  except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait()
  code=124
receipt.update(state='complete',exit_code=code,seconds=round(time.monotonic()-start,3),log_sha256=sha((run/'output.log').read_bytes()))
(run/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k] for k in ['HEAD','exit_code','seconds','log','log_sha256']}));print((run/'output.log').read_text()[-9000:]);raise SystemExit(code)
