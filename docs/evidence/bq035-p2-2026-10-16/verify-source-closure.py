from pathlib import Path
import hashlib,json,subprocess,gzip,sys
repo=Path('/workspace/UndergroundBattle')
assert len(sys.argv)==3, 'Usage: verify-source-closure.py <clean-source-commit> <plain-source-copy>'
copy=Path(sys.argv[2])
out=Path('/workspace/bq035-p2-evidence-20261016')
commit=sys.argv[1]
base='ab1ea260a47cb028085dcfede5be9019b6098d87'
def git(*args):return subprocess.check_output(['git',*args],cwd=repo)
def sha(data):return hashlib.sha256(data).hexdigest()
assert git('rev-parse','HEAD').decode().strip()==commit
assert not git('status','--porcelain').strip(),'Audit must start from the clean source commit'
assert not (copy/'.git').exists(),'Plain source folder must not be a worktree'
pairs=[]
def add(source,target):
 p=repo/source
 if p.is_dir():
  pairs.extend((str(f.relative_to(repo)),str(Path(target)/f.relative_to(p))) for f in sorted(p.rglob('*')) if f.is_file())
 else:
  assert p.is_file(),source
  pairs.append((source,target))
for file in ['package.json','package-lock.json','wrangler.jsonc','drizzle.config.ts','README.md','.openai/hosting.json']:add('sites/'+file,file)
for directory in ['src','db','drizzle','test','scripts']:add('sites/'+directory,directory)
for file in ['build-storage.mjs','frozen-kernels.mjs','bq035-oracle.mjs']:add('tools/'+file,'tools/'+file)
for file in ['Cargo.toml','Cargo.lock']:add(file,file)
add('rust-game-wasm/build.sh','rust-game-wasm/build.sh')
add('rust-game-wasm/frozen/abi77/identity.json','rust-game-wasm/frozen/abi77/identity.json')
for file in ['legacy77-kernel.mjs','current78-kernel.mjs','bq035-compare.mjs','bq035Native78.fixture.json.gz']:add('rust-game-wasm/tests/'+file,'rust-game-wasm/tests/'+file)
for directory in ['src','public']:add('web/'+directory,'web/'+directory)
for file in ['package.json','package-lock.json','index.html','vite.config.ts','tsconfig.json','tsconfig.app.json','tsconfig.node.json']:add('web/'+file,'web/'+file)
for crate in ['rust-game','rust-game-wasm']:
 add(crate+'/src',crate+'/src')
 for file in ['Cargo.toml','Cargo.lock']:
  if (repo/crate/file).is_file():add(crate+'/'+file,crate+'/'+file)
add('rust-game/data/cards.json','rust-game/data/cards.json')
for file in ['hegemony_wasm.js','hegemony_wasm_bg.wasm']:add('rust-game-wasm/pkg/'+file,'rust-game-wasm/pkg/'+file)
assert len({target for _,target in pairs})==len(pairs)
files=[]
for source,target in pairs:
 raw=(repo/source).read_bytes();assert (copy/target).read_bytes()==raw,target
 files.append({'source':source,'target':target,'bytes':len(raw),'sha256':sha(raw)})
actual={str(f.relative_to(copy)) for f in copy.rglob('*') if f.is_file()}
owned={'.gitignore','source-provenance.json'}
assert actual=={target for _,target in pairs}|owned,actual-({target for _,target in pairs}|owned)
provenance=json.loads((copy/'source-provenance.json').read_text())
assert provenance['originalCommit']==commit
artifact=json.loads((out/'candidate-artifact-binding.json').read_text())
assert provenance['wasmSha256']==artifact['current78WasmSha256']
for name in actual:
 parts=Path(name).parts
 assert not ({'node_modules','target','.wrangler','.git'}&set(parts)),name
 assert not name.endswith(('.sqlite','.sqlite3','.db')) and not Path(name).name.startswith('.env'),name
binding=json.loads(gzip.decompress((repo/'rust-game-wasm/tests/bq035Native78.fixture.json.gz').read_bytes()))['sourceBindings']['files']
assert len(binding)==121
for name,digest in binding.items():
 assert sha(git('show',commit+':'+name))==digest,name
 assert sha((copy/name).read_bytes())==digest,name
paths=['.gitignore','docs/factions/card-specifications.json','rust-game','rust-game-wasm','sites/src','sites/scripts','sites/test','tools','web/src','web/public']
changed=git('diff','--name-only',base,commit,'--',*paths).decode().splitlines()
delta=[]
for file in changed:
 entry={'file':file}
 for key,ref in [('before',base),('after',commit)]:
  result=subprocess.run(['git','show',ref+':'+file],cwd=repo,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
  entry[key]=None if result.returncode else {'bytes':len(result.stdout),'sha256':sha(result.stdout)}
 delta.append(entry)
(out/'staged-source-files.json').write_text(json.dumps({'sourceCommit':commit,'files':files},indent=2)+'\n')
(out/'review-source-delta.json').write_text(json.dumps({'baselineGameTree':base,'candidateCodeCommit':commit,'files':delta},indent=2)+'\n')
result={'status':'closed','candidateSourceCommit':commit,'nativeCompilerSourceCommit':'fc810d3fa4fdf2c1a10eb7b451405a46503db954','all121CompilerInputsMatchCandidateAndCopy':True,'publishedV59GameSourceBaselineTree':base,'plainSourceCopy':str(copy),'gitWorktreeCreated':False,'mappedFiles':len(files),'extraOwnedFiles':sorted(owned),'exactMappedFiles':True,'sourceProvenance':provenance,'artifactBinding':json.loads((out/'candidate-artifact-binding.json').read_text()),'publishedProviderCheckoutUnchanged':'a2a002fb7cafb9030405853a1c98bd91800b627c','published':False,'workerComplete204AfterCoreFix':True,'workerRerunAfterOutputIsolation':False}
(out/'source-closure.json').write_text(json.dumps(result,indent=2)+'\n')
assert not git('status','--porcelain').strip(),'Audit must not change source'
print(json.dumps({'sourceCommit':commit,'mappedFiles':len(files),'compilerBoundFiles':len(binding),'reviewDeltaFiles':len(delta),'exactMatch':True}))
