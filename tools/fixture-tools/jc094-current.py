"""Join unchanged Native snapshots with the actual accepted activation command."""
from pathlib import Path
import hashlib,json,sys

root=Path(sys.argv[1]).resolve()
steps=[(p,json.loads(p.read_text())) for p in sorted((root/'trace').glob('step-*.json'))]
cases=[]
for name in ['public-target','hidden-target','self-target','target-is-cost']:
    p=root/'native'/f'{name}.json'
    snap=json.loads(p.read_text())
    matches=[(q,s) for q,s in steps if s['state']==snap['state']
             and s['command']['action'].get('kind')=='game'
             and s['command']['action'].get('action')==snap['action']
             and s['expected']['outcome']=='accepted']
    assert matches,p
    q,step=matches[0]
    cases.append({'name':name,'seat':step['seat'],'before':snap['state'],
                  'beforeViews':snap['views'],'command':step['command'],'expected':step['expected'],
                  'afterViews':step['views'],'provenance':{
                  'snapshotSha256':hashlib.sha256(p.read_bytes()).hexdigest(),
                  'acceptedNativeStepSha256':hashlib.sha256(q.read_bytes()).hexdigest()}})
result={'scope':'Explicit offline Native layouts and actual accepted Room commands; no browser, saved-room migration or online injection.',
        'sourceBaseline':'3860acd80b6ffe38bfe45bddcdc1ade50e8712f0',
        'wasmSha256':hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'cases':cases}
out=Path('web/src/game/jc094Native67.fixture.json')
out.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'cases':len(cases),'path':str(out),'bytes':out.stat().st_size}))
