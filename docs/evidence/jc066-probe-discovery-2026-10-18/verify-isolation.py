from pathlib import Path
import datetime,hashlib,json,subprocess,sys
repo=Path('/workspace/UndergroundBattle')
base='a8bf844c30cf988abb25af73e15951849b5fc93a'
product='fa94ec406a9e8163ed9b42756181bd629d3a129f'
oldpacket=repo/'docs/evidence/jc066-p2-2026-10-18'
a=json.loads((oldpacket/'artifacts.json').read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def rec(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def tree(p):return {str(f.relative_to(p)):rec(f) for f in sorted(p.rglob('*')) if f.is_file()}
def git(*args):return subprocess.check_output(['git',*args],cwd=repo)
old=git('show',base+':sites/test/jc066-p2-restore.test.mjs').decode()
probe=repo/'sites/probes/jc066-p2-restore.mjs'
assert probe.read_text()=='// Opt-in probe: excluded from the default test/*.test.mjs discovery.\n'+old.replace("from './assert-session-journal.mjs'","from '../test/assert-session-journal.mjs'")
assert not (repo/'sites/test/jc066-p2-restore.test.mjs').exists()
assert (repo/'sites/package.json').read_bytes()==git('show',base+':sites/package.json')
assert json.loads((repo/'sites/package.json').read_text())['scripts']['test']=='node --test test/*.test.mjs'
for name,h in a['compilerInputs']['files'].items():
 assert rec(repo/name)['sha256']==h,name
 assert sha(git('show',base+':'+name))==h,name
 assert sha(git('show',product+':'+name))==h,name
assert rec(repo/'rust-game-wasm/tests/jc066Native86.fixture.json.gz')==a['native']['fixture']
assert rec(repo/'rust-game-wasm/tests/jc044Native85.fixture.json.gz')==a['prior85Fixture']
assert rec(repo/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm')==a['afterWasm']
assert tree(repo/'sites/dist')==a['production']
assert tree(repo/'sites/generated')==a['generated']
assert tree(Path('/workspace/jc066-p2-evidence-20261018/worker-after/server'))==a['testWorker']
# All earlier natural receipts, logs, source witnesses and artifact bindings remain immutable.
prior=json.loads(git('show',base+':docs/evidence/jc066-p2-2026-10-18/manifest.json'))
for name,v in prior['files'].items():
 if name!='README.md':assert rec(oldpacket/name)==v,name
p=repo/'.git/codex-index-refresh.lock';s=p.stat()
assert (s.st_ino,s.st_size,s.st_mtime_ns)==(1048672,0,1792294521284793620)
result={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'infraBaseCommit':base,'unchangedProductSourceCommit':product,'probeChangeOnly':['relocation outside default test glob','one relative helper import','opt-in header comment'],'packageTestScriptUnchanged':True,'compilerInputCount':len(a['compilerInputs']['files']),'compilerInputsUnchanged':True,'wasm':a['afterWasm'],'own86Fixture':a['native']['fixture'],'prior85Fixture':a['prior85Fixture'],'productionFilesByteIdentical':len(a['production']),'clientFilesByteIdentical':len([n for n in a['production'] if n.startswith('client/')]),'generatedFilesByteIdentical':len(a['generated']),'testWorkerFilesByteIdentical':len(a['testWorker']),'priorEvidenceUnchangedExceptReadmeAndItsManifestEntry':True,'protectedIndexRefreshLockPreserved':True,'rebuilt':False,'published':False}
if len(sys.argv)>1:
 with Path(sys.argv[1]).open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result))
