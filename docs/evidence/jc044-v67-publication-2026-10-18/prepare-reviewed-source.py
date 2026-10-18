import hashlib,json,pathlib,shutil,subprocess,datetime,gzip
repo=pathlib.Path('/workspace/UndergroundBattle');provider=pathlib.Path('/workspace/hidden-world-sites-abi77-20261015');out=pathlib.Path('/workspace/jc044-publication-20261018')
code='f34424b8a699f74f742748a5cdbd4c73529bbe94';evidence='f9b88a7a3d67e420edfc628cb6b11a03c49ea618';previous='7ab9768bd86a5a7e66dd988a42324624c1a23943'
def git(*args,cwd=repo):return subprocess.check_output(['git',*args],cwd=cwd)
def sha(b):return hashlib.sha256(b).hexdigest()
def dump(name,data):
 with (out/name).open('x')as f:json.dump(data,f,ensure_ascii=False,indent=2);f.write('\n')
assert git('rev-parse','HEAD',cwd=provider).decode().strip()==previous
assert not git('status','--porcelain',cwd=provider).strip()
assert git('rev-parse','HEAD').decode().strip()==evidence
assert not git('status','--porcelain').strip()
subprocess.run(['node',str(repo/'tools/build-storage.mjs'),'--output-dir',str(provider),'--output-dir',str(out)],check=True,cwd=repo)
plan=json.loads(pathlib.Path('/workspace/jc057-publication-20261018/provider-copy-plan.json').read_text())
tracked=set(git('ls-tree','-r','--name-only',code).decode().splitlines())
changed=git('diff','--name-only','5a1a40dee9506b073f3ddd6dc4c796c3f7c72647',code,'--','rust-game','rust-game-wasm','web','sites','tools').decode().splitlines()
for src in changed:
 assert src in tracked,('Review deletion before projection',src)
 dest=src[6:]if src.startswith('sites/')else src
 assert dest not in plan or plan[dest]==src
 plan[dest]=src
bundle=json.loads(gzip.decompress((repo/'rust-game-wasm/tests/jc044Native85.fixture.json.gz').read_bytes()));native=json.loads(bundle['raw']['json']);identity={'sourceBindings':native['sourceBindings'],'wasmSha256':bundle['wasmSha256']};binding=identity['sourceBindings']['files'];assert len(binding)==133
reverse={v:k for k,v in plan.items()};assert all(src in reverse for src in binding)
storage=['scripts/restore-test-fixtures.mjs','scripts/test-fixture-storage.json','TEST_FIXTURE_STORAGE.md','test/fixtures/jc046-native-v077.json.gz','.gitignore']
retained={name:(provider/name).read_bytes()for name in storage}
protected=['.openai/hosting.json','wrangler.jsonc','drizzle.config.ts','db/schema.ts','drizzle/0000_numerous_lucky_pierre.sql','drizzle/meta/0000_snapshot.json','drizzle/meta/_journal.json']
protected_before={name:(provider/name).read_bytes()for name in protected}
hooks={name:{key:json.loads((provider/name).read_text())['scripts'][key]for key in ['prebuild','pretest']}for name in ['package.json','web/package.json']}
payload={}
for dest,src in plan.items():
 path=pathlib.PurePosixPath(dest);assert not path.is_absolute()and '..'not in path.parts
 assert not any(part in ['.git','.build-cache','node_modules','.wrangler','.env','.sites-runtime']for part in path.parts),dest
 if src in tracked:b=git('show',code+':'+src)
 else:
  assert src in ['rust-game-wasm/pkg/hegemony_wasm.js','rust-game-wasm/pkg/hegemony_wasm_bg.wasm'];b=(repo/src).read_bytes()
  generated=repo/'sites/generated'/path.name;assert generated.read_bytes()==b
  if path.suffix=='.wasm':assert sha(b)==identity['wasmSha256']
 if dest in hooks:
  data=json.loads(b);data['scripts'].update(hooks[dest]);b=(json.dumps(data,ensure_ascii=False,indent=2)+'\n').encode()
 payload[dest]=b
