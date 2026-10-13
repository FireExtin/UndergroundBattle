"""Join unchanged Native73 states with complete, actually accepted Room commands.

Run in the current checkout after its real ABI73 build and Native export.
No old state relabeling, payload repair, or baseline ABI fallback is allowed.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
version = 'rust-v0.2.73-jc052-original-region-candidate'
catalog = json.loads(subprocess.check_output([
    'node', '--input-type=module', '-e',
    "import {catalog} from './rust-game-wasm/tests/current-kernel.mjs';"
    "console.log(JSON.stringify({engineVersion:catalog.engineVersion,cardPoolVersion:catalog.cardPoolVersion}));"
], text=True))
assert catalog['engineVersion'] == version, 'Build the actual current JC052 ABI first.'
assert catalog['cardPoolVersion'] == 'limited-v2.66-jc052-original-region-candidate'
steps = [(p, json.loads(p.read_text())) for p in sorted((root / 'trace').glob('step-*.json'))]
cases = []
for p in sorted((root / 'native').glob('*.json')):
    snap = json.loads(p.read_text())
    matches = [(q, s) for q, s in steps
               if s['state'] == snap['state']
               and s['command']['action'].get('kind') == 'game'
               and s['command']['action'].get('action') == snap['action']
               and s['expected']['outcome'] == 'accepted']
    assert matches, f'{p}: no exact accepted Native command; regenerate evidence.'
    q, s = matches[0]
    cases.append({
        'name': p.stem, 'seat': s['seat'], 'before': snap['state'],
        'beforeViews': snap['views'], 'command': s['command'],
        'expected': s['expected'], 'afterViews': s['views'],
        'provenance': {
            'snapshotSha256': hashlib.sha256(p.read_bytes()).hexdigest(),
            'acceptedNativeStepSha256': hashlib.sha256(q.read_bytes()).hexdigest()
        }
    })
assert {c['name'] for c in cases} == {'deploy', 'entry-accept', 'entry-decline', 'leader-sacrifice'}
assert len(cases) == 4
out = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('web/src/game/jc052Native73.fixture.json')
result = {
    'scope': 'Exact prepared Native layouts and accepted Room commands; not browser play or save migration.',
    'sourceBaseline': '9890163a093621f814f9768712e2663de2fd5239',
    'engineVersion': version,
    'wasmSha256': hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),
    'cases': cases
}
payload = json.dumps(result, ensure_ascii=False, separators=(',', ':')) + '\n'
if out.exists():
    assert out.read_text() == payload, 'Existing inputs differ: retain them and use a fresh checkout.'
else:
    with out.open('x') as f:
        f.write(payload)
print(json.dumps({'path': str(out), 'cases': len(cases), 'bytes': out.stat().st_size}))
