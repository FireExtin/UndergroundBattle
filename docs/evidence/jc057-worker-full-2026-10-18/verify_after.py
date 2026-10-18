from pathlib import Path
import json,hashlib,subprocess,re,datetime
root=Path(__file__).resolve().parent
before=json.loads((root/'before.json').read_text());finished=json.loads((root/'exit.json').read_text())
log=(root/'worker-full-final.log').read_text()
metric=lambda key:int(re.search(r'^ℹ '+key+r' (\d+)$',log,re.M).group(1))
counts={k:metric(k) for k in ['tests','pass','fail','cancelled','skipped','todo']}
assert finished['exitCode']==0 and counts['tests']==counts['pass'] and counts['fail']==counts['cancelled']==counts['skipped']==counts['todo']==0
assert not subprocess.check_output(['git','diff',before['code'],'--','Cargo.toml','Cargo.lock','rust-game','rust-game-wasm','web','sites','tools'])
for row in before['productionFiles']:
 b=(Path('sites/dist')/row['path']).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256'],row['path']
for row in before['fixedClockWorkerFiles']:
 b=Path(row['path']).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256'],row['path']
r=before['originalLock'];p=Path(r['path']);s=p.stat();assert r=={'path':str(p),'bytes':s.st_size,'mtimeNs':str(s.st_mtime_ns),'inode':str(s.st_ino)}
assert hashlib.sha256(Path('docs/evidence/jc057-2026-10-18/runtime-manifest.json').read_bytes()).hexdigest()==before['manifestSha256']
out={**finished,'verifiedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'code':before['code'],'startingHead':before['startingHead'],'endingHead':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'command':before['command'],'counts':counts,'sourcePathsUnchanged':True,'productionFilesUnchanged':len(before['productionFiles']),'fixedClockWorkerFilesUnchanged':len(before['fixedClockWorkerFiles']),'originalLockUnchanged':True,'manifestSha256':before['manifestSha256'],'logSha256':hashlib.sha256((root/'worker-full-final.log').read_bytes()).hexdigest(),'productionCodeFix':False,'websiteAccess':False,'buildOrDeployment':False}
(root/'result.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps(out,ensure_ascii=False))
