import os,sys,subprocess,json,datetime,hashlib,time
from pathlib import Path
root=Path(__file__).resolve().parent; repo=root/os.environ.get('TEST_REPO','UndergroundBattle-wm043'); work=root/'verification';work.mkdir(exist_ok=True)
name,fil=sys.argv[1:]; log=work/(name+'.log'); receipt=work/(name+'.json'); assert not log.exists()
tools=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/.tools')
env=dict(os.environ);env.update(CARGO_HOME=str(root/'build-cargo-home'),RUSTUP_HOME=str(tools/'rustup'),CARGO_TARGET_DIR=str(root/'build-target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',RUSTUP_AUTO_INSTALL='0',CC=str(tools/'batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools/'batch3-zig-cc'),ZIG_GLOBAL_CACHE_DIR=str(root/'zig-global'),ZIG_LOCAL_CACHE_DIR=str(root/'zig-local'),PATH=str(tools/'cargo/bin')+':'+env['PATH'])
cmd=[str(tools/'cargo/bin/cargo'),'test','--offline','--locked','-p','hegemony-server','--lib',fil,'--','--test-threads=1']
os.sched_setaffinity(0,[30]);os.nice(19)
import resource
resource.setrlimit(resource.RLIMIT_AS,(8*1024**3,8*1024**3))
subprocess.run(["ionice","-c","3","-p",str(os.getpid())],check=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
d={'command':cmd,'sourceHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'startUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'jobs':1,'testThreads':1,'affinity':list(os.sched_getaffinity(0)),'sourceFiles':[{'path':str(p.relative_to(repo)),'sha256':sha(p)} for p in sorted(list((repo/'rust-game/src').rglob('*.rs'))+[repo/'rust-game/data/cards.json',repo/'Cargo.lock',repo/'Cargo.toml',repo/'rust-game/Cargo.toml'])]};receipt.write_text(json.dumps(d,indent=2))
start=time.monotonic()
with log.open('x') as out:r=subprocess.run(cmd,cwd=repo,env=env,stdout=out,stderr=subprocess.STDOUT)
d.update(exitCode=r.returncode,seconds=round(time.monotonic()-start,3),logSha256=sha(log));receipt.write_text(json.dumps(d,indent=2));print(json.dumps({k:d[k] for k in ['exitCode','seconds','logSha256']}))

sys.exit(r.returncode)
