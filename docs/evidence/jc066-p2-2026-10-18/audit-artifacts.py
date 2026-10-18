from pathlib import Path
import datetime,gzip,hashlib,json,re,subprocess,sys
repo=Path('/workspace/UndergroundBattle');root=Path('/workspace/jc066-p2-evidence-20261018')
def sha(b):return hashlib.sha256(b).hexdigest()
def rec(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def tree(p):return {str(f.relative_to(p)):rec(f) for f in sorted(p.rglob('*')) if f.is_file()}
f=repo/'rust-game-wasm/tests/jc066Native86.fixture.json.gz';bundle=json.loads(gzip.decompress(f.read_bytes()));raw=json.loads(bundle['raw']['json'])
assert bundle['raw']['sha256']==sha(bundle['raw']['json'].encode());assert bundle['raw']['bytes']==len(bundle['raw']['json'].encode());assert len(raw['sourceBindings']['files'])==135
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
for name,h in raw['sourceBindings']['files'].items():
 assert rec(repo/name)['sha256']==h,name
 assert sha(subprocess.check_output(['git','show',head+':'+name],cwd=repo))==h,name
before=json.loads((root/'native-before/repro.json').read_text());after=json.loads((root/'native-after-final/repro.json').read_text());repros=[]
for a,b in zip(before['cases'],after['cases']):
 assert a['state']==b['state'] and a['originalState']==b['originalState']
 assert a['gameAccepted'] and a['roomAccepted'] and not b['gameAccepted'] and not b['roomAccepted']
 repros.append({k:b[k] for k in ['name','revision','frameId','targetId','liveFaceDown','roomError'] }|{'originalStateSha256':sha(b['originalState'].encode()),'badStateSha256':sha(b['state'].encode()),'beforeNativeAccepted':True,'afterNativeRejected':True,'sameExactBytesBeforeAndAfter':True})
old=json.loads(json.loads(gzip.decompress((root/'baseline.fixture.json.gz').read_bytes()))['raw']['json'])
for s in old['scenarios']:assert s==next(t for t in raw['scenarios'] if t['name']==s['name']),s['name']
prod=repo/'sites/dist';generated=repo/'sites/generated';wrapper=root/'worker-after/server';wasm=rec(repo/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm')
assert wasm['sha256']==bundle['wasmSha256']=='2c9b850eb5b9e5e0b2e5e137d5056154602e82ba05db7d33ad251209ba2c3352'
assert rec(generated/'hegemony_wasm_bg.wasm')==wasm
assert (generated/'hegemony_wasm.js').read_bytes()==(repo/'rust-game-wasm/pkg/hegemony_wasm.js').read_bytes()
assert (repo/'rust-game-wasm/pkg/hegemony_wasm.js').read_bytes()==(root/'baseline-pkg/hegemony_wasm.js').read_bytes()
retained={}
for n in range(77,86):
 p=repo/f'rust-game-wasm/frozen/abi{n}/identity.json';identity=json.loads(p.read_text())
 assert p.read_bytes()==subprocess.check_output(['git','show','8607a94298ed323b0375106fac7db01ffbc77ccd:'+str(p.relative_to(repo))],cwd=repo)
 d=generated/f'abi{n}';assert rec(d/'hegemony_wasm_bg.wasm')['sha256']==identity['wasmSha256'];assert rec(d/'hegemony_wasm.js')['sha256']==identity['jsSha256']
 retained[str(n)]={'identity':identity,'generated':tree(d)}
old_modules=sorted(rec(f)['sha256'] for f in (root/'baseline-worker').glob('*.wasm'))
new_modules={f.name:rec(f) for f in (prod/'server').glob('*.wasm')};test_modules={f.name:rec(f) for f in wrapper.glob('*.wasm')}
assert len(old_modules)==len(new_modules)==len(test_modules)==10
prior='a389ee48edb55ef5089a641c781352f8890ea1a6238efd1bde2664fb6f78d30e'
assert sorted(old_modules)==sorted([prior]+[v['identity']['wasmSha256'] for v in retained.values()])
expected=sorted([wasm['sha256']]+[v['identity']['wasmSha256'] for v in retained.values()])
assert sorted(v['sha256'] for v in new_modules.values())==expected
assert sorted(v['sha256'] for v in test_modules.values())==expected
assert tree(prod/'client')==tree(root/'pre-p2-production/sites/dist/client')
code=(prod/'server/index.js').read_text();assert re.search(r'function createWorker\([^)]*now = Date\.now\)',code);assert 'var index_default = createWorker(kernel);' in code;assert 'LOCAL_TEST_CLOCK_MS' not in code
assert 'LOCAL_TEST_CLOCK_MS' in (wrapper/'worker.js').read_text()
assert len(code.encode())+sum(v['bytes'] for v in new_modules.values())<64*1024*1024
saved=json.loads((root/'pre-p2-production/manifest.json').read_text())
for entry in saved['files']:assert rec(root/'pre-p2-production'/entry['path'])=={k:entry[k] for k in ['bytes','sha256']}
# No re-signing or replacement of any prior frozen fixture/carrier metadata.
changed=subprocess.check_output(['git','diff','--name-only','8607a94298ed323b0375106fac7db01ffbc77ccd','--','rust-game-wasm/frozen'],cwd=repo,text=True).strip();assert not changed
old85=repo/'rust-game-wasm/tests/jc044Native85.fixture.json.gz';assert rec(old85)['sha256']=='4ad888e66386e390662f19cac5582d16dd36047b2111b90ec89590a84f8ccd63'
result={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'candidateSourceCommit':head,'beforeSourceCommit':'8607a94298ed323b0375106fac7db01ffbc77ccd','compilerInputs':raw['sourceBindings'],'native':{'rawSha256':bundle['raw']['sha256'],'rawBytes':bundle['raw']['bytes'],'fixture':rec(f),'counts':bundle['counts'],'original15NormalScenariosUnchanged':True},'p2Repros':repros,'beforeWasm':rec(root/'baseline-pkg/hegemony_wasm_bg.wasm'),'afterWasm':wasm,'retained':retained,'production':tree(prod),'generated':tree(generated),'testWorker':tree(wrapper),'client167FilesByteIdentical':True,'productionClockReal':True,'testClockExcluded':True,'onlyCurrent86KernelReplaced':True,'beforeProduction535FilesPreserved':len(saved['files']),'prior85Fixture':rec(old85),'selfSealRuling':{'choice':'B','authority':'explicit user project ruling','userReplyUtc':'2026-10-18 10:33 UTC','officialFaqClaim':False},'published':False,'independentReview':'pending-parent-review'}
p=root/sys.argv[1]
with p.open('x') as f:json.dump(result,f,indent=2,ensure_ascii=False);f.write('\n')
print(json.dumps({'output':str(p),'source':head,'wasm':wasm,'original15Unchanged':True,'productionFiles':len(result['production']),'clientFiles':len(tree(prod/'client')),'retainedKernels':list(retained),'sameBadStatesRejected':2,'selfSealAuthority':'user project ruling B','published':False}))
