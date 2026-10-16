"""Verify the complete ordinary QA tree against reviewed source + the exact pass delta.
Uses a temporary Git index, never changes HEAD, branches, the real index or worktrees.
Generated kernels, dependencies and outputs are intentionally outside the tracked tree.
"""
import argparse, hashlib, json, os, subprocess, tempfile
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('--repository',default='/workspace/UndergroundBattle')
parser.add_argument('--qa-source',default='/workspace/pass-context-abi77-review-20261016')
parser.add_argument('--output',default='/workspace/pass-context-evidence-20261016')
args=parser.parse_args()
repo=Path(args.repository).resolve();qa=Path(args.qa_source).resolve();out=Path(args.output).resolve()
reviewed='7ce4cb4d3b1af7bcd5502dc4f96e522d70cabc08'
delta_base='d3406a5d719d2ce748b7935350ec56c0dc65467a'
source='8a27c1ca9cfc065cc03a543d73f14b140ac93515'
def git(*arguments,environment=None,input=None):
    return subprocess.check_output(['git','-C',str(repo),*arguments],env=environment,input=input)
patch=git('diff','--binary',delta_base,source)
out.mkdir(parents=True,exist_ok=True)
(out/'source-feature.patch').write_bytes(patch)
with tempfile.TemporaryDirectory(prefix='pass-overlay-index-',dir=os.environ.get('TMPDIR','/workspace/undergroundbattle-build/tmp')) as temporary:
    environment={**os.environ,'GIT_INDEX_FILE':str(Path(temporary)/'index')}
    git('read-tree',reviewed,environment=environment)
    git('apply','--cached','--whitespace=error-all','-',environment=environment,input=patch)
    tree=git('write-tree',environment=environment).decode().strip()
entries=git('ls-tree','-r','-z',tree).split(b'\0')
files=[]
for entry in entries:
    if not entry:continue
    header,name=entry.split(b'\t',1);mode,kind,blob=header.decode().split()
    assert kind=='blob'
    relative=name.decode();file=qa/relative
    data=os.readlink(file).encode() if mode=='120000' else file.read_bytes()
    actual=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
    assert actual==blob, 'QA source differs from the exact reviewed overlay: '+relative
    files.append({'path':relative,'mode':mode,'gitBlob':blob,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
wasm=(qa/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()
js=(qa/'rust-game-wasm/pkg/hegemony_wasm.js').read_bytes()
assert hashlib.sha256(wasm).hexdigest()=='b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c'
assert hashlib.sha256(js).hexdigest()=='fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb'
manifest={'reviewedBase':reviewed,'progressDeltaBase':delta_base,'featureSourceCommit':source,
    'featurePatchSha256':hashlib.sha256(patch).hexdigest(),'reviewedOverlayTree':tree,
    'entireTrackedQaTreeMatches':True,'trackedFiles':len(files),
    'published77WasmSha256':hashlib.sha256(wasm).hexdigest(),'published77JsSha256':hashlib.sha256(js).hexdigest(),
    'note':'Apply only this delta to the reviewed base. The progress branch includes unfinished ABI78 ancestors and must not be deployed wholesale.',
    'files':files}
(out/'source-overlay-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({key:value for key,value in manifest.items() if key!='files'},ensure_ascii=False,indent=2))
