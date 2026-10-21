"""Join current Native checkpoints with actual accepted commands, no state edits."""
from pathlib import Path
import subprocess
import hashlib,json,sys

root=Path(sys.argv[1]).resolve()
suffix=sys.argv[2] if len(sys.argv)>2 else '66'
steps=[(p,json.loads(p.read_text())) for p in sorted((root/'trace').glob('step-*.json'))]
cases=[]
for p in sorted((root/'native').glob('*.json')):
    snap=json.loads(p.read_text()); decline='decline' in p.stem
    matches=[(q,s) for q,s in steps if s['state']==snap['state']
             and s['command']['action'].get('kind')=='game'
             and s['command']['action'].get('action',{}).get('kind')=='choose'
             and s['expected']['outcome']=='accepted'
             and s['command']['action']['action']['selected']==snap['selected']]
    assert matches,p
    q,step=matches[0]
    cases.append({'name':p.stem,'seat':step['seat'],'before':snap['state'],
                  'beforeViews':snap['views'],'command':step['command'],'expected':step['expected'],
                  'afterViews':step['views'],'provenance':{'snapshotSha256':hashlib.sha256(p.read_bytes()).hexdigest(),
                  'acceptedNativeStepSha256':hashlib.sha256(q.read_bytes()).hexdigest()}})
assert len(cases)==4,len(cases)
wasm=Path('rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()
result={'scope':'Explicit offline Native layouts and actual accepted Room commands; no browser, saved-room migration or online injection.',
        'sourceBaseline':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        'wasmSha256':hashlib.sha256(wasm).hexdigest(),'cases':cases}
out=Path(f'web/src/game/jc087Native{suffix}.fixture.json') if len(sys.argv)<4 else Path(sys.argv[3])
with out.open('x') as stream: stream.write(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(json.dumps({'cases':len(cases),'path':str(out),'bytes':out.stat().st_size}))
