from pathlib import Path
import datetime,gzip,hashlib,json,shutil,subprocess,os
repo=Path('/workspace/UndergroundBattle');provider=Path('/workspace/hidden-world-sites-abi77-20261015');out=Path('/workspace/jc041-publication-20261018')
head='4ffa7155537187d057b52612d2da076da6a2a47d';product='6b5b824b1e3fd08edfb1b47f7d979ef1ffe69317';previous='ff7fce139c7fa94e3b6cdbe1758498383fa632c4'
def git(*args,cwd=repo):return subprocess.check_output(['git',*args],cwd=cwd)
def sha(b):return hashlib.sha256(b).hexdigest()
def rec(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def dump(name,v):
 with (out/name).open('x') as f:json.dump(v,f,indent=2,ensure_ascii=False);f.write('\n')
assert git('rev-parse','HEAD',cwd=provider).decode().strip()==previous
assert not git('status','--porcelain',cwd=provider).strip()
assert git('rev-parse','HEAD').decode().strip()==head
assert not git('status','--porcelain').strip()
manifest=repo/'docs/evidence/jc041-p2-source-region-2026-10-18/artifacts-after.json';assert sha(manifest.read_bytes())=='deaa965554f59805b9c222c6b8869e6c3db31378b561231c6bb07ad3a3f597f7';a=json.loads(manifest.read_text());files=a['production'];assert len(files)==183
assert {str(p.relative_to(repo/'sites/dist')):rec(p) for p in (repo/'sites/dist').rglob('*') if p.is_file()}==files
plan=json.loads(Path('/workspace/jc066-publication-20261018/provider-copy-plan.json').read_text())
tracked=set(git('ls-tree','-r','--name-only',head).decode().splitlines())
changed=git('diff','--name-only','224a619459d64eaead06810b679ff57b1de94fd6',head,'--','rust-game','rust-game-wasm','web','sites','tools').decode().splitlines()
for src in changed:
 assert src in tracked,('Review deletion before projection',src)
 dest=src[6:] if src.startswith('sites/') else src
 assert dest not in plan or plan[dest]==src
 plan[dest]=src
bundle=json.loads(gzip.decompress((repo/'rust-game-wasm/tests/jc041Native87.fixture.json.gz').read_bytes()));native=json.loads(bundle['raw']['json']);binding=native['sourceBindings']['files'];assert len(binding)==137
reverse={v:k for k,v in plan.items()};assert all(src in reverse for src in binding)
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
  assert src in ['rust-game-wasm/pkg/hegemony_wasm.js','rust-game-wasm/pkg/hegemony_wasm_bg.wasm'];b=(repo/src).read_bytes()
  assert (repo/'sites/generated'/p.name).read_bytes()==b
  if p.suffix=='.wasm':assert sha(b)==bundle['wasmSha256']==a['currentWasm']['sha256']
 if dest in hooks:
  d=json.loads(b);d['scripts'].update(hooks[dest]);b=(json.dumps(d,ensure_ascii=False,indent=2)+'\n').encode()
 payload[dest]=b
for n,b in protected_before.items():assert payload[n]==b,('Protected config changed',n)
for n,h in binding.items():assert sha(payload[reverse[n]])==h,n
legacy=git('ls-files','--','.build-cache',cwd=provider).decode().splitlines();legacy_hashes={n:sha((provider/n).read_bytes()) for n in legacy}
dump('provider-copy-plan.json',plan);dump('retained-fixture-storage.json',{n:{'bytes':len(b),'sha256':sha(b)} for n,b in retained.items()})
for dest,b in payload.items():
 p=provider/dest;p.parent.mkdir(parents=True,exist_ok=True)
 if p.exists() and p.read_bytes()==b:continue
 src=repo/plan[dest]
 if dest not in hooks:
  assert src.is_file() and not src.is_symlink() and src.read_bytes()==b,('Immutable approved source mismatch',dest)
  temporary=p.with_name(p.name+'.jc041-approved-link');assert not temporary.exists()
  os.link(src,temporary);os.replace(temporary,p)
 else:p.write_bytes(b)
provenance={'originalRepository':'https://github.com/FireExtin/UndergroundBattle.git','originalCommit':head,'reviewedHead':head,'reviewedProductSource':product,'parentWholeBatchReviewPassed':True,'noRemainingBlockingP0P1P2':True,'wasmSha256':a['currentWasm']['sha256'],'productionWorkerSha256':files['server/index.js']['sha256'],'artifactsManifestSha256':sha(manifest.read_bytes()),'productionFileCount':183,'nativeCompilerInputCount':137,'previousOfficialSource':previous,'runtimePreparation':'Exact reviewed ABI87 production artifacts, no rebuild. Production real clock; test Worker excluded. Preserve exact77-86, fixture recovery hooks, D1 config/migrations, public audience. No game-main promotion or Actions.'}
(provider/'source-provenance.json').write_text(json.dumps(provenance,ensure_ascii=False,indent=2)+'\n')
backup=out/'prepublish-v68-dist';assert not backup.exists();shutil.move(str(provider/'dist'),str(backup));shutil.copytree(repo/'sites/dist',provider/'dist',copy_function=os.link)
for n,b in payload.items():assert (provider/n).read_bytes()==b,n
for n,b in retained.items():assert (provider/n).read_bytes()==b,n
for n,b in protected_before.items():assert (provider/n).read_bytes()==b,n
for n,h in legacy_hashes.items():assert sha((provider/n).read_bytes())==h,n
assert {str(p.relative_to(provider/'dist')):rec(p) for p in (provider/'dist').rglob('*') if p.is_file()}==files
changed_provider=set(git('diff','--name-only',cwd=provider).decode().splitlines());new_provider=set(git('ls-files','--others','--exclude-standard',cwd=provider).decode().splitlines());unknown=(changed_provider|new_provider)-(set(plan)|{'source-provenance.json'});assert not unknown,unknown
source_files={n:{'source':plan[n],'bytes':len(b),'sha256':sha(b),'fixtureHookException':n in hooks} for n,b in payload.items()};dump('provider-source-files.json',source_files)
audit={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reviewedHead':head,'reviewedProductSource':product,'previousOfficialSource':previous,'sourceFiles':len(plan),'compilerInputsVerified':137,'packageHookExceptions':list(hooks),'productionFilesExact':183,'changedProviderFiles':sorted(changed_provider),'newProviderFiles':sorted(new_provider),'unknownChanges':sorted(unknown),'retainedHistoricalCacheFilesUnchanged':len(legacy),'newCacheOrDatabaseOrCredentialFiles':0,'protectedConfigAndMigrationsUnchanged':True,'fixtureRecoveryHooksUnchanged':True,'oldV68DistRetained':str(backup),'productionClockReal':True,'testWorkerExcluded':True,'testsAndBuildNotRepeated':True,'newImmutableSourceAndProductionUsedRegularHardLinks':True};dump('provider-input-audit.json',audit)
print(json.dumps({k:v for k,v in audit.items() if k not in ['changedProviderFiles','newProviderFiles']},ensure_ascii=False));print(json.dumps({'changedSourceFiles':len(changed_provider),'newSourceFiles':len(new_provider)}))
