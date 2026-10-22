from pathlib import Path
import os,subprocess,json,time,hashlib,datetime,sys
root=Path(__file__).absolute().parent;repo=Path.cwd();assert repo.name=='UndergroundBattle-batch3'
env=dict(os.environ);cargo=root/'.tools/cargo';env.update(PATH=f'{root}/.tools/node-v24.22.1-linux-x64/bin:{cargo}/bin:'+env['PATH'],CARGO_HOME=str(cargo),RUSTUP_HOME=str(root/'.tools/rustup'),CARGO_NET_OFFLINE='true',WASM_BINDGEN=str(root/'.tools/wasm-bindgen-0.2.104-from-crate/bin/wasm-bindgen'),CC=str(root/'.tools/batch3-zig-cc'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(root/'.tools/batch3-zig-cc'),ZIG_GLOBAL_CACHE_DIR=str(root/'.tools/zig-cache'),ZIG_LOCAL_CACHE_DIR=str(root/'.tools/zig-cache-local'),UNDERGROUNDBATTLE_BUILD_ROOT=str(root/'batch3-build'),RUSTFLAGS=f'--remap-path-prefix={cargo}=/__undergroundbattle/cargo --remap-path-prefix={repo}=/__undergroundbattle/source')
label=sys.argv[1] if len(sys.argv)>1 else '01';assert label.isdigit();log=root/f'batch3-abi94-wasm-build-{label}.log';out=root/f'batch3-abi94-wasm-build-{label}.json';assert not log.exists() and not out.exists()
versions={name:subprocess.check_output(args,env=env,text=True).strip() for name,args in [('rustc',['rustc','-Vv']),('cargo',['cargo','-V']),('node',['node','--version']),('wasmBindgen',[env['WASM_BINDGEN'],'--version'])]}
assert versions['node']=='v24.22.1' and versions['wasmBindgen']=='wasm-bindgen 0.2.104' and 'rustc 1.90.0' in versions['rustc']
start=time.monotonic()
with log.open('w') as f:r=subprocess.run(['bash','rust-game-wasm/build.sh'],env=env,stdout=f,stderr=subprocess.STDOUT)
files=[]
if r.returncode==0:
 for name in ['hegemony_wasm.js','hegemony_wasm.d.ts','hegemony_wasm_bg.wasm','hegemony_wasm_bg.wasm.d.ts','package.json']:
  p=repo/'rust-game-wasm/pkg'/name;b=p.read_bytes();files.append({'path':str(p.relative_to(repo)),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
record={'recordedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':r.returncode,'seconds':round(time.monotonic()-start,3),'versions':versions,'recipe':{'command':['bash','rust-game-wasm/build.sh'],'offline':True,'pathNormalization':env['RUSTFLAGS'],'wasmBindgen':env['WASM_BINDGEN'],'buildRoot':env['UNDERGROUNDBATTLE_BUILD_ROOT'],'guard':'tools/build-storage.mjs original jobs1 / minimum free space guard'},'outputs':files,'limits':['Author local build only; not an independent reproducibility run','Complete twenty-card local candidate; no publication or full batch acceptance']};out.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'exitCode':r.returncode,'seconds':record['seconds'],'outputs':files}));raise SystemExit(r.returncode)