for name,b in protected_before.items():assert payload[name]==b,('Protected config changed',name)
for name,hash in binding.items():assert sha(payload[reverse[name]])==hash,name
manifest_path=repo/'docs/evidence/jc044-stack-source-2026-10-18/runtime-manifest.json';m=json.loads(manifest_path.read_text());assert m['source']==code and len(m['files'])==179
files={r['path']:r for r in m['files']}
for name,r in files.items():
 b=(repo/'sites/dist'/name).read_bytes();assert len(b)==r['bytes']and sha(b)==r['sha256'],name
assert set(files)=={str(p.relative_to(repo/'sites/dist'))for p in (repo/'sites/dist').rglob('*')if p.is_file()}
legacy=git('ls-files','--','.build-cache',cwd=provider).decode().splitlines();legacy_hashes={name:sha((provider/name).read_bytes())for name in legacy}
dump('provider-copy-plan.json',plan);dump('retained-fixture-storage.json',{name:{'bytes':len(b),'sha256':sha(b)}for name,b in retained.items()})
for dest,b in payload.items():
 p=provider/dest;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
provenance={'originalRepository':'https://github.com/FireExtin/UndergroundBattle.git','originalCommit':code,'reviewedCodeCommit':code,'reviewedEvidenceCommit':evidence,'parentWholeBatchReviewPassed':True,'P2ClosedByParentSerialReview':True,'wasmSha256':identity['wasmSha256'],'productionWorkerSha256':'77139a8daecdfa5f733a7cdd8715e1892e2d779eef4d59d799df99d50d2972b1','runtimeManifestSha256':sha(manifest_path.read_bytes()),'runtimeFileCount':176,'buildOutputCount':179,'nativeCompilerInputCount':133,'previousOfficialSource':previous,'runtimePreparation':'Exact reviewed ABI85 production artifacts, no rebuild. Production real clock, test Worker excluded. Retain exact77-84 kernels, fixture recovery hooks, D1 config/migrations and public audience. No game-main promotion or Actions.'}
(provider/'source-provenance.json').write_text(json.dumps(provenance,ensure_ascii=False,indent=2)+'\n')
backup=out/'prepublish-v66-dist';assert not backup.exists();shutil.move(str(provider/'dist'),str(backup));shutil.copytree(repo/'sites/dist',provider/'dist')
for dest,b in payload.items():assert (provider/dest).read_bytes()==b,dest
for name,b in retained.items():assert (provider/name).read_bytes()==b,name
for name,b in protected_before.items():assert (provider/name).read_bytes()==b,name
for name,hash in legacy_hashes.items():assert sha((provider/name).read_bytes())==hash,name
assert set(files)=={str(p.relative_to(provider/'dist'))for p in (provider/'dist').rglob('*')if p.is_file()}
for name,r in files.items():
 p=provider/'dist'/name;b=p.read_bytes();assert not p.is_symlink()and len(b)==r['bytes']and sha(b)==r['sha256'],name
changed_provider=set(git('diff','--name-only',cwd=provider).decode().splitlines());new_provider=set(git('ls-files','--others','--exclude-standard',cwd=provider).decode().splitlines());unknown=(changed_provider|new_provider)-(set(plan)|{'source-provenance.json'});assert not unknown,unknown
source_files={name:{'source':plan[name],'bytes':len(b),'sha256':sha(b),'fixtureHookException':name in hooks}for name,b in payload.items()};dump('provider-source-files.json',source_files)
audit={'checkedAtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reviewedCode':code,'reviewedEvidence':evidence,'previousOfficialSource':previous,'sourceFiles':len(plan),'compilerInputsVerified':len(binding),'explicitPkgBlobs':2,'packageHookExceptions':list(hooks),'buildOutputsExact':179,'runtimeFilesExact':176,'changedProviderFiles':sorted(changed_provider),'newProviderFiles':sorted(new_provider),'unknownChanges':sorted(unknown),'retainedHistoricalCacheFilesUnchanged':len(legacy_hashes),'newCacheOrDatabaseOrCredentialFiles':0,'protectedConfigAndMigrationsUnchanged':True,'fixtureRecoveryHooksUnchanged':True,'oldV66DistRetained':str(backup),'productionClockReal':True,'testWorkerExcluded':True,'testsAndBuildNotRepeated':True};dump('provider-input-audit.json',audit)
print(json.dumps({k:v for k,v in audit.items()if k not in ['changedProviderFiles','newProviderFiles']},ensure_ascii=False));print({'changedSourceFiles':len(changed_provider),'newSourceFiles':len(new_provider)})
