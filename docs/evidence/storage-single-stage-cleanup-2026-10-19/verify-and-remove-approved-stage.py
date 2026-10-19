from pathlib import Path
import base64,datetime,gzip,hashlib,json,os,shutil,stat,subprocess,sys

repo=Path('/workspace/UndergroundBattle')
out=Path('/workspace/storage-single-stage-cleanup-20261019')
target=Path('/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114')
approval='Sentinel_38d5c85712f48191a49d195ea66daa41'
audit='1abe880efc143d93deb68fa51b38b751591c3304'
main='224a619459d64eaead06810b679ff57b1de94fd6'
source='53b2114a220ca001ca8c40ff3b2a6f2b6276ef45'
assert sys.argv[1:]==['--execute-approved-single-path']
def git(*a):return subprocess.check_output(['git',*a],cwd=repo)
def sha(b):return hashlib.sha256(b).hexdigest()
def stamp():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def disk():
 s=os.statvfs('/workspace');return {'availableBytes':s.f_bavail*s.f_frsize,'guardBytes':1073741824}
def dump(n,x):
 p=out/n;assert not p.exists(),n;p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def nested_mounts():
 values=[]
 for line in Path('/proc/self/mountinfo').read_text().splitlines():
  p=line.split()[4]
  for a,b in [('\\040',' '),('\\011','\t'),('\\012','\n'),('\\134','\\')]:p=p.replace(a,b)
  if p==str(target) or p.startswith(str(target)+'/'):values.append(p)
 return values
assert target==Path('/workspace/undergroundbattle-build/source-stage-bq035-p2-20261016-53b2114')
assert target.resolve(strict=True)==target
for p in [target,*target.parents]:assert not p.is_symlink(),p
assert not nested_mounts(),'Nested mount boundary'
rootStat=target.lstat();assert stat.S_ISDIR(rootStat.st_mode)
files={};directories=[]
for parent,names,leafs in os.walk(target,followlinks=False):
 for name in names+leafs:
  p=Path(parent)/name;s=p.lstat();assert not stat.S_ISLNK(s.st_mode),p
  assert s.st_dev==rootStat.st_dev,p
  assert stat.S_ISREG(s.st_mode) or stat.S_ISDIR(s.st_mode),p
  if stat.S_ISREG(s.st_mode):files[str(p.relative_to(target))]=p
  else:directories.append(str(p.relative_to(target)))
