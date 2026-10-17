from pathlib import Path
import hashlib,json,os,shutil,subprocess
R=Path('/workspace/UndergroundBattle');S=Path('/workspace/hidden-world-sites-abi77-20261015');O=Path('/workspace/jc054-publication-20261017')
CODE='b73f94f94dbbf5a3261f57b9913d237dc8b1c378';EVIDENCE='671f035ff14dc407f99db928920134d38aa64095';OLD='1f1b785df84738e6f85c1462482236b3ddabb8be'
def git(*args,cwd=R):return subprocess.check_output(['git',*args],cwd=cwd).decode().strip()
def sha(b):return hashlib.sha256(b).hexdigest()
def info(p):b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
assert git('rev-parse','HEAD')==EVIDENCE and not git('status','--porcelain')
assert not git('diff','--name-only',CODE,EVIDENCE,'--','.',':(exclude)docs')
assert git('rev-parse','HEAD',cwd=S)==OLD and not git('status','--porcelain',cwd=S)
previous=json.loads((R/'docs/evidence/wm062-publication-2026-10-17/provider-source-files.json').read_text())
for destination,row in previous.items():assert info(S/destination)['sha256']==row['sha256'],destination
plan=json.loads((R/'docs/evidence/wm062-publication-2026-10-17/provider-copy-plan.json').read_text())
for source,destination in [('sites/src','src'),('sites/db','db'),('sites/drizzle','drizzle'),('sites/test','test'),('sites/scripts','scripts'),('web/src','web/src'),('web/public','web/public'),('rust-game/src','rust-game/src'),('rust-game-wasm/src','rust-game-wasm/src')]:
 for file in (R/source).rglob('*'):
  if file.is_file():plan[str(Path(destination)/file.relative_to(R/source))]=str(file.relative_to(R))
for file in ['tools/jc054-oracle.mjs','rust-game-wasm/tests/current81-kernel.mjs','rust-game-wasm/tests/jc054-compare.mjs','rust-game-wasm/tests/jc054Native81.fixture.json.gz','rust-game-wasm/frozen/abi80/identity.json','rust-game-wasm/frozen/abi80/native-source-binding.json']:plan[file]=file
tree={}
for line in git('ls-tree','-r',CODE).splitlines():
 meta,name=line.split('\t',1);tree[name]=meta.split()[2]
sources=[f for f in plan.values() if f in tree]
ids=subprocess.check_output(['git','hash-object','--stdin-paths'],cwd=R,input=''.join(f+'\n' for f in sources).encode()).decode().splitlines()
assert all(tree[f]==h for f,h in zip(sources,ids))
assert sorted(set(plan.values())-set(tree))==['rust-game-wasm/pkg/hegemony_wasm.js','rust-game-wasm/pkg/hegemony_wasm_bg.wasm']
manifest_path=R/'docs/evidence/jc054-p2-2026-10-17/runtime-fixed-manifest.json';manifest=json.loads(manifest_path.read_text());runtime=Path(manifest['externalDirectory'])
for file,row in manifest['files'].items():assert info(runtime/file)==row,file
assert len(manifest['files'])==168
binding=json.loads((R/'docs/evidence/jc054-p2-2026-10-17/source-closure.json').read_text())['compilerInputs']
assert len(binding)==127
for file,h in binding.items():assert sha((R/file).read_bytes())==h,file
for file in ['.openai/hosting.json','wrangler.jsonc','drizzle.config.ts']:
 assert (S/file).read_bytes()==(R/'sites'/file).read_bytes()==subprocess.check_output(['git','show',OLD+':'+file],cwd=S),file
