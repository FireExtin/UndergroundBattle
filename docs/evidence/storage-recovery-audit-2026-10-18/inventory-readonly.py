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
before=disk()
assert git('rev-parse','HEAD').decode().strip()=='d0a52c8d8c7b9589952b7dadbb0ab7283dd6a8e9'
assert not git('status','--porcelain').strip()
assert git('ls-remote','origin','refs/heads/main').decode().split()[0]==main
roots=['/workspace','/workspace/undergroundbattle-build','/workspace/UndergroundBattle','/workspace/undergroundbattle-build/target/debug','/workspace/undergroundbattle-build/npm-cache','/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114']
sizes={}
for root in roots:
 rows=[]
 for line in subprocess.check_output(['du','-x','-B1','--max-depth=1',root],text=True).splitlines():
  n,p=line.split('\t',1);rows.append({'path':p,'allocatedBytes':int(n)})
 sizes[root]=sorted(rows,key=lambda x:x['allocatedBytes'],reverse=True)
dump('directory-sizes.json',{'checkedUtc':before['checkedUtc'],'rows':sizes,'notAdditiveAcrossSeparateDuInvocations':True})
procRoots=['/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114','/workspace/undergroundbattle-build/npm-cache','/workspace/undergroundbattle-build/target/debug']
live=[];references=[];unreadable=[];zombies=0
for proc in Path('/proc').iterdir():
 if not proc.name.isdigit():continue
 try:
  stat=(proc/'stat').read_text();state=stat[stat.rfind(')')+2:].split()[0]
  if state=='Z':zombies+=1;continue
  pid=int(proc.name);comm=(proc/'comm').read_text().strip();live.append({'pid':pid,'comm':comm,'state':state})
  for kind in ['cwd','exe']:
   try:
    target=os.readlink(proc/kind)
    if any(target==r or target.startswith(r+'/') for r in procRoots):references.append({'pid':pid,'kind':kind,'path':target})
   except OSError:pass
  try:
   for fd in (proc/'fd').iterdir():
    try:target=os.readlink(fd)
    except OSError:continue
    if any(target==r or target.startswith(r+'/') for r in procRoots):references.append({'pid':pid,'kind':'fd','path':target})
  except OSError:unreadable.append({'pid':pid,'comm':comm,'part':'fd'})
  try:
   for line in (proc/'maps').read_text().splitlines():
    if any(r in line for r in procRoots):references.append({'pid':pid,'kind':'maps','path':line.split(maxsplit=5)[-1]})
  except OSError:unreadable.append({'pid':pid,'comm':comm,'part':'maps'})
 except OSError:continue
trash=[]
for p in [Path('/workspace/.local/share/Trash'),Path('/root/.local/share/Trash'),Path('/workspace/.Trash'),Path('/.Trash-0')]:
 try:trash.append({'path':str(p),'exists':p.exists(),'sameDeviceAsWorkspace':p.stat().st_dev==repo.stat().st_dev if p.exists() else None})
 except OSError as e:trash.append({'path':str(p),'unreadable':type(e).__name__})
dump('process-and-recovery.json',{'live':live,'zombies':zombies,'candidateReferences':references,'unreadable':unreadable,'fullOccupancyNotAttested':bool(unreadable),'trashPaths':trash,'gio':shutil.which('gio'),'trashPut':shutil.which('trash-put'),'sameFilesystemTrashCannotReleaseBlocks':True,'noNewEnvironmentUsed':True})

locks={n:json.loads(git('show',main+':'+n)) for n in ['sites/package-lock.json','web/package-lock.json']}
integrities={};lockProof={}
for n,d in locks.items():
 b=git('show',main+':'+n);assert (repo/n).read_bytes()==b
 lockProof[n]={'matchesFixedMain':True,'sha256':sha(b)}
 for name,p in d.get('packages',{}).items():
  if p.get('integrity'):integrities.setdefault(p['integrity'],[]).append({'lock':n,'package':name,'version':p.get('version'),'resolved':p.get('resolved')})
cache=Path('/workspace/undergroundbattle-build/npm-cache/_cacache');byPath={}
for file in (cache/'index-v5').rglob('*'):
 if not file.is_file():continue
 for line in file.read_text(errors='replace').splitlines():
  try:d=json.loads(line.split('\t',1)[1])
  except (ValueError,IndexError):continue
  integ=d.get('integrity');hits=integrities.get(integ,[])
  if not hits:continue
  alg,digest=integ.split('-',1);h=base64.b64decode(digest).hex();p=cache/'content-v2'/alg/h[:2]/h[2:4]/h[4:]
  if not p.is_file() or p.is_symlink():continue
  s=p.stat();byPath[str(p)]={'path':str(p),'bytes':s.st_size,'allocatedBytes':s.st_blocks*512,'links':s.st_nlink,'inode':s.st_ino,'integrity':integ,'matchingMainLock':hits,'url':hits[0]['resolved']}
selected=sorted(byPath.values(),key=lambda x:x['allocatedBytes'],reverse=True)[:7]
assert len(selected)==7
remote=[]
for row in selected:
 b=Path(row['path']).read_bytes();integ='sha512-'+base64.b64encode(hashlib.sha512(b).digest()).decode();assert integ==row['integrity']
 row['localBytesAndIntegrityVerified']=True;row['sha256']=sha(b)
 # Read-only network proof: stream the exact registry URL into a digest, never execute or persist the payload.
 try:
  request=urllib.request.Request(row['url'],headers={'User-Agent':'UndergroundBattle-readonly-cache-recovery-audit/1'})
  with urllib.request.urlopen(request,timeout=25) as response:
   digest=hashlib.sha512();count=0
   while True:
    chunk=response.read(1024*1024)
    if not chunk:break
    digest.update(chunk);count+=len(chunk)
   actual='sha512-'+base64.b64encode(digest.digest()).decode()
   proof={'url':row['url'],'status':response.status,'bytes':count,'integrity':actual,'sameAsLocalAndFixedMain':count==row['bytes'] and actual==row['integrity'],'payloadPersisted':False,'payloadExecuted':False}
   assert proof['sameAsLocalAndFixedMain']
   remote.append(proof)
 except Exception as e:remote.append({'url':row['url'],'errorType':type(e).__name__,'message':str(e),'payloadPersisted':False,'payloadExecuted':False})
dump('npm-cache-recovery-candidates.json',{'fixedMain':main,'lockProof':lockProof,'selected':selected,'firstSixAllocatedBytes':sum(x['allocatedBytes'] for x in selected[:6]),'allSevenAllocatedBytes':sum(x['allocatedBytes'] for x in selected),'allCurrentUrlsByteVerified':all(x.get('sameAsLocalAndFixedMain') for x in remote),'remoteReadOnlyProof':remote,'permanentRemovalNotAuthorizedByRebuildabilityAlone':True,'deletionPerformed':False,'cacheIndexUntouched':True,'restorationRequiresDownloadAndAdditionalSpace':True})

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
