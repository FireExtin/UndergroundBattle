"""Join unchanged Native WM056 snapshots and exact accepted commands."""
from pathlib import Path
import hashlib,json,sys
root=Path(sys.argv[1]).resolve()
steps=[(p,json.loads(p.read_text())) for p in sorted((root/'trace').glob('step-*.json'))]
cases=[]
for p in sorted((root/'native').glob('*.json')):
    snap=json.loads(p.read_text())
    matches=[(q,s) for q,s in steps if s['state']==snap['state']
             and s['command']['action'].get('kind')=='game'
             and s['command']['action'].get('action')==snap['action']
             and s['expected']['outcome']=='accepted']
    assert matches,p
    q,s=matches[0]
    cases.append({'name':p.stem,'seat':s['seat'],'before':snap['state'],'beforeViews':snap['views'],
                  'command':s['command'],'expected':s['expected'],'afterViews':s['views'],
                  'provenance':{'snapshotSha256':hashlib.sha256(p.read_bytes()).hexdigest(),
                                'acceptedNativeStepSha256':hashlib.sha256(q.read_bytes()).hexdigest()}})
assert len(cases)==4,len(cases)
out=Path('web/src/game/wm056Native68.fixture.json')
result={'scope':'Explicit offline Native layouts; real paid Room commands, projections and guarded resolution choices. Not browser play or save migration.',
        'sourceBaseline':'3bbd8b5e6d4deaf10a348d68e959abdd8c32dc75',
        'wasmSha256':hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'cases':cases}
out.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'path':str(out),'cases':len(cases),'bytes':out.stat().st_size}))
