"""Join unchanged Native snapshots with the actual accepted activation command."""
from pathlib import Path
import subprocess
import hashlib,json,sys

root=Path(sys.argv[1]).resolve()
suffix=sys.argv[2] if len(sys.argv)>2 else '67'
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
        'sourceBaseline':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        'wasmSha256':hashlib.sha256(Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()).hexdigest(),'cases':cases}
out=Path(f'web/src/game/jc094Native{suffix}.fixture.json') if len(sys.argv)<4 else Path(sys.argv[3])
with out.open('x') as stream: stream.write(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'cases':len(cases),'path':str(out),'bytes':out.stat().st_size}))
