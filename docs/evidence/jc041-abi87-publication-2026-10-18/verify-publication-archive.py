from pathlib import Path
import json,hashlib,subprocess,tarfile,io,gzip,datetime,sys
repo=Path('/workspace/UndergroundBattle');provider=Path('/workspace/hidden-world-sites-abi77-20261015');out=Path('/workspace/jc041-publication-20261018');commit=sys.argv[1]
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
changed=subprocess.check_output(['git','diff','--name-only','ff7fce139c7fa94e3b6cdbe1758498383fa632c4',commit],cwd=provider,text=True).splitlines();assert set(changed)<=set(source)|{'source-provenance.json'}
assert not any(any(p in Path(name).parts for p in ['.build-cache','node_modules','.wrangler','.env','.sites-runtime'])for name in changed)
manifest=repo/'docs/evidence/jc041-p2-source-region-2026-10-18/artifacts-after.json';assert sha(manifest.read_bytes())=='deaa965554f59805b9c222c6b8869e6c3db31378b561231c6bb07ad3a3f597f7';m=json.loads(manifest.read_text());expected={'dist/'+name:row for name,row in m['production'].items()};assert len(expected)==183
for p in (provider/'drizzle').rglob('*'):
 if p.is_file():expected['dist/.openai/drizzle/'+str(p.relative_to(provider/'drizzle'))]={'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())}
archive=out/'abi87-reviewed.tar.gz';payload={}
with tarfile.open(archive,'r:gz')as tar:
 for member in tar.getmembers():
  assert member.isdir()or member.isfile(),member.name
  assert '..'not in Path(member.name).parts and not member.name.startswith('/')
  if member.isfile():
   assert member.name not in payload;payload[member.name]=tar.extractfile(member).read()
assert set(payload)==set(expected),(set(payload)-set(expected),set(expected)-set(payload))
for name,b in payload.items():assert len(b)==expected[name]['bytes']and sha(b)==expected[name]['sha256'],name
assert len(payload)==186
worker='dist/server/index.js';assert sha(payload[worker])=='57e82a6dcda7532425732f96cdfc9d71a0139f55bff84dbcc20489823b5c2500';assert b'LOCAL_TEST_CLOCK_MS'not in payload[worker]
class HashSink:
 def __init__(self):self.bytes=0;self.digest=hashlib.sha256()
 def write(self,b):self.bytes+=len(b);self.digest.update(b);return len(b)
 def tell(self):return self.bytes
 def flush(self):pass
raw=HashSink()
with gzip.open(archive,'rb') as stream:
 while True:
  block=stream.read(1024*1024)
  if not block:break
  raw.write(block)
canonical=HashSink()
with tarfile.open(mode='w|',fileobj=canonical,format=tarfile.PAX_FORMAT)as tar:
 for name,b in sorted(payload.items()):
  info=tarfile.TarInfo(name);info.size=len(b);tar.addfile(info,io.BytesIO(b))
result={'checkedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'providerSource':commit,'sourceFilesMatching':len(source),'sourceFilesMatchingCommittedBlobs':len(source)-len(restored),'losslesslyRestoredFixtureTargets':sorted(restored),'retainedFixtureArchiveRawSha256':sha(plain),'reviewedProductSource':'6b5b824b1e3fd08edfb1b47f7d979ef1ffe69317','reviewedHead':'4ffa7155537187d057b52612d2da076da6a2a47d','remoteVerification':'Official source helper normal push then exact ls-remote HEAD check; no persistent remote assumed.','productionBuildOutputsMatching':183,'payloadFiles':186,'extraPayloadOnlyThreeUnchangedDrizzleFiles':True,'gzipArchive':{'path':str(archive),'bytes':archive.stat().st_size,'sha256':sha(archive.read_bytes())},'rawTar':{'bytes':raw.bytes,'sha256':raw.digest.hexdigest()},'canonicalPaxTar':{'bytes':canonical.bytes,'sha256':canonical.digest.hexdigest(),'fileCount':186,'headerPolicy':'Python tarfile.PAX_FORMAT, sorted regular files, fresh TarInfo(name), size from validated bytes; default0644 uid/gid/mtime0 uname/gname empty'},'fileManifest':expected,'productionWorkerSha256':sha(payload[worker]),'testClockWorkerExcluded':True,'sourceTreeClean':True,'rawAndCanonicalTarDigestsComputedWithoutDiskMaterialization':True,'newCacheOrDatabaseOrCredentialFiles':0}
(out/'publication-archive-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='fileManifest'}))
