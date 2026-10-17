from pathlib import Path
import datetime
import gzip
import hashlib
import json
import subprocess

REPO = Path('/workspace/UndergroundBattle')
SITE = Path('/workspace/hidden-world-sites-abi77-20261015')
OUT = Path('/workspace/wm062-publication-20261017')
CODE = 'a9161c958dedceb95b70310d897dd5eeb8550e52'
EVIDENCE = 'd1a170939ac680c74dc7b73e2c1da9647d0bf32e'

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def git(*args, cwd=REPO):
    return subprocess.check_output(['git', *args], cwd=cwd, text=True).strip()

assert git('rev-parse', 'HEAD') == EVIDENCE
assert not git('status', '--porcelain')
assert not git('diff', '--name-only', CODE, EVIDENCE, '--', '.', ':(exclude)docs')
plan = json.loads((OUT / 'provider-copy-plan.json').read_text())
tree = {}
for row in git('ls-tree', '-r', CODE).splitlines():
    identity, name = row.split('\t', 1)
    tree[name] = identity.split()[2]
sources = [name for name in plan.values() if name in tree]
objects = subprocess.check_output(
    ['git', 'hash-object', '--stdin-paths'], cwd=REPO,
    input=''.join(name + '\n' for name in sources), text=True,
).splitlines()
assert all(tree[name] == object_id for name, object_id in zip(sources, objects))
assert len(objects) == len(sources) == 1167
assert sorted(set(plan.values()) - set(tree)) == [
    'rust-game-wasm/pkg/hegemony_wasm.js',
    'rust-game-wasm/pkg/hegemony_wasm_bg.wasm',
]

copied = {}
for destination, source in plan.items():
    before, after = (REPO / source).read_bytes(), (SITE / destination).read_bytes()
    if destination in ['package.json', 'web/package.json']:
        expected, actual = json.loads(before), json.loads(after)
        hooks = json.loads((OUT / ('provider-hooks-' + destination.replace('/', '-'))).read_text())
        for key, value in hooks.items():
            assert actual['scripts'].pop(key) == value
        assert actual == expected, destination
    else:
        assert after == before, destination
    copied[destination] = {'source': source, 'sha256': sha(after), 'bytes': len(after)}

storage = json.loads((SITE / 'scripts/test-fixture-storage.json').read_text())
fixture = gzip.decompress((SITE / storage['archive']).read_bytes())
assert sha(fixture) == storage['sha256'] and len(fixture) == storage['bytes']
for target in storage['targets']:
    assert (SITE / target).read_bytes() == fixture
    assert not git('ls-files', '--', target, cwd=SITE), target
retained_files = [
    'scripts/restore-test-fixtures.mjs', 'scripts/test-fixture-storage.json',
    'TEST_FIXTURE_STORAGE.md', 'test/fixtures/jc046-native-v077.json.gz', '.gitignore',
]
for name in retained_files:
    assert (SITE / name).read_bytes() == (OUT / 'provider-retained-storage' / name).read_bytes()

native = json.loads((REPO / 'docs/evidence/wm062-2026-10-16/native-source-binding.json').read_text())
assert len(native['files']) == 125
for name, digest in native['files'].items():
    assert sha((SITE / name).read_bytes()) == digest, name

manifest_path = REPO / 'docs/evidence/wm062-2026-10-16/runtime-manifest.json'
manifest = json.loads(manifest_path.read_text())
runtime = {}
for group in ['client', 'server']:
    expected = {p: record for p, record in manifest[group].items()
                if group == 'client' or p == 'index.js' or p.endswith('.wasm')}
    actual = {p.relative_to(SITE / 'dist' / group).as_posix()
              for p in (SITE / 'dist' / group).rglob('*') if p.is_file()
              and (group == 'client' or p.name == 'index.js' or p.suffix == '.wasm')}
    assert actual == set(expected), group
    for name, record in expected.items():
        raw = (SITE / 'dist' / group / name).read_bytes()
        assert sha(raw) == record['sha256'] and len(raw) == record['bytes'], group + '/' + name
        runtime[group + '/' + name] = record
assert len(runtime) == 166
assert sha((SITE / 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm').read_bytes()) == '0229f75b7af79035f2a4025fc11c6de6f3f0b895127cc8b537f829268d030472'
assert sha((SITE / 'rust-game-wasm/pkg/hegemony_wasm.js').read_bytes()) == 'fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb'
for name in ['.openai/hosting.json', 'wrangler.jsonc']:
    raw = (SITE / name).read_bytes()
    assert raw == (REPO / 'sites' / name).read_bytes()
    assert raw == subprocess.check_output(['git', 'show', '906a1b4c54e43eff5dd722c275d55ad567b8c20c:' + name], cwd=SITE)
migrations = {p.relative_to(SITE / 'drizzle').as_posix(): sha(p.read_bytes())
              for p in (SITE / 'drizzle').rglob('*') if p.is_file()}
assert migrations == {p.relative_to(REPO / 'sites/drizzle').as_posix(): sha(p.read_bytes())
                      for p in (REPO / 'sites/drizzle').rglob('*') if p.is_file()}
hosting = json.loads((SITE / '.openai/hosting.json').read_text())
assert hosting == {'project_id': 'appgprj_6abf7bf54a7481918a50e1ef1509ca68', 'd1': 'DB', 'r2': None}
assert json.loads((SITE / 'dist/.openai/hosting.json').read_text()) == hosting
provenance = json.loads((SITE / 'source-provenance.json').read_text())
assert provenance['reviewedCodeCommit'] == CODE and provenance['reviewedEvidenceCommit'] == EVIDENCE
assert provenance['runtimeManifestSha256'] == sha(manifest_path.read_bytes())
assert provenance['runtimeFileCount'] == 166
allowed = set(plan) | set(retained_files) | {'source-provenance.json'}
changes = git('diff', '--name-only', '906a1b4c54e43eff5dd722c275d55ad567b8c20c', cwd=SITE).splitlines()
untracked = git('ls-files', '--others', '--exclude-standard', cwd=SITE).splitlines()
assert all(name in allowed for name in changes + untracked), sorted(set(changes + untracked) - allowed)
subprocess.check_call(['git', 'diff', '--check'], cwd=SITE)
report = {
    'checkedAtUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'reviewedCodeCommit': CODE, 'evidenceCommit': EVIDENCE,
    'copiedSourceFiles': len(copied), 'sourceFilesMatchingReviewedGit': len(sources),
    'nativeInputFiles': 125, 'runtimeFiles': 166, 'runtimeManifestSha256': sha(manifest_path.read_bytes()),
    'current80WasmSha256': provenance['wasmSha256'],
    'packageDifferences': 'Only original root/web fixture restoration prebuild/pretest hooks retained.',
    'oldFixtureByteIdentityPreserved': True, 'hostingAndWranglerUnchanged': True,
    'migrationsExactlyUnchanged': migrations, 'unknownChangedProviderFiles': [],
    'changedProviderFiles': changes, 'newProviderFiles': untracked,
    'runtimeFilesExact': runtime,
}
(OUT / 'provider-input-audit.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
(OUT / 'provider-source-files.json').write_text(json.dumps(copied, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: report[k] for k in [
    'reviewedCodeCommit', 'evidenceCommit', 'copiedSourceFiles', 'sourceFilesMatchingReviewedGit',
    'nativeInputFiles', 'runtimeFiles', 'current80WasmSha256', 'unknownChangedProviderFiles',
]}))
