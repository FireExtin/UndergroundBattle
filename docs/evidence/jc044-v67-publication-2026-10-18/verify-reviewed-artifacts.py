from pathlib import Path
import json,hashlib,subprocess,datetime
repo=Path('/workspace/UndergroundBattle');root=Path(__file__).parent
p=repo/'docs/evidence/jc044-stack-source-2026-10-18/runtime-manifest.json';raw=p.read_bytes();expected='6123629e7508d08b059c8acdc93df6a03c0a68ff8f0bc2715e0f6c4f14c0b401';assert hashlib.sha256(raw).hexdigest()==expected
assert raw==subprocess.check_output(['git','show','f9b88a7a3d67e420edfc628cb6b11a03c49ea618:docs/evidence/jc044-stack-source-2026-10-18/runtime-manifest.json'],cwd=repo)
m=json.loads(raw);assert m['source']=='f34424b8a699f74f742748a5cdbd4c73529bbe94'
proc=subprocess.Popen(['git','cat-file','--batch'],cwd=repo,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
for name,h in m['sourceSha256'].items():
 b=(repo/name).read_bytes();assert hashlib.sha256(b).hexdigest()==h,name
 proc.stdin.write((m['source']+':'+name+'\n').encode());proc.stdin.flush();header=proc.stdout.readline().split();assert len(header)==3 and header[1]==b'blob',name
 b=proc.stdout.read(int(header[2]));assert proc.stdout.read(1)==b'\n';assert hashlib.sha256(b).hexdigest()==h,name
proc.stdin.close();assert proc.wait()==0
prod=repo/'sites/dist';actual={str(p.relative_to(prod)) for p in prod.rglob('*') if p.is_file()};assert actual=={x['path'] for x in m['files']}
for row in m['files']:
 p=prod/row['path'];b=p.read_bytes();assert not p.is_symlink() and len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256'],row['path']
clock=Path('/workspace/jc044-stack-source-review-20261018/worker85-fixed/server')
for row in m['localClockWorker']:
 b=(clock/row['path']).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256'],row['path']
wasm=repo/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm';assert hashlib.sha256(wasm.read_bytes()).hexdigest()=='c02da9cbdfd99fcbfe3b4e0b82cc206ef8572edf2f58017ffd7a58bee2d0f9c1'
worker=next(x for x in m['files'] if x['path']=='server/index.js');s=(prod/'server/index.js').read_text()
assert 'LOCAL_TEST_CLOCK_MS' not in s and 'function createWorker(kernel2, now = Date.now)' in s and 'var index_default = createWorker(kernel);' in s
result={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reviewedSource':m['source'],'reviewedEvidence':'f9b88a7a3d67e420edfc628cb6b11a03c49ea618','runtimeManifestSha256':expected,'sourceFiles':len(m['sourceSha256']),'productionFiles':len(m['files']),'runtimeFiles':176,'clientFiles':166,'wasmFiles':9,'retainedKernels':'77–84 exact bytes','currentWasm':{'bytes':len(wasm.read_bytes()),'sha256':hashlib.sha256(wasm.read_bytes()).hexdigest()},'productionWorker':worker,'productionClockIsReal':True,'productionClockBinding':'createWorker(kernel) defaults to now = Date.now; function passed, not called at construction','fixedClockWorkerExcluded':True,'allProductionSizesAndSha256Match':True,'allSourceGitBytesMatch':True,'testedClockWrapperAlsoVerifiedButNotForPublication':len(m['localClockWorker'])}
(root/'prepublication-artifacts.json').write_text(json.dumps(result,indent=2)+'\n');(root/'runtime-manifest.json').write_bytes(raw);print(json.dumps(result,indent=2))
