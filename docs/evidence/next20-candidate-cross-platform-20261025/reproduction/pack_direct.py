from pathlib import Path
import json,subprocess,sys
root=Path(__file__).resolve().parent;env=json.loads((root/'native-env.json').read_text());source=Path(json.loads((root/'frozen-source.json').read_text())['source']);wasm=source/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm';out=root/'fixtures/rust-game-wasm/tests'
phase=sys.argv[1]
rows=[
 ('jc035','JC035_NATIVE_DIR','Native89.json','jc035Native112.fixture.json.gz'),
 ('jc041','JC041_NATIVE_DIR','Native87.json','jc041Native112.fixture.json.gz'),
 ('jc044','JC044_NATIVE_DIR','native85.json','jc044Native112.fixture.json.gz'),
 ('jc054','JC054_NATIVE_DIR','native81.json','jc054Native112.fixture.json.gz'),
 ('jc057','JC057_NATIVE_DIR','native84.json','jc057Native112.fixture.json.gz'),
 ('jc066','JC066_NATIVE_DIR','native86.json','jc066Native112.fixture.json.gz'),
 ('msjc0405','MSJC0405_NATIVE_DIR','native83.json','msjc0405Native112.fixture.json.gz')
] if phase=='1' else [('response-preview','RESPONSE_PREVIEW_NATIVE_DIR','native82.json','responsePreviewNative112.fixture.json.gz'),('xq23','XQ23_CURRENT_CORE_NATIVE_DIR','NativeCurrent89.json','xq23CoreNative112.fixture.json.gz'),('xq23-current-owner','XQ23_CURRENT_NATIVE_DIR','NativeCurrentOwner89.json','xq23OwnerNative112.fixture.json.gz')]
def run(name,args):
 assert not (root/'logs'/('pack-'+name+'.json')).exists()
 result=subprocess.run([sys.executable,str(root/'run_stage.py'),'pack-'+name,*map(str,args)])
 if result.returncode:raise SystemExit(result.returncode)
for name,key,file,output in rows:run(name,['node','tools/'+name+'-oracle.mjs',Path(env[key])/file,wasm,out/output])
if phase=='1':
 run('jc041-p2',['node','tools/jc041-oracle.mjs','--p2',Path(env['JC041_P2_DIR'])/'p2.json',wasm,out/'jc041P2Native112.fixture.json.gz'])
 run('jz46',['node','tools/jz46-oracle.mjs',env['JZ46_NATIVE_DIR'],env['JZ46_INVALID_DIR'],out/'jz46Native112.fixture.json.gz'])
else:
 run('wm062',['node','tools/wm062-oracle.mjs',env['WM062_NATIVE_DIR'],env['WM062_NEGATIVE_DIR'],out/'wm062Native112.fixture.json.gz'])
print('DIRECT_PHASE_COMPLETE',phase,flush=True)
