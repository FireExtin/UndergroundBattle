from pathlib import Path
import json,hashlib,re,datetime
root=Path(__file__).resolve().parent
before=json.loads((root/'affected-before.json').read_text())
run=json.loads((root/'affected-worker-run.json').read_text());assert run['exitCode']==0
log=(root/'affected-worker.log').read_text()
counts={key:int(re.search(r'^ℹ '+key+r' (\d+)$',log,re.M).group(1))for key in ['tests','pass','fail','cancelled','skipped','todo']}
assert counts=={'tests':53,'pass':53,'fail':0,'cancelled':0,'skipped':0,'todo':0},counts
hashof=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for name,hash in before['sourceHashes'].items():assert hashof(Path(name))==hash,name
for group,base in [('productionFiles',Path('sites/dist')),('fixedClockWorkerFiles',Path(before['fixedClockRoot']))]:
 for row in before[group]:
  p=base/row['path'];assert p.stat().st_size==row['bytes'] and hashof(p)==row['sha256'],str(p)
lock=Path('.git/codex-index-refresh.lock');s=lock.stat()
assert before['originalLock']=={'bytes':s.st_size,'mtimeNs':str(s.st_mtime_ns),'inode':str(s.st_ino)}
assert hashof(Path('rust-game-wasm/tests/msjc0405Native83.fixture.json.gz'))=='ee2bc666717e6951dd2c658bfb171c0e00be773f68fbb1a3a2a3d642d0c1a80c'
diags=[json.loads(line[2:])for line in log.splitlines()if line.startswith('ℹ {')]
bad=[d for d in diags if 'invalid'in d];positive=[d for d in diags if 'scenario'in d]
assert len(bad)==24 and all(d['http400']==10 and d['realD1Reopen'] and d['roomCommandsJournalUnchanged']for d in bad)
assert len(positive)==13 and all(d['originalReceiptsAfterReopen']and d['opaqueStatesAndJournalsEqual']for d in positive)
result={'verifiedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'run':run,'counts':counts,'positiveNativeHttpScenarios':len(positive),'corruptedNativeArchives':len(bad),'http400BeforeAfterD1Reopen':sum(d['http400']for d in bad),'malformedRoomCommandsJournalUnchanged':True,'sourceHashesUnchanged':True,'productionFilesUnchanged':177,'fixedClockWorkerFilesUnchanged':9,'client165UnchangedSince2261':True,'frozen77To83Unchanged':True,'frozen83OriginalFixtureUnchanged':True,'originalLockUnchanged':True,'logSha256':hashof(root/'affected-worker.log'),'websiteAccess':False,'deployed':False}
(root/'affected-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
