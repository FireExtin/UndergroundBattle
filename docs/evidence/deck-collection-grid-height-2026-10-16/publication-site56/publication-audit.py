import argparse
import gzip
import hashlib
import json
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path('/workspace/UndergroundBattle')
SITE = pathlib.Path('/workspace/hidden-world-sites-abi77-20261015')
OUT = pathlib.Path('/workspace/collection-layout-20261016/publication')
REVIEWED = '3492f7775cfd470f18164c9dfe847b3c2e3cb7ec'
CANONICAL = '7146467ef1994debd9613e2f3a0f1d2a45b37632'
PRIOR = pathlib.Path('/workspace/deck-collection-ui-20261016/publication/source-provider-binding.json')


def git(directory, *args):
    return subprocess.check_output(['git', *args], cwd=directory, text=True).strip()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, value):
    OUT.mkdir(exist_ok=True)
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def decoded(entry):
    data = (SITE / entry['providerStoredPath']).read_bytes()
    return gzip.decompress(data) if entry['encoding'].startswith('gzip') else data


def canonical_guard():
    assert git(ROOT, 'rev-parse', 'HEAD') == CANONICAL
    assert not git(ROOT, 'status', '--porcelain')
    assert git(ROOT, 'rev-parse', f'{REVIEWED}:web') == git(ROOT, 'rev-parse', f'{CANONICAL}:web')


