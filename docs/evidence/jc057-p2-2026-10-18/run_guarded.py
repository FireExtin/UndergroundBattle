import datetime,json,os,pathlib,subprocess,sys
root=pathlib.Path('/workspace/jc057-p2-20261018')
name,*command=sys.argv[1:]
record=root/(name+'-run.json')
if record.exists() or (root/(name+'.log')).exists():
 raise SystemExit('Refusing to overwrite run evidence')
env=os.environ.copy()
env.update(PATH='/workspace/tooling/cargo/bin:'+env['PATH'],CARGO_HOME='/workspace/tooling/cargo',RUSTUP_HOME='/workspace/tooling/rustup',WASM_BINDGEN='/workspace/tooling/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen')
started=datetime.datetime.now(datetime.timezone.utc).isoformat()
with (root/(name+'.log')).open('w') as log:
 result=subprocess.run(command,cwd='/workspace/UndergroundBattle',env=env,stdout=log,stderr=subprocess.STDOUT)
data={'name':name,'command':command,'cwd':'/workspace/UndergroundBattle','startedAtUtc':started,'finishedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':result.returncode,'log':name+'.log'}
record.write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(data),flush=True)
raise SystemExit(result.returncode)
