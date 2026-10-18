from pathlib import Path
import json,hashlib,sys,datetime
repo=Path('/workspace/UndergroundBattle');root=Path(__file__).parent
old=json.loads((repo/'docs/evidence/jc044-2026-10-18/runtime-manifest.json').read_text())
def row(p,base):
 b=p.read_bytes();return {'path':str(p.relative_to(base)),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
source={name:hashlib.sha256((repo/name).read_bytes()).hexdigest() for name in old['sourceSha256']}
prod=repo/'sites/dist';wrapper=root/'worker85-fixed/server'
files=[row(p,prod) for p in sorted(prod.rglob('*')) if p.is_file()]
local=[row(p,wrapper) for p in sorted(wrapper.rglob('*')) if p.is_file()]
assert len(source)==960 and len(files)==179 and len(local)==12,(len(source),len(files),len(local))
result={'sourceSha256':source,'files':files,'localClockWorker':local}
phase=sys.argv[1]
if phase=='before':
 (root/'runtime-before-worker.json').write_text(json.dumps(result,indent=2)+'\n')
else:
 assert result==json.loads((root/'runtime-before-worker.json').read_text()),'Source/output changed during HTTP verification'
 (root/'runtime-after-worker.json').write_text(json.dumps({'checkedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sourceFiles':len(source),'productionFiles':len(files),'wrapperFiles':len(local),'allBytesUnchangedThroughHttpVerification':True},indent=2)+'\n')
print(json.dumps({'phase':phase,'sourceFiles':len(source),'productionFiles':len(files),'wrapperFiles':len(local),'verified':True}))
