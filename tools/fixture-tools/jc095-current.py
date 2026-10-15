"""Join unchanged JC095 Native Room traces for the exact matching current ABI."""
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
versions = {
    'rust-v0.2.76-jc095-influence-conversion-candidate': ('76', 'limited-v2.69-jc095-influence-conversion-candidate', 'e53e8b017f0ec974c6cf3d8f446b6541771cf4a5'),
    'rust-v0.2.77-jc046-mandatory-sacrifice-candidate': ('77', 'limited-v2.70-jc046-mandatory-sacrifice-candidate', 'dbdee5d6014c260f9833c162bf85aff4b491b690'),
}
assert catalog['engineVersion'] in versions, 'Build an explicitly supported current ABI first.'
suffix, expected_pool, source_baseline = versions[catalog['engineVersion']]
assert catalog['cardPoolVersion'] == expected_pool
assert len(source_baseline) == 40 and all(c in '0123456789abcdef' for c in source_baseline), \
    'Set the exact completed source checkpoint commit before generating the current ABI.'
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
    'sourceBaseline': source_baseline,
    'catalog': catalog,
    'wasmSha256': hashlib.sha256((repo / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),
    'scenarios': scenarios,
}
payload = json.dumps(result, ensure_ascii=False, separators=(',', ':')) + '\n'
for name in [f'web/src/game/jc095Native{suffix}.fixture.json', f'sites/test/fixtures/jc095-native-v0{suffix}.json']:
    output = repo / name
    if output.exists():
        assert output.read_text() == payload, 'Preserve existing inputs; do not overwrite different evidence.'
    else:
        with output.open('x') as stream:
            stream.write(payload)
print(json.dumps({'scenarios': len(scenarios), 'steps': sum(len(s['steps']) for s in scenarios),
                  'wasmSha256': result['wasmSha256']}))
