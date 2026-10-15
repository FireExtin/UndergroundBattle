"""Join unchanged JC046 Native Room traces for the exact current ABI77."""
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
assert catalog['engineVersion'] == 'rust-v0.2.77-jc046-mandatory-sacrifice-candidate'
assert catalog['cardPoolVersion'] == 'limited-v2.70-jc046-mandatory-sacrifice-candidate'
assert len(catalog['cards']) == 143
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
        assert step['expected']['view']['versions']['cardPool'] == catalog['cardPoolVersion']
    row['provenance'] = {'nativeFile': path.name, 'nativeFileSha256': hashlib.sha256(raw).hexdigest()}
    scenarios.append(row)
# All 37 names are taken from the current jc046_tests.rs Trace::new arguments.
assert {s['name'] for s in scenarios} == {
    'aura-loss-before-next-seat',
    'choose-self-among-others',
    'combat-exhausted',
    'combat-hidden',
    'combat-kill-two',
    'combat-other-region',
    'combat-two-sources',
    'conceal-no-trigger',
    'continuation-container-reference',
    'control-source-sacrifice-restores-next-seat',
    'current-control-owner-graves',
    'death-callbacks-after-all-seats',
    'deploy-no-trigger',
    'duel-seat-zero-first',
    'duplicate-frame-reference',
    'illegal-and-stale-choices',
    'paid-entry-guard-reference',
    'paid-seat0',
    'paid-seat1',
    'paid-seat2',
    'paid-seat3',
    'pending-guard-reference',
    'pre-eliminated-seat-skip',
    'prepared-florence-free-reveal',
    'prepared-granted-glory',
    'prepared-granted-renown',
    'prepared-leader-paused-continuation',
    'prepared-response-source-destroyed',
    'prepared-source-control-change',
    'response-source-bounce',
    'response-source-hide',
    'reveal-extra-target',
    'reveal-low-assets',
    'reveal-no-red-loyalty',
    'sacrifice-bypasses-protection',
    'self-sacrifice-continues',
    'two-real-reveals-independent-frames',
}
assert len(scenarios) == 37
result = {
    'scope': 'Unchanged current Native Room traces from explicit offline layouts; not browser gameplay or save migration.',
    'sourceBaseline': 'dbdee5d6014c260f9833c162bf85aff4b491b690',
    'catalog': catalog,
    'wasmSha256': hashlib.sha256((repo / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),
    'scenarios': scenarios,
}
payload = json.dumps(result, ensure_ascii=False, separators=(',', ':')) + '\n'
for name in ['web/src/game/jc046Native77.fixture.json', 'sites/test/fixtures/jc046-native-v077.json']:
    output = repo / name
    if output.exists():
        assert output.read_text() == payload, 'Preserve existing inputs; do not overwrite different evidence.'
    else:
        with output.open('x') as stream:
            stream.write(payload)
print(json.dumps({'scenarios': len(scenarios), 'steps': sum(len(s['steps']) for s in scenarios),
                  'wasmSha256': result['wasmSha256']}))