migrations={str(f.relative_to(S/'drizzle')):info(f) for f in (S/'drizzle').rglob('*') if f.is_file()}
assert migrations=={str(f.relative_to(R/'sites/drizzle')):info(f) for f in (R/'sites/drizzle').rglob('*') if f.is_file()}
retained=['scripts/restore-test-fixtures.mjs','scripts/test-fixture-storage.json','TEST_FIXTURE_STORAGE.md','test/fixtures/jc046-native-v077.json.gz','.gitignore','package.json','web/package.json']
retained_bytes={f:(S/f).read_bytes() for f in retained}
changes=[]
for destination,source in plan.items():
 data=(R/source).read_bytes();target=S/destination
 if destination in ['package.json','web/package.json']:
  actual=json.loads(target.read_text());expected=json.loads(data)
  for hook in ['prebuild','pretest']:assert actual['scripts'].pop(hook)=='node '+('../' if destination.startswith('web/') else '')+'scripts/restore-test-fixtures.mjs'
  assert actual==expected,destination
  continue
 if target.exists() and target.read_bytes()==data:continue
 if destination not in previous:assert not target.exists(),destination
 target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(R/source,target);changes.append(destination)
for file,data in retained_bytes.items():assert (S/file).read_bytes()==data,file
for file,h in binding.items():assert sha((S/file).read_bytes())==h,file
assert not (S/'.github').exists() and not (R/'.github').exists()
provenance={'originalRepository':'https://github.com/FireExtin/UndergroundBattle.git','originalCommit':EVIDENCE,'reviewedCodeCommit':CODE,'reviewedEvidenceCommit':EVIDENCE,'coreInputCommit':'db15514a1400cfec1319a20c821bcd779766aff5','wasmSha256':'669403d547a0ce451570e7953195315949a8ed8252ede34aa9269a607a4e0583','runtimeManifestSha256':sha(manifest_path.read_bytes()),'runtimeFileCount':168,'nativeCompilerInputCount':127,'previousOfficialSource':OLD,'runtimePreparation':'Reused the exact independently reviewed JC054+action-hints production build; all168 files and127 compiler inputs matched. Existing fixture storage hooks, D1 configuration and migrations retained. Response preview before BeginResponse is not implemented.'}
(S/'source-provenance.json').write_text(json.dumps(provenance,ensure_ascii=False,indent=2)+'\n')
assert not (O/'prepublish-v62-dist').exists();(S/'dist').rename(O/'prepublish-v62-dist')
shutil.copytree(runtime/'client',S/'dist/client')
(S/'dist/server').mkdir(parents=True)
for f in Path('/workspace/jc054-evidence-20261017/p2-stack-card/backend-fixed/candidate-worker/dist').iterdir():
 if f.is_file():shutil.copy2(f,S/'dist/server'/f.name)
(S/'dist/.openai').mkdir();shutil.copy2(S/'.openai/hosting.json',S/'dist/.openai/hosting.json')
for file,row in manifest['files'].items():assert info(S/'dist'/file)==row,file
for file,row in plan.items():
 if file not in ['package.json','web/package.json']:assert (S/file).read_bytes()==(R/row).read_bytes(),file
changed=git('diff','--name-only',cwd=S).splitlines();new=git('ls-files','--others','--exclude-standard',cwd=S).splitlines();assert not (set(changed+new)-set(plan)-{'source-provenance.json'})
subprocess.check_call(['git','diff','--check'],cwd=S)
(O/'provider-copy-plan.json').write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n')
(O/'provider-source-files.json').write_text(json.dumps({f:{'source':v,**info(S/f)} for f,v in plan.items()},ensure_ascii=False,indent=2)+'\n')
report={'reviewedCodeCommit':CODE,'reviewedEvidenceCommit':EVIDENCE,'previousOfficialSource':OLD,'sourceFiles':len(plan),'sourceFilesMatchingReviewedGit':len(sources),'nativeCompilerInputs':127,'runtimeFiles':168,'runtimeBytes':manifest['totalBytes'],'runtimeFilesExact':manifest['files'],'changedProviderFiles':changed,'newProviderFiles':new,'hostingWranglerAndD1ConfigUnchanged':True,'migrationsUnchanged':migrations,'oldFixtureStorageHooksPreserved':True,'unknownChanges':[],'noWorkflowFiles':True,'gameMainUnchanged':True}
(O/'provider-input-audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'sources':len(plan),'changed':len(changes),'runtimeFiles':168,'compilerInputs':127,'D1ConfigAndMigrationsUnchanged':True,'unknownChanges':[],'previousRuntimePreserved':str(O/'prepublish-v62-dist')}))
