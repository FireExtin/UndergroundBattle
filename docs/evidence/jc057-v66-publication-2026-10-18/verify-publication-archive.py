from pathlib import Path
import json,hashlib,subprocess,tarfile,io,gzip,datetime
repo=Path('/workspace/UndergroundBattle');provider=Path('/workspace/hidden-world-sites-abi77-20261015');out=Path('/workspace/jc057-publication-20261018');commit='7ab9768bd86a5a7e66dd988a42324624c1a23943'
sha=lambda b:hashlib.sha256(b).hexdigest()
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=provider,text=True).strip()==commit
assert not subprocess.check_output(['git','status','--porcelain'],cwd=provider).strip()
source=json.loads((out/'provider-source-files.json').read_text());storage=json.loads((provider/'scripts/test-fixture-storage.json').read_text())
restored=set(storage['targets']);assert restored=={'test/fixtures/jc046-native-v077.json','web/src/game/jc046Native77.fixture.json'}
compressed=subprocess.check_output(['git','show',commit+':'+storage['archive']],cwd=provider);plain=gzip.decompress(compressed)
assert len(plain)==storage['bytes']and sha(plain)==storage['sha256']
for name in restored:
 assert (provider/name).read_bytes()==plain;assert source[name]['bytes']==len(plain)and source[name]['sha256']==sha(plain),name
 assert not subprocess.check_output(['git','ls-files','--',name],cwd=provider).strip(),name
proc=subprocess.Popen(['git','cat-file','--batch'],cwd=provider,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
for name,row in source.items():
 if name in restored:continue
 proc.stdin.write((commit+':'+name+'\n').encode());proc.stdin.flush();header=proc.stdout.readline().split();assert len(header)==3 and header[1]==b'blob',name
 body=proc.stdout.read(int(header[2]));assert proc.stdout.read(1)==b'\n';assert len(body)==row['bytes']and sha(body)==row['sha256'],name
proc.stdin.close();assert proc.wait()==0
changed=subprocess.check_output(['git','diff','--name-only','6efc6273a4788e3ed4774d1a07b0921ff6d9d9a0',commit],cwd=provider,text=True).splitlines();assert set(changed)<=set(source)|{'source-provenance.json'}
assert not any(any(p in Path(name).parts for p in ['.build-cache','node_modules','.wrangler','.env','.sites-runtime'])for name in changed)
m=json.loads((repo/'docs/evidence/jc057-p2-2026-10-18/runtime-manifest.json').read_text());expected={'dist/'+row['path']:{'bytes':row['bytes'],'sha256':row['sha256']}for row in m['files']};assert len(expected)==177
for p in (provider/'drizzle').rglob('*'):
 if p.is_file():expected['dist/.openai/drizzle/'+str(p.relative_to(provider/'drizzle'))]={'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())}
archive=out/'abi84-reviewed.tar.gz';payload={}
with tarfile.open(archive,'r:gz')as tar:
 for member in tar.getmembers():
  assert member.isdir()or member.isfile(),member.name
  assert '..'not in Path(member.name).parts and not member.name.startswith('/')
  if member.isfile():
   assert member.name not in payload;payload[member.name]=tar.extractfile(member).read()
assert set(payload)==set(expected),(set(payload)-set(expected),set(expected)-set(payload))
for name,b in payload.items():assert len(b)==expected[name]['bytes']and sha(b)==expected[name]['sha256'],name
assert len(payload)==180
worker='dist/server/index.js';assert sha(payload[worker])=='6a92fb1b6794bc94d31c976bc2e7e77a9b172504516c07b877b89ac81ead9b46';assert b'LOCAL_TEST_CLOCK_MS'not in payload[worker]
raw=gzip.decompress(archive.read_bytes());canonical=out/'canonical-pax.tar';assert not canonical.exists()
with tarfile.open(canonical,'w',format=tarfile.PAX_FORMAT)as tar:
 for name,b in sorted(payload.items()):
  info=tarfile.TarInfo(name);info.size=len(b);tar.addfile(info,io.BytesIO(b))
result={'checkedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'providerSource':commit,'sourceFilesMatching':len(source),'sourceFilesMatchingCommittedBlobs':len(source)-len(restored),'losslesslyRestoredFixtureTargets':sorted(restored),'retainedFixtureArchiveRawSha256':sha(plain),'reviewedCode':'5a1a40dee9506b073f3ddd6dc4c796c3f7c72647','reviewedEvidence':'c674946d010559fa991ef78842d01d42de07d2f3','remoteVerification':'Official source helper normal push then exact ls-remote HEAD check; no persistent remote assumed.','productionBuildOutputsMatching':177,'runtimeFilesMatching':174,'payloadFiles':180,'extraPayloadOnlyThreeUnchangedDrizzleFiles':True,'gzipArchive':{'path':str(archive),'bytes':archive.stat().st_size,'sha256':sha(archive.read_bytes())},'rawTar':{'bytes':len(raw),'sha256':sha(raw)},'canonicalPaxTar':{'bytes':canonical.stat().st_size,'sha256':sha(canonical.read_bytes()),'fileCount':180,'headerPolicy':'Python tarfile.PAX_FORMAT, sorted regular files, fresh TarInfo(name), size from validated bytes; default0644 uid/gid/mtime0 uname/gname empty'},'fileManifest':expected,'productionWorkerSha256':sha(payload[worker]),'testClockWorkerExcluded':True,'sourceTreeClean':True,'newCacheOrDatabaseOrCredentialFiles':0}
(out/'publication-archive-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='fileManifest'}))