def sync():
    canonical_guard()
    prior = json.loads(PRIOR.read_text())
    assert not git(SITE, 'status', '--porcelain')
    assert git(SITE, 'rev-parse', 'HEAD') == prior['providerCurrentHead']
    for entry in prior['files']:
        assert sha(decoded(entry)) == entry['sha256'], entry['providerStoredPath']
    changes = []
    for line in git(ROOT, 'diff', '--name-status', prior['canonicalCommit'], CANONICAL,
                    '--', 'web', 'sites', 'rust-game', 'rust-game-wasm', 'tools/build-storage.mjs').splitlines():
        status, name = line.split('\t')
        assert status in ['A', 'M'] and name.startswith('web/src/game/'), line
        data = (ROOT / name).read_bytes()
        assert (SITE / name).parent.is_dir()
        changes.append({'status': status, 'canonicalPath': name, 'providerPath': name,
                        'bytes': len(data), 'sha256': sha(data)})
    assert len(changes) == 2
    assert {entry['canonicalPath'] for entry in changes} == {'web/src/game/DeckLibraryPanel.tsx', 'web/src/game/lobby-pages.css'}
    plan = {'canonicalCommit': CANONICAL, 'reviewedSourceCommit': REVIEWED,
            'providerBefore': prior['providerCurrentHead'], 'priorBoundFilesVerified': len(prior['files']),
            'providerCleanBefore': True, 'changes': changes}
    write('source-sync-plan.json', plan)
    for entry in changes:
        shutil.copyfile(ROOT / entry['canonicalPath'], SITE / entry['providerPath'])
    provenance = json.loads((SITE / 'source-provenance.json').read_text())
    provenance['originalCommit'] = CANONICAL
    provenance['reviewedSourceCommit'] = REVIEWED
    (SITE / 'source-provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    bind()
    print(json.dumps({'priorVerifiedFiles': len(prior['files']), 'syncedWebFiles': len(changes),
                      'canonicalCommit': CANONICAL, 'siteHeadBefore': prior['providerCurrentHead']}))


def bind():
    canonical_guard()
    prior = json.loads(PRIOR.read_text())
    changes = json.loads((OUT / 'source-sync-plan.json').read_text())['changes']
    entries = {entry['canonicalPath']: dict(entry) for entry in prior['files']}
    for entry in changes:
        if entry['canonicalPath'] not in entries:
            entries[entry['canonicalPath']] = {'canonicalPath': entry['canonicalPath'],
                'providerPath': entry['providerPath'], 'providerStoredPath': entry['providerPath'],
                'encoding': 'identity'}
    result = []
    for name, entry in sorted(entries.items()):
        canonical = (ROOT / name).read_bytes()
        assert canonical == decoded(entry), name
        entry.update(bytes=len(canonical), sha256=sha(canonical), matches=True)
        result.append(entry)
    adaptations = prior['packageManifestAdaptations']
    for entry in adaptations:
        canonical = json.loads((ROOT / entry['canonicalPath']).read_text())
        provider = json.loads((SITE / entry['providerPath']).read_text())
        for key, value in entry['onlyAddedNpmHooks'].items():
            assert provider['scripts'].pop(key) == value
        assert canonical == provider, entry['providerPath']
    scope = ['web/src/', 'web/public/', 'rust-game/src/', 'rust-game-wasm/src/',
             'sites/src/', 'sites/db/', 'sites/drizzle/']
    tracked = git(ROOT, 'ls-files').splitlines()
    uncovered = [name for name in tracked if any(name.startswith(s) for s in scope) and name not in entries]
    assert not uncovered, uncovered
    report = {'canonicalCommit': CANONICAL, 'reviewedSourceCommit': REVIEWED,
              'providerBefore': prior['providerCurrentHead'], 'providerCurrentHead': git(SITE, 'rev-parse', 'HEAD'),
              'matchedFiles': len(result), 'allComparedFileBytesMatch': True,
              'coverageRoots': scope, 'uncoveredTrackedFiles': uncovered, 'files': result,
              'packageManifestAdaptations': adaptations,
              'providerSourceProvenance': json.loads((SITE / 'source-provenance.json').read_text())}
    write('source-provider-binding.json', report)
    print(json.dumps({'matchedFiles': len(result), 'allComparedFileBytesMatch': True}))


def artifacts():
    canonical_guard()
    frozen = json.loads(gzip.decompress((ROOT / 'docs/evidence/deck-collection-grid-height-2026-10-16/artifact-and-source-binding.json.gz').read_bytes()))
    assert frozen['source'] == REVIEWED
    for entry in frozen['frontendFiles']:
        data = (ROOT / 'sites/dist/client' / entry['path']).read_bytes()
        assert sha(data) == entry['sha256'] and len(data) == entry['bytes'], entry['path']
    for entry in frozen['runtime']:
        data = (ROOT / entry['path']).read_bytes()
        assert sha(data) == entry['sha256'] and len(data) == entry['bytes'], entry['path']
    files = []
    for source in sorted((ROOT / 'sites/dist/client').rglob('*')):
        if not source.is_file():
            continue
        relative = source.relative_to(ROOT / 'sites/dist/client')
        candidate = SITE / 'dist/client' / relative
        data = source.read_bytes()
        assert data == candidate.read_bytes(), str(relative)
        files.append({'path': str(relative), 'bytes': len(data), 'sha256': sha(data)})
    actual = sorted(str(p.relative_to(SITE / 'dist/client')) for p in (SITE / 'dist/client').rglob('*') if p.is_file())
    assert actual == sorted(f['path'] for f in files)
    runtime = []
    for name in ['dist/server/index.js', 'generated/hegemony_wasm_bg.wasm',
                 'dist/server/9b2c4d3e4e9562fdb0bd7c078c66341ea6cdea39-hegemony_wasm_bg.wasm']:
        data = (SITE / name).read_bytes()
        assert data == (ROOT / 'sites' / name).read_bytes(), name
        runtime.append({'path': name, 'bytes': len(data), 'sha256': sha(data)})
    report = {'canonicalCommit': CANONICAL, 'reviewedSourceCommit': REVIEWED,
              'canonicalAndProviderBuiltClientBytesIdentical': True,
              'clientFileCount': len(files), 'clientFiles': files, 'runtime': runtime,
              'hostingManifestIdentical': (ROOT / 'sites/.openai/hosting.json').read_bytes() == (SITE / 'dist/.openai/hosting.json').read_bytes()}
    assert report['hostingManifestIdentical']
    write('prepublication-artifacts.json', report)
    print(json.dumps({'clientFilesMatched': len(files), 'runtime': runtime, 'hostingManifestIdentical': True}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['sync', 'bind', 'artifacts'])
    action = parser.parse_args().action
    {'sync': sync, 'bind': bind, 'artifacts': artifacts}[action]()
