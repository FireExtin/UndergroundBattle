from pathlib import Path
import datetime,gzip,hashlib,json,re,subprocess,sys
repo=Path('/workspace/UndergroundBattle');root=Path('/workspace/jc066-evidence-20261018')
def sha(b):return hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def tree(p):
 assert p.is_dir(),p
 return {str(f.relative_to(p)):record(f) for f in sorted(p.rglob('*')) if f.is_file()}
fixture=repo/'rust-game-wasm/tests/jc066Native86.fixture.json.gz';bundle=json.loads(gzip.decompress(fixture.read_bytes()));raw=json.loads(bundle['raw']['json'])
assert bundle['raw']['bytes']==len(bundle['raw']['json'].encode());assert bundle['raw']['sha256']==sha(bundle['raw']['json'].encode())
assert len(raw['sourceBindings']['files'])==135
for name,h in raw['sourceBindings']['files'].items():assert record(repo/name)['sha256']==h,name
pkg=repo/'rust-game-wasm/pkg';current=record(pkg/'hegemony_wasm_bg.wasm');assert current['sha256']==bundle['wasmSha256']
generated=repo/'sites/generated';prod=repo/'sites/dist';wrapper=root/'worker86/server'
assert record(generated/'hegemony_wasm_bg.wasm')==current
assert (pkg/'hegemony_wasm.js').read_bytes()==(generated/'hegemony_wasm.js').read_bytes()
retained={}
for n in range(77,86):
 identity=json.loads((repo/f'rust-game-wasm/frozen/abi{n}/identity.json').read_bytes())
 p=generated/f'abi{n}'
 assert record(p/'hegemony_wasm_bg.wasm')['sha256']==identity['wasmSha256'],n
 assert record(p/'hegemony_wasm.js')['sha256']==identity['jsSha256'],n
 retained[str(n)]={'identity':identity,'generated':tree(p)}
prod_modules={f.name:record(f) for f in (prod/'server').glob('*.wasm')}
wrapper_modules={f.name:record(f) for f in wrapper.glob('*.wasm')}
assert len(prod_modules)==10 and len(wrapper_modules)==10
expected=[current['sha256']]+[x['identity']['wasmSha256'] for x in retained.values()]
assert sorted(x['sha256'] for x in prod_modules.values())==sorted(expected)
assert sorted(x['sha256'] for x in wrapper_modules.values())==sorted(expected)
production=(prod/'server/index.js').read_text();local=(wrapper/'worker.js').read_text()
assert re.search(r'function createWorker\([^)]*now = Date\.now\)',production)
assert re.search(r'var index_default = createWorker\(kernel\);',production)
assert 'LOCAL_TEST_CLOCK_MS' not in production
assert 'LOCAL_TEST_CLOCK_MS' in local
assert current['bytes']+sum(x['generated']['hegemony_wasm_bg.wasm']['bytes'] for x in retained.values())+len(production.encode())<64*1024*1024
# Pin v67 proof and original ABI85 witness instead of re-signing it.
published_manifest=repo/'docs/evidence/jc044-stack-source-2026-10-18/runtime-manifest.json'
assert record(published_manifest)['sha256']=='6123629e7508d08b059c8acdc93df6a03c0a68ff8f0bc2715e0f6c4f14c0b401'
original85=repo/'rust-game-wasm/tests/jc044Native85.fixture.json.gz'
assert record(original85)['sha256']=='4ad888e66386e390662f19cac5582d16dd36047b2111b90ec89590a84f8ccd63'
old=json.loads((root/'pre-abi86-v67/manifest.json').read_text())
for entry in old['files']:
 name=entry['path'];p=root/'pre-abi86-v67'/name
 assert record(p)=={k:entry[k] for k in ['bytes','sha256']},name
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
result={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'gitHeadAtAudit':head,
 'identity':{k:raw['catalog'][k] for k in ['rulesVersion','cardPoolVersion','engineVersion']},'cards':len(raw['catalog']['cards']),'societies':len(raw['catalog']['societies']),
 'native':{'rawSha256':bundle['raw']['sha256'],'rawBytes':bundle['raw']['bytes'],'fixture':record(fixture),'counts':bundle['counts'],'sourceBindings':raw['sourceBindings']},
 'currentWasm':current,'currentGlue':record(pkg/'hegemony_wasm.js'),'retained':retained,
 'production':tree(prod),'generated':tree(generated),'fixedClockTestWrapper':tree(wrapper),
 'productionClockIsRealServerTime':True,'fixedClockWrapperExcludedFromProduction':True,'tenExactWasmInProductionAndWrapper':True,
 'publishedV67':{'runtimeManifest':record(published_manifest),'fixture85':record(original85),'backupFilesUnchanged':len(old['files']),'siteChanged':False},
 'scope':'Local source/artifact identity audit only; no publication, browser bypass, online gameplay or main merge.'}
out=root/sys.argv[1]
with out.open('x') as f:json.dump(result,f,indent=2,ensure_ascii=False);f.write('\n')
print(json.dumps({'output':str(out),'currentWasm':current,'nativeRawSha256':bundle['raw']['sha256'],'productionFiles':len(result['production']),'generatedFiles':len(result['generated']),'retainedExactKernels':list(retained),'preservedV67Files':len(old['files'])}))
