import datetime,hashlib,json,os,subprocess,sys,time,signal
from pathlib import Path
root=Path(__file__).resolve().parent;name=sys.argv[1];args=sys.argv[2:]
pin=json.loads((root/'frozen-source.json').read_text());source=Path(pin['source']);tools=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/.tools');build=root.parent/'next20-validation-20261025';toolchain=tools/'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin'
out=root/'cross-logs';out.mkdir(exist_ok=True);log=out/(name+'.log');receipt=out/(name+'.json');assert not log.exists() and not receipt.exists()
env=dict(os.environ);env.update(PATH=str(toolchain)+':'+str(tools/'node-v24.22.1-linux-x64/bin')+':'+env['PATH'],CARGO_HOME=str(build/'cargo'),CARGO_TARGET_DIR=str(root/'cross-build/target'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'cross-build'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',NPM_CONFIG_OFFLINE='true',NPM_CONFIG_CACHE=str(root/'npm-cache'),WRANGLER_SEND_METRICS='false',CC=str(tools/'batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools/'batch3-zig-cc'),ZIG_LOCAL_CACHE_DIR=str(build/'zig-local'),ZIG_GLOBAL_CACHE_DIR=str(build/'zig-global'),WASM_BINDGEN=str(tools/'wasm-bindgen-0.2.104-from-crate/bin/wasm-bindgen'),CARGO_TERM_COLOR='never',HEGEMONY_CANDIDATE_NATIVE_FIXTURES=str(root/'fixtures'))
env['XDG_CONFIG_HOME']=str(root/'config')
cmd=[str(tools/'node-v24.22.1-linux-x64/bin/node'),str(source/'tools/build-storage.mjs'),'--',*args]
os.sched_setaffinity(0,[29] if name.startswith('worker') else [28]);os.nice(19)
d={'state':'running','name':name,**pin,'cwd':str(source),'command':cmd,'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'log':str(log),'timeout_seconds':600};receipt.write_text(json.dumps(d,indent=2)+'\n');start=time.monotonic()
with log.open('xb') as output:
 p=subprocess.Popen(cmd,cwd=source,env=env,stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
 try:code=p.wait(timeout=600)
 except subprocess.TimeoutExpired:
  os.killpg(p.pid,signal.SIGTERM)
  try:p.wait(timeout=10)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  code=124
d.update(state='complete',exit_code=code,seconds=round(time.monotonic()-start,3),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest());receipt.write_text(json.dumps(d,indent=2)+'\n');print(json.dumps({k:d[k] for k in ['name','source_commit','exit_code','seconds','log_sha256']}));print(log.read_text()[-6500:]);raise SystemExit(code)
