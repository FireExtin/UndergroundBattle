from pathlib import Path
import datetime,gzip,hashlib,json,shutil,subprocess,os

repo=Path('/workspace/UndergroundBattle')
provider=Path('/workspace/hidden-world-sites-abi77-20261015')
out=Path('/workspace/xq23-publication-20261019')
head='b7e4c78e5afaf2b73ab5a0fb770451e7b5fd78d4'
product='ffd2fb8303ba1c173d7f6ce9b1952802c5cc8742'
previous='0bf3a2793b0e4d28d4324b8251cebde685c8c085'
manifest=repo/'docs/evidence/xq23-p2-2026-10-19/candidate-p2-production.manifest.json'

def git(*args,cwd=repo): return subprocess.check_output(['git',*args],cwd=cwd)
def sha(b): return hashlib.sha256(b).hexdigest()
def rec(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def dump(name,value):
 with (out/name).open('x') as f: json.dump(value,f,indent=2,ensure_ascii=False);f.write('\n')
def independent_write(p,b):
 p.parent.mkdir(parents=True,exist_ok=True)
 temp=p.with_name(p.name+'.xq23-independent-copy')
 with temp.open('xb') as f:f.write(b)
 if p.exists():os.chmod(temp,p.stat().st_mode & 0o777)
 os.replace(temp,p)

assert git('rev-parse','HEAD',cwd=provider).decode().strip()==previous
assert not git('status','--porcelain',cwd=provider).strip()
assert git('rev-parse','HEAD').decode().strip()==head
assert not git('status','--porcelain').strip()
assert sha(manifest.read_bytes())=='8d90ce050632311097fb28bd6b1b9b50f43463dbf4aa76f540f965ef9ac1b1ad'
a=json.loads(manifest.read_bytes())
files={x['path']:{'bytes':x['bytes'],'sha256':x['sha256']} for x in a['files']}
assert len(files)==185
assert {str(p.relative_to(repo/'sites/dist')):rec(p) for p in (repo/'sites/dist').rglob('*') if p.is_file()}==files
assert 'LOCAL_TEST_CLOCK' not in (repo/'sites/dist/server/index.js').read_text()
assert 'Date.now' in (repo/'sites/dist/server/index.js').read_text()

plan=json.loads(Path('/workspace/jc041-publication-20261018/provider-copy-plan.json').read_text())
tracked=set(git('ls-tree','-r','--name-only',head).decode().splitlines())
changed=git('diff','--name-only','4ffa7155537187d057b52612d2da076da6a2a47d',head,'--','rust-game','rust-game-wasm','web','sites','tools').decode().splitlines()
for src in changed+['docs/evidence/xq23-p2-2026-10-19/paid-room-before.json','docs/evidence/xq23-p2-2026-10-19/current-p2.native.json.gz']:
 assert src in tracked,('Review deletion before projection',src)
 dest=src[6:] if src.startswith('sites/') else src
 assert dest not in plan or plan[dest]==src
 plan[dest]=src

bundle=json.loads(gzip.decompress((repo/'rust-game-wasm/tests/xq23Native88.fixture.json.gz').read_bytes()))
native=json.loads(bundle['raw']['json']);binding=native['sourceBindings']['files']
assert len(binding)==139
reverse={v:k for k,v in plan.items()}
assert all(src in reverse for src in binding)
storage=['scripts/restore-test-fixtures.mjs','scripts/test-fixture-storage.json','TEST_FIXTURE_STORAGE.md','test/fixtures/jc046-native-v077.json.gz','.gitignore']
retained={n:(provider/n).read_bytes() for n in storage}
protected=['.openai/hosting.json','wrangler.jsonc','drizzle.config.ts','db/schema.ts','drizzle/0000_numerous_lucky_pierre.sql','drizzle/meta/0000_snapshot.json','drizzle/meta/_journal.json']
protected_before={n:(provider/n).read_bytes() for n in protected}
hooks={n:{k:json.loads((provider/n).read_text())['scripts'][k] for k in ['prebuild','pretest']} for n in ['package.json','web/package.json']}
payload={}
for dest,src in plan.items():
 p=Path(dest);assert not p.is_absolute() and '..' not in p.parts
 assert not any(part in ['.git','.build-cache','node_modules','.wrangler','.env','.sites-runtime'] for part in p.parts),dest
 if src in tracked:b=git('show',head+':'+src)
 else:
  assert src in ['rust-game-wasm/pkg/hegemony_wasm.js','rust-game-wasm/pkg/hegemony_wasm_bg.wasm']
  b=(repo/src).read_bytes()
  assert (repo/'sites/generated'/p.name).read_bytes()==b
  if p.suffix=='.wasm':assert sha(b)==bundle['wasmSha256']==a['wasmSha256']
 if dest in hooks:
  d=json.loads(b);d['scripts'].update(hooks[dest]);b=(json.dumps(d,ensure_ascii=False,indent=2)+'\n').encode()
 payload[dest]=b
for n,b in protected_before.items():assert payload[n]==b,('Protected config changed',n)
for n,h in binding.items():assert sha(payload[reverse[n]])==h,n
legacy=git('ls-files','--','.build-cache',cwd=provider).decode().splitlines()
legacy_hashes={n:sha((provider/n).read_bytes()) for n in legacy}
dump('provider-copy-plan.json',plan)
dump('retained-fixture-storage.json',{n:{'bytes':len(b),'sha256':sha(b)} for n,b in retained.items()})

for dest,b in payload.items():
 p=provider/dest
 if p.exists() and p.read_bytes()==b:continue
 independent_write(p,b)

provenance={
 'originalRepository':'https://github.com/FireExtin/UndergroundBattle.git',
 'originalCommit':head,'reviewedHead':head,'reviewedProductSource':product,
 'parentWholeBatchReviewPassed':True,'noRemainingBlockingP0P1P2':True,
 'wasmSha256':a['wasmSha256'],'productionWorkerSha256':a['workerSha256'],
 'artifactsManifestSha256':sha(manifest.read_bytes()),'productionFileCount':185,
 'nativeCompilerInputCount':139,'previousOfficialSource':previous,
 'runtimePreparation':'Exact reviewed ABI88 production artifacts, no rebuild. Production real clock; test Worker excluded. Preserve exact77-87, fixture recovery hooks, D1 config/migrations, public audience. No game-main promotion or Actions.',
 'sourceMaterialization':'Changed editable source uses independent atomic copies. Only sealed generated production files share regular hardlinks.'
}
independent_write(provider/'source-provenance.json',(json.dumps(provenance,ensure_ascii=False,indent=2)+'\n').encode())
backup=out/'prepublish-v69-dist';assert not backup.exists()
shutil.move(str(provider/'dist'),str(backup))
shutil.copytree(repo/'sites/dist',provider/'dist',copy_function=os.link)

for n,b in payload.items():assert (provider/n).read_bytes()==b,n
for n,b in retained.items():assert (provider/n).read_bytes()==b,n
for n,b in protected_before.items():assert (provider/n).read_bytes()==b,n
for n,h in legacy_hashes.items():assert sha((provider/n).read_bytes())==h,n
assert {str(p.relative_to(provider/'dist')):rec(p) for p in (provider/'dist').rglob('*') if p.is_file()}==files
changed_provider=set(git('diff','--name-only',cwd=provider).decode().splitlines())
new_provider=set(git('ls-files','--others','--exclude-standard',cwd=provider).decode().splitlines())
unknown=(changed_provider|new_provider)-(set(plan)|{'source-provenance.json'})
assert not unknown,unknown
source_files={n:{'source':plan[n],'bytes':len(b),'sha256':sha(b),'fixtureHookException':n in hooks} for n,b in payload.items()}
dump('provider-source-files.json',source_files)
audit={
 'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'reviewedHead':head,'reviewedProductSource':product,'previousOfficialSource':previous,
 'sourceFiles':len(plan),'compilerInputsVerified':139,'packageHookExceptions':list(hooks),
 'productionFilesExact':185,'manifestSha256':sha(manifest.read_bytes()),
 'changedProviderFiles':sorted(changed_provider),'newProviderFiles':sorted(new_provider),'unknownChanges':sorted(unknown),
 'retainedHistoricalCacheFilesUnchanged':len(legacy),'newCacheOrDatabaseOrCredentialFiles':0,
 'protectedConfigAndMigrationsUnchanged':True,'fixtureRecoveryHooksUnchanged':True,
 'oldV69DistRetained':str(backup),'productionClockReal':True,'testWorkerExcluded':True,
 'testsAndBuildNotRepeated':True,'changedEditableSourcesAreIndependentCopies':True,
 'sealedProductionUsesRegularHardLinks':True
}
dump('provider-input-audit.json',audit)
print(json.dumps({k:v for k,v in audit.items() if k not in ['changedProviderFiles','newProviderFiles']},ensure_ascii=False))
print(json.dumps({'changedSourceFiles':len(changed_provider),'newSourceFiles':len(new_provider)}))
