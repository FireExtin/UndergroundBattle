from pathlib import Path
import hashlib,json,subprocess
repo=Path('/workspace/UndergroundBattle')
provider=Path('/workspace/hidden-world-sites-abi77-20261015')
out=Path('/workspace/main-reviewed-v59-evidence-20261016')
expected_main='3d642f0c1e6e5ac021cd40560086df35638bfe4c'
approved='ab1ea260a47cb028085dcfede5be9019b6098d87'
provider_commit='a2a002fb7cafb9030405853a1c98bd91800b627c'
def git(*args,cwd=repo):return subprocess.check_output(['git',*args],cwd=cwd)
def sha(b):return hashlib.sha256(b).hexdigest()
review=repo/'docs/evidence/pass-history-2026-10-16'
published=repo/'docs/evidence/pass-history-publication-2026-10-16'
manifest=json.loads((review/'source-overlay-manifest.json').read_text())
assert manifest['reviewedOverlayTree']==approved
assert git('show','0dc88c4ae48f67a3b14c6a0928d94c5a4e9fef38:docs/evidence/pass-history-2026-10-16/source-overlay-manifest.json')==(review/'source-overlay-manifest.json').read_bytes()
patch=git('diff','--binary',manifest['progressDeltaBase'],manifest['featureSourceCommit'])
assert sha(patch)==manifest['featurePatchSha256']
entries={}
for row in git('ls-tree','-r','-z',approved).split(b'\0'):
 if not row:continue
 header,name=row.split(b'\t',1);mode,kind,blob=header.decode().split();assert kind=='blob';entries[name.decode()]=(mode,blob)
assert len(entries)==len(manifest['files'])==3440
batch=subprocess.Popen(['git','cat-file','--batch'],cwd=repo,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
for row in manifest['files']:
 assert entries[row['path']]==(row['mode'],row['gitBlob']),row['path']
 batch.stdin.write((row['gitBlob']+'\n').encode());batch.stdin.flush()
 blob,kind,size=batch.stdout.readline().decode().split();size=int(size);assert kind=='blob' and size==row['bytes']
 digest=hashlib.sha256();remaining=size
 while remaining:
  b=batch.stdout.read(min(1048576,remaining));assert b;digest.update(b);remaining-=len(b)
 assert batch.stdout.read(1)==b'\n';assert digest.hexdigest()==row['sha256'],row['path']
batch.stdin.close();batch.wait();assert batch.returncode==0
summary=json.loads((review/'summary.json').read_text())
assert summary['reviewedOverlayTree']==approved
assert summary['frontend']=={'files':128,'passed':1211,'failed':0}
assert summary['worker']=={'passed':189,'failed':0}
assert summary['nativeJournalReplay']=={'passed':1,'fullNativeRun':False}
assert summary['productionBuildPassed'] and summary['browser']['outcome']=='passed'
assert b'1211 passed' in (review/'release-frontend-final.log').read_bytes()
assert b'pass 189' in (review/'release-worker-full-final.log').read_bytes()
assert b'1 passed; 0 failed' in (review/'native-journal-replay.log').read_bytes()
assert git('rev-parse','HEAD',cwd=provider).decode().strip()==provider_commit
assert not git('status','--porcelain',cwd=provider).strip()
source_rows=json.loads((published/'provider-source-files.json').read_text())
assert len(source_rows)==1122
root_by_path={row['path']:row for row in manifest['files']}
provider_mapping=[]
for row in source_rows:
 path=row['path'];data=(provider/path).read_bytes();assert sha(data)==row['sha256'],path
 source='sites/'+path if path.split('/')[0] in {'.openai','db','drizzle','scripts','src','test'} or path in {'README.md','drizzle.config.ts','package-lock.json','wrangler.jsonc'} else path
 assert root_by_path[source]['sha256']==row['sha256'],source
 provider_mapping.append({'provider':path,'reviewedRoot':source,'sha256':row['sha256']})
provenance=json.loads((provider/'source-provenance.json').read_text())
assert provenance==json.loads((published/'provider-provenance.json').read_text())
assert provenance['reviewedOverlayTree']==approved and 'approved this exact overlay' in provenance['reviewApproval']
changed=git('diff','--name-status',expected_main,approved).decode().splitlines()
assert len(changed)==760
assert not any('bq035' in row.lower() for row in changed)
workflows={ref:git('ls-tree','-r','--name-only',ref,'--','.github/workflows').decode().splitlines() for ref in [expected_main,manifest['reviewedBase'],approved]}
assert not any(workflows.values())
assert subprocess.run(['git','merge-base','--is-ancestor',expected_main,manifest['reviewedBase']],cwd=repo).returncode==0
result={'approvedProviderSource':provider_commit,'approvedGameTree':approved,'expectedRemoteMain':expected_main,'reviewedBase':manifest['reviewedBase'],'featureSource':manifest['featureSourceCommit'],'deltaBase':manifest['progressDeltaBase'],'all3440ReviewedFilesMatchGitObjects':True,'all1122ProviderFilesMatchApprovedGameTree':True,'frontend':summary['frontend'],'worker':summary['worker'],'nativeJournalReplay':summary['nativeJournalReplay'],'productionBuildPassed':True,'browserLocalOnly':True,'fullNativeRerunForPassPatch':False,'fullMainDeltaFiles':len(changed),'bq035Included':False,'repositoryWorkflowTrees':workflows,'mutationPerformed':False}
(out/'reviewed-v59-audit.json').write_text(json.dumps(result,indent=2)+'\n')
(out/'full-main-delta.txt').write_text('\n'.join(changed)+'\n')
(out/'provider-to-approved-root.json').write_text(json.dumps(provider_mapping,indent=2)+'\n')
print(json.dumps(result))

artifact_file=review/'artifact-manifest.json'
artifacts=json.loads(artifact_file.read_text())
assert sha(artifact_file.read_bytes())==provenance['artifactManifestSha256']
assert artifacts['reviewedOverlayTree']==approved and artifacts['wasmAbi']==77 and artifacts['cardCount']==143
for row in artifacts['files']:
 raw=(provider/row['path'].removeprefix('sites/')).read_bytes()
 assert len(raw)==row['bytes'] and sha(raw)==row['sha256'],row['path']
result.update({'all162PublishedArtifactsMatchReview':True,'artifactManifestSha256':provenance['artifactManifestSha256'],'publishedWasm77Sha256':provenance['wasmSha256'],'publishedWorkerSha256':provenance['workerSha256']})
(out/'reviewed-v59-audit.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'reviewedArtifactsVerified':len(artifacts['files'])}))
