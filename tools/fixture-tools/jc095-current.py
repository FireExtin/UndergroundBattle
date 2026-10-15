"""Join unchanged JC095 Native Room traces for the exact current ABI76."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

root = Path(sys.argv[1]).resolve(strict=True)
repo = Path(__file__).resolve().parents[2]
catalog = json.loads(subprocess.check_output([
    'node', '--input-type=module', '-e',
    "import {catalog} from './rust-game-wasm/tests/current-kernel.mjs';console.log(JSON.stringify(catalog));"
], cwd=repo, text=True))
assert catalog['engineVersion'] == 'rust-v0.2.76-jc095-influence-conversion-candidate'
assert catalog['cardPoolVersion'] == 'limited-v2.69-jc095-influence-conversion-candidate'
scenarios = []
for path in sorted(root.glob('*.json')):
    raw = path.read_bytes()
    row = json.loads(raw)
    assert row['catalog'] == catalog and row['steps']
    assert row['initial']['state'] == row['steps'][0]['state']
    for i, step in enumerate(row['steps']):
        if i:
            assert row['steps'][i - 1]['expected']['state'] == step['state']
        assert step['expected']['view']['versions']['engine'] == catalog['engineVersion']
    row['provenance'] = {'nativeFile': path.name, 'nativeFileSha256': hashlib.sha256(raw).hexdigest()}
    scenarios.append(row)
assert {s['name'] for s in scenarios} == {
    'paid-deploy-accept', 'paid-deploy-decline', 'secret-reveal',
    'response-increase', 'response-zero', 'source-return', 'source-hide',
    'leader-sacrifice-new', 'renown-team', 'win-window-fixed-winner', 'duel-paid-deploy',
}
assert len({s['name'] for s in scenarios}) == len(scenarios)
result = {
    'scope': 'Unchanged current Native Room traces from explicit offline layouts; not browser gameplay or save migration.',
    'sourceBaseline': 'e53e8b017f0ec974c6cf3d8f446b6541771cf4a5',
    'catalog': catalog,
    'wasmSha256': hashlib.sha256((repo / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),
    'scenarios': scenarios,
}
payload = json.dumps(result, ensure_ascii=False, separators=(',', ':')) + '\n'
for name in ['web/src/game/jc095Native76.fixture.json', 'sites/test/fixtures/jc095-native-v076.json']:
    output = repo / name
    if output.exists():
        assert output.read_text() == payload, 'Preserve existing inputs; do not overwrite different evidence.'
    else:
        with output.open('x') as stream:
            stream.write(payload)
print(json.dumps({'scenarios': len(scenarios), 'steps': sum(len(s['steps']) for s in scenarios),
                  'wasmSha256': result['wasmSha256']}))
