from pathlib import Path
import base64,datetime,gzip,hashlib,json,os,shutil,subprocess,urllib.request

repo=Path('/workspace/UndergroundBattle')
out=Path('/workspace/storage-recovery-audit-20261018')
main='224a619459d64eaead06810b679ff57b1de94fd6'
def sha(b):return hashlib.sha256(b).hexdigest()
def dump(n,v):
 p=out/n
 assert not p.exists(),n
 p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def disk():
 s=os.statvfs('/workspace')
 return {'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'availableBytes':s.f_bavail*s.f_frsize,'totalFreeBytes':s.f_bfree*s.f_frsize,'guardBytes':1073741824,'deficitBytes':max(0,1073741824-s.f_bavail*s.f_frsize)}
def git(*args):return subprocess.check_output(['git',*args],cwd=repo)

before={'checkedUtc':'2026-10-18T15:35:00.903626+00:00','availableBytes':994414592,'guardBytes':1073741824,'deficitBytes':79327232}
remote=json.loads((out/'npm-cache-recovery-candidates.json').read_text())['remoteReadOnlyProof']
selected=json.loads((out/'npm-cache-recovery-candidates.json').read_text())['selected']
unreadable=json.loads((out/'process-and-recovery.json').read_text())['unreadable']
stage=Path('/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114')
bound=json.loads((repo/'docs/evidence/bq035-p2-2026-10-16/staged-source-files.json').read_text());assert bound['sourceCommit']=='53b2114a220ca001ca8c40ff3b2a6f2b6276ef45'
assert subprocess.run(['git','merge-base','--is-ancestor',bound['sourceCommit'],main],cwd=repo).returncode==0
files=bound['files'];allowed={'.gitignore','source-provenance.json'};seen={str(p.relative_to(stage)) for p in stage.rglob('*') if p.is_file()};assert seen=={r['target'] for r in files}|allowed
proc=subprocess.Popen(['git','cat-file','--batch'],cwd=repo,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
tracked=0;generated=[]
for row in files:
 b=(stage/row['target']).read_bytes();assert len(b)==row['bytes'] and sha(b)==row['sha256'],row['target']
 if row['source'] in ['rust-game-wasm/pkg/hegemony_wasm.js','rust-game-wasm/pkg/hegemony_wasm_bg.wasm']:
  canonical=repo/'sites/generated/frozen/abi78'/Path(row['source']).name
  if not canonical.is_file():canonical=Path('/workspace/undergroundbattle-build/frozen-kernels/abi78')/Path(row['source']).name
  if canonical.is_file():restored=canonical.read_bytes()
  else:
   canonical=canonical.with_name(canonical.name+'.gz')
   assert canonical.is_file(),canonical
   restored=gzip.decompress(canonical.read_bytes())
  assert restored==b,canonical
  generated.append({'source':row['source'],'sha256':sha(b),'canonical':str(canonical),'exact':True,'restoredFromGzip':canonical.suffix=='.gz'});continue
 proc.stdin.write((bound['sourceCommit']+':'+row['source']+'\n').encode());proc.stdin.flush();header=proc.stdout.readline().split();assert len(header)==3 and header[1]==b'blob',row['source'];raw=proc.stdout.read(int(header[2]));assert proc.stdout.read(1)==b'\n';assert raw==b,row['source'];tracked+=1
proc.stdin.close();assert proc.wait()==0
dump('merged-stage-recovery.json',{'path':str(stage),'sourceCommit':bound['sourceCommit'],'mergedInto':main,'filesMappedAndExact':len(files),'filesMatchingRetainedGitBlobs':tracked,'generatedExactRetainedCanonical':generated,'extraOwnedFiles':{n:{'bytes':len((stage/n).read_bytes()),'sha256':sha((stage/n).read_bytes())} for n in sorted(allowed)},'notGitWorktree':not(stage/'.git').exists(),'databaseOrCredentials':False,'noTrashRecoveryExists':True,'deletionPerformed':False,'reconstructionNotEquivalentToTrashUndo':True})
after=disk();dump('inventory-summary.json',{'before':before,'afterReadOnlyAudit':after,'fixedMain':main,'head':git('rev-parse','HEAD').decode().strip(),'deletedFiles':0,'freedBytesByCleanup':0,'buildsOrBrowserLaunches':0,'guardLowered':False,'readonlyNetworkDownloadsNeverPersisted':True,'personalContextResult':'Unavailable for this conversation; no additional cleanup authorization inferred.','restorationToolAvailable':False,'noGuaranteedGlobalFdOccupancy':bool(unreadable)})
print(json.dumps({'before':before,'afterReadOnlyAudit':after,'remoteByteRecoveryProofPassed':all(x.get('sameAsLocalAndFixedMain') for x in remote),'firstSixAllocatedBytes':sum(x['allocatedBytes'] for x in selected[:6]),'allSevenAllocatedBytes':sum(x['allocatedBytes'] for x in selected),'stageFilesExact':len(files),'deletedFiles':0}))
