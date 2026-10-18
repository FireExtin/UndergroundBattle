import os,sys,json,subprocess,datetime,pathlib
root=pathlib.Path(__file__).parent
name=sys.argv[1];cmd=sys.argv[2:]
env=os.environ.copy();env.update(PATH='/workspace/tooling/cargo/bin:'+env['PATH'],CARGO_HOME='/workspace/tooling/cargo',RUSTUP_HOME='/workspace/tooling/rustup',WASM_BINDGEN='/workspace/tooling/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen')
start=datetime.datetime.now(datetime.timezone.utc)
with (root/(name+'.log')).open('x') as log:
 p=subprocess.run(cmd,cwd='/workspace/UndergroundBattle',env=env,stdout=log,stderr=subprocess.STDOUT)
result={'command':cmd,'startedUtc':start.isoformat(),'endedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':p.returncode}
(root/(name+'.exit.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));sys.exit(p.returncode)