assert not (target/'.git').exists()
assert git('rev-parse','HEAD').decode().strip()==audit
assert git('branch','--show-current').decode().strip()=='codex/storage-recovery-audit-wip-20261018'
assert not git('status','--porcelain','--untracked-files=all').strip()
assert git('ls-remote','origin','refs/heads/main').decode().split()[0]==main
assert subprocess.run(['git','merge-base','--is-ancestor',source,main],cwd=repo).returncode==0
mappingPath='docs/evidence/bq035-p2-2026-10-16/staged-source-files.json'
raw=git('show',audit+':'+mappingPath);assert git('show',main+':'+mappingPath)==raw
mapping=json.loads(raw);assert mapping['sourceCommit']==source
extras=json.loads(git('show',audit+':docs/evidence/storage-recovery-audit-2026-10-18/stage-extra-owned-files.json'))['files']
assert set(files)=={r['target'] for r in mapping['files']}|set(extras)
proc=subprocess.Popen(['git','cat-file','--batch'],cwd=repo,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
count=0;generated=[]
for row in mapping['files']:
 b=files[row['target']].read_bytes();assert len(b)==row['bytes'] and sha(b)==row['sha256'],row['target']
 if row['source'] in ['rust-game-wasm/pkg/hegemony_wasm.js','rust-game-wasm/pkg/hegemony_wasm_bg.wasm']:
  p=Path('/workspace/undergroundbattle-build/frozen-kernels/abi78')/Path(row['source']).name
  if p.is_file():original=p.read_bytes()
  else:p=p.with_name(p.name+'.gz');original=gzip.decompress(p.read_bytes())
  assert original==b
  generated.append({'path':str(p),'sha256':sha(p.read_bytes()),'restoredBytes':len(original),'restoredSha256':sha(original)})
 else:
  proc.stdin.write((source+':'+row['source']+'\n').encode());proc.stdin.flush();header=proc.stdout.readline().split();assert len(header)==3 and header[1]==b'blob'
  original=proc.stdout.read(int(header[2]));assert proc.stdout.read(1)==b'\n';assert original==b,row['source'];count+=1
for n,row in extras.items():
 b=files[n].read_bytes();assert b==base64.b64decode(row['contentBase64']) and sha(b)==row['sha256']
proc.stdin.close();assert proc.wait()==0
protected={}
for p in [repo,repo/'.git',repo/'docs',repo/'sites/dist',repo/'rust-game-wasm/pkg',Path('/workspace/hidden-world-sites-abi77-20261015'),Path('/workspace/undergroundbattle-build/frozen-kernels'),Path('/workspace/undergroundbattle-build/target'),Path('/workspace/undergroundbattle-build/tmp'),Path('/workspace/undergroundbattle-build/npm-cache'),Path('/workspace/undergroundbattle-build/source-stage-bq035-20261016-11927a3'),Path('/workspace/undergroundbattle-build/source-stage-bq035-20261016')]:
 s=p.lstat();protected[str(p)]={'device':s.st_dev,'inode':s.st_ino}
production=json.loads((repo/'docs/evidence/jc041-p2-source-region-2026-10-18/artifacts-after.json').read_text())['production']
for n,row in production.items():
 b=(repo/'sites/dist'/n).read_bytes();assert len(b)==row['bytes'] and sha(b)==row['sha256']
lock=(repo/'.git/codex-index-refresh.lock').stat();assert(lock.st_ino,lock.st_size,lock.st_mtime_ns)==(1048672,0,1792294521284793620)
active=[];references=[];limited=[]
for p in Path('/proc').iterdir():
 if not p.name.isdigit():continue
 try:
  raw=(p/'stat').read_text();state=raw[raw.rfind(')')+2:].split()[0]
  if state=='Z':continue
  pid=int(p.name);comm=(p/'comm').read_text().strip()
  if comm in ['cargo','rustc','workerd','chromium','node','git','vim','nvim','nano','code','esbuild']:active.append({'pid':pid,'comm':comm})
  for kind in ['cwd','exe']:
   try:
    path=os.readlink(p/kind)
    if path==str(target) or path.startswith(str(target)+'/'):references.append({'pid':pid,'comm':comm,'kind':kind,'path':path})
   except OSError:pass
  try:
   for fd in (p/'fd').iterdir():
    try:path=os.readlink(fd)
    except OSError:continue
    if path==str(target) or path.startswith(str(target)+'/'):references.append({'pid':pid,'comm':comm,'kind':'fd','path':path})
  except OSError:limited.append({'pid':pid,'comm':comm,'part':'fd'})
  try:
   for line in (p/'maps').read_text().splitlines():
    if str(target) in line:references.append({'pid':pid,'comm':comm,'kind':'maps'})
  except OSError:limited.append({'pid':pid,'comm':comm,'part':'maps'})
 except OSError:continue
assert not active and not references,{'active':active,'references':references}
allocated=int(subprocess.check_output(['du','-x','-B1','-s',str(target)],text=True).split()[0])
verified={'checkedUtc':stamp(),'approvalMessage':approval,'approvedOnlyPath':str(target),'auditCommit':audit,'mappedFiles':len(mapping['files']),'retainedGitBlobMatches':count,'extraOwnedFilesExact':len(extras),'filesToDelete':len(files),'directoriesInside':len(directories),'allocatedBytesBefore':allocated,'mappingRetainedInFixedMain':True,'externalAbi78Originals':generated,'symlinksFound':0,'nestedMounts':[],'sameFilesystemThroughout':True,'activeApplicationProcesses':active,'candidateProcessReferences':references,'existingSystemProcReadLimits':limited,'readLimitsAlreadyDisclosedInApprovedAudit':True,'protectedDirectories':protected,'production183ExactBefore':True,'noCleanupOutsideExactApprovedPath':True}
dump('pre-deletion-verification.json',verified)
assert not nested_mounts()
assert (target.lstat().st_dev,target.lstat().st_ino)==(rootStat.st_dev,rootStat.st_ino)
assert shutil.rmtree.avoids_symlink_attacks
parentFd=os.open(target.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
try:
 before=disk();started=stamp()
 # Only this literal user-approved child is removed; dir_fd confines all traversal.
 shutil.rmtree('source-stage-bq035-p2-20261016-53b2114',dir_fd=parentFd)
 after=disk();finished=stamp()
finally:os.close(parentFd)
assert not target.exists() and not target.is_symlink()
for p,row in protected.items():
 s=Path(p).lstat();assert(s.st_dev,s.st_ino)==(row['device'],row['inode']),p
for row in generated:assert sha(Path(row['path']).read_bytes())==row['sha256']
for n,row in production.items():assert sha((repo/'sites/dist'/n).read_bytes())==row['sha256']
assert not git('status','--porcelain','--untracked-files=all').strip()
guard=subprocess.run(['node','tools/build-storage.mjs'],cwd=repo,text=True,capture_output=True)
result={'startedUtc':started,'finishedUtc':finished,'approvalMessage':approval,'deletedOnlyPath':str(target),'allocatedBytesBefore':allocated,'allocatedBytesAfter':0,'filesDeleted':len(files),'directoryAbsent':True,'before':before,'after':after,'actualAvailableBytesIncrease':after['availableBytes']-before['availableBytes'],'protectedDirectoriesSameInodes':True,'retainedGitOriginalsAndAbi78Exact':True,'production183ExactAfter':True,'worktreeClean':True,'guard':{'exitCode':guard.returncode,'stdout':guard.stdout,'stderr':guard.stderr},'noOtherCleanup':True,'mainNotMerged':True,'noDeployment':True}
dump('deletion-and-space-result.json',result);print(json.dumps(result))
