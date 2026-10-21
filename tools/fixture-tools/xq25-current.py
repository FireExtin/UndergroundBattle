"""Join exact unchanged Native XQ25 snapshots and accepted Room commands."""
from pathlib import Path
import subprocess
import json,hashlib,sys
root=Path(sys.argv[1]).resolve()
suffix=sys.argv[2] if len(sys.argv)>2 else '70'
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
out=Path(f'web/src/game/xq25Native{suffix}.fixture.json') if len(sys.argv)<4 else Path(sys.argv[3])
result={'scope':'Explicit offline Native layouts and exact accepted Room commands; no browser play, online injection or save migration.',
        'sourceBaseline':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        'wasmSha256':hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'cases':cases}
with out.open('x') as stream: stream.write(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'path':str(out),'cases':len(cases),'bytes':out.stat().st_size}))
