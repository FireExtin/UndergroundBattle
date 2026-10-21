import os, subprocess, pathlib, hashlib, json
base=pathlib.Path('/home/ddd/Documents/Codex/2026-10-21/task-3')
repo=base/'UndergroundBattle'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo).decode().strip()=='bb77eb8ad69738ebecad860d60df05a9276efd00'
assert not subprocess.check_output(['git','status','--porcelain=v1','-z'],cwd=repo)
original=repo/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm'
original_sha=hashlib.sha256(original.read_bytes()).hexdigest()
assert original_sha=='098986aff958d97b5fb5f49339daf07586a200997bf49c179f424ea51dd04e7e'
output=base/'repro-abi93-remap-build/pkg'
output.mkdir(parents=True,exist_ok=True)
subprocess.run(['cargo','build','--offline','--locked','--release','-p','hegemony-wasm','--target','wasm32-unknown-unknown'],cwd=repo,check=True)
subprocess.run([str(base/'.tools/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen'),str(pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'wasm32-unknown-unknown/release/hegemony_wasm.wasm'),'--target','web','--out-dir',str(output),'--out-name','hegemony_wasm'],cwd=repo,check=True)
assert original_sha==hashlib.sha256(original.read_bytes()).hexdigest()
print(json.dumps({'experimentalWasmSha256':hashlib.sha256((output/'hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'bytes':(output/'hegemony_wasm_bg.wasm').stat().st_size,'originalArtifactUnchanged':True,'source':'bb77eb8ad69738ebecad860d60df05a9276efd00','cargoRegistryPathRemap':os.environ['RUSTFLAGS'],'published':False}))
