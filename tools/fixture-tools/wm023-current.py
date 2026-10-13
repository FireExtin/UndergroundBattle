"""Join exact unchanged Native WM023 snapshots and accepted Room commands."""
from pathlib import Path
import json,hashlib,sys
root=Path(sys.argv[1]).resolve()
steps=[(p,json.loads(p.read_text())) for p in sorted((root/'trace').glob('step-*.json'))]
cases=[]
for p in sorted((root/'native').glob('*.json')):
    snap=json.loads(p.read_text())
    matches=[(q,s) for q,s in steps if s['state']==snap['state']
             and s['command']['action'].get('kind')=='game'
             and s['command']['action'].get('action')==snap['action'] and s['expected']['outcome']=='accepted']
    assert matches,p
    q,s=matches[0]
    cases.append({'name':p.stem,'seat':s['seat'],'before':snap['state'],'beforeViews':snap['views'],
                  'command':s['command'],'expected':s['expected'],'afterViews':s['views'],
                  'provenance':{'snapshotSha256':hashlib.sha256(p.read_bytes()).hexdigest(),
                                'acceptedNativeStepSha256':hashlib.sha256(q.read_bytes()).hexdigest()}})
assert len(cases)==4,len(cases)
out=Path('web/src/game/wm023Native69.fixture.json')
result={'scope':'Explicit offline Native layouts and exact accepted Room commands; no browser play, online injection or save migration.',
        'sourceBaseline':'a967e134a5473e2249142ddf4791691aba521051',
        'wasmSha256':hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'cases':cases}
out.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'path':str(out),'cases':len(cases),'bytes':out.stat().st_size}))
