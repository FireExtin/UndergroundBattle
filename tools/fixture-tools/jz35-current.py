"""Join unchanged current Native inputs and accepted commands after its ABI build.

Do not run against baseline Native71 evidence or relabel its opaque states.
"""
from pathlib import Path
import json,hashlib,sys,subprocess,re
root=Path(sys.argv[1]).resolve()
catalog=json.loads(subprocess.check_output(['node','--input-type=module','-e',
    "import {catalog} from './rust-game-wasm/tests/current-kernel.mjs';console.log(JSON.stringify(catalog));"],text=True))
suffix=sys.argv[2] if len(sys.argv)>2 else re.fullmatch(r'rust-v0\.2\.(\d+)-.*',catalog['engineVersion'])[1]
assert suffix.isdigit() and catalog['engineVersion'].startswith(f'rust-v0.2.{suffix}-')
steps=[(p,json.loads(p.read_text())) for p in sorted((root/'trace').glob('step-*.json'))]
cases=[]
for p in sorted((root/'native').glob('*.json')):
    snap=json.loads(p.read_text())
    matches=[(q,s) for q,s in steps if s['state']==snap['state']
             and s['command']['action'].get('kind')=='game'
             and s['command']['action'].get('action')==snap['action'] and s['expected']['outcome']=='accepted']
    assert matches,p
    q,s=matches[0]
    assert snap['views'][s['seat']]['versions']['engine']==catalog['engineVersion'], 'Regenerate current Native inputs; do not relabel old states.'
    # Reject old partial payloads; regenerate Native evidence rather than patch it.
    legal_ids={a.get('abilityId') for a in snap['views'][s['seat']]['legalActions']
               if a.get('kind')=='play' and a.get('cardId')==snap['action'].get('cardId')}
    assert snap['action'].get('abilityId') and legal_ids=={snap['action']['abilityId']}, \
        f'{p}: regenerate Native input with its complete legal abilityId'
    cases.append({'name':p.stem,'seat':s['seat'],'before':snap['state'],'beforeViews':snap['views'],
                  'command':s['command'],'expected':s['expected'],'afterViews':s['views'],
                  'provenance':{'snapshotSha256':hashlib.sha256(p.read_bytes()).hexdigest(),
                                'acceptedNativeStepSha256':hashlib.sha256(q.read_bytes()).hexdigest()}})
assert len(cases)==4,len(cases)
out=Path(f'web/src/game/jz35Native{suffix}.fixture.json')
result={'scope':'Exact current Native prepared layouts and accepted Room commands; not browser play or save migration.',
        'sourceBaseline':'d3035451f3a9375a53f2fd08fbbafd35858e3267',
        'wasmSha256':hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'cases':cases}
out.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'path':str(out),'cases':len(cases),'bytes':out.stat().st_size}))
