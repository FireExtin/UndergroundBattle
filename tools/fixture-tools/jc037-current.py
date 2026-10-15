"""Join exact, unchanged current JC037 Native traces; no migration or relabeling."""
from pathlib import Path
import hashlib,json,subprocess,sys
root=Path(sys.argv[1]).resolve()
repo=Path(__file__).resolve().parents[2]
catalog=json.loads(subprocess.check_output(['node','--input-type=module','-e',
    "import {catalog} from './rust-game-wasm/tests/current-kernel.mjs';console.log(JSON.stringify(catalog));"],cwd=repo,text=True))
versions={
    'rust-v0.2.74-jc037-hidden-investigation-candidate':('74','limited-v2.67-jc037-hidden-investigation-candidate','0939eab8f0ac971fc375183315313f4459a4e489'),
    'rust-v0.2.75-jc060-occult-adviser-candidate':('75','limited-v2.68-jc060-occult-adviser-candidate','3d642f0c1e6e5ac021cd40560086df35638bfe4c'),
}
assert catalog['engineVersion'] in versions, 'Build an explicitly supported current ABI first.'
suffix,expected_pool,source_baseline=versions[catalog['engineVersion']]
assert catalog['cardPoolVersion']==expected_pool
scenarios=[]
for path in sorted(root.glob('*.json')):
    raw=path.read_bytes();row=json.loads(raw)
    assert row['catalog']==catalog and row['steps']
    assert row['initial']['state']==row['steps'][0]['state']
    for i,step in enumerate(row['steps']):
        if i: assert row['steps'][i-1]['expected']['state']==step['state']
        assert step['expected']['view']['versions']['engine']==catalog['engineVersion']
    row['provenance']={'nativeFile':path.name,'nativeFileSha256':hashlib.sha256(raw).hexdigest()}
    scenarios.append(row)
assert {s['name'] for s in scenarios}=={'paid-deploy','paid-reveal','secret-deployment','source-control','source-return','source-hide','leader-replace','duel-paid-deploy'}
result={'scope':'Unchanged source-produced current Native Room traces; explicit offline initial layouts, not browser play or save migration.',
        'sourceBaseline':source_baseline,'catalog':catalog,
        'wasmSha256':hashlib.sha256((repo/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'scenarios':scenarios}
payload=json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n'
for name in [f'web/src/game/jc037Native{suffix}.fixture.json',f'sites/test/fixtures/jc037-native-v0{suffix}.json']:
    output=repo/name
    if output.exists():assert output.read_text()==payload, 'Preserve existing inputs and choose new evidence; no overwrite.'
    else:
        with output.open('x') as f:f.write(payload)
print(json.dumps({'scenarios':len(scenarios),'steps':sum(len(s['steps']) for s in scenarios),'wasmSha256':result['wasmSha256']}))
