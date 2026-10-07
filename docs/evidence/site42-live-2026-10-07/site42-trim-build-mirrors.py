from pathlib import Path
import hashlib, json, shutil
root=Path('/tmp/lantern-site-fce0-official')
final=root/'dist/client'
expected='9b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885'
wasms=list((root/'dist/server').glob('*.wasm'))
assert len(wasms)==1
assert hashlib.sha256(wasms[0].read_bytes()).hexdigest()==expected
assert hashlib.sha256((root/'generated/hegemony_wasm_bg.wasm').read_bytes()).hexdigest()==expected
source=json.loads(Path('/workspace/game-publication-evidence/site42-source-projection.json').read_bytes())
for rel,entry in source['mappedFiles'].items():
    assert hashlib.sha256((root/rel).read_bytes()).hexdigest()==entry['sha256']
files=[p for p in final.rglob('*') if p.is_file()]
ledger={str(p.relative_to(final)):{'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in files}
removed=[]
for mirror in [root/'web/dist',root/'public']:
    copies=[p for p in mirror.rglob('*') if p.is_file()]
    assert {str(p.relative_to(mirror)) for p in copies}==set(ledger)
    for p in copies:
        assert hashlib.sha256(p.read_bytes()).hexdigest()==ledger[str(p.relative_to(mirror))]['sha256']
    removed.append({'path':str(mirror),'bytes':sum(p.stat().st_size for p in copies)})
    shutil.rmtree(mirror)
Path('/workspace/game-publication-evidence/site42-build-mirror-verification.json').write_text(json.dumps({'distClientFiles':ledger,'verifiedFiles':len(files),'removed':removed,'retained':'complete dist/client and dist/server production output'},indent=2)+'\n')
print(json.dumps({'verifiedClientFiles':len(files),'removedRebuildableMirrorBytes':sum(x['bytes'] for x in removed)}))
