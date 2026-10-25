import datetime, gzip, hashlib, json, os, re, subprocess, tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
repo = Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
source = root / 'source'
pin = json.loads((root / 'frozen-source.json').read_text())
manifest = json.loads((root / 'candidate-fixture-manifest.json').read_text())
assert manifest['source_commit'] == pin['source_commit']
sha = lambda b: hashlib.sha256(b).hexdigest()
git = lambda *a: subprocess.check_output(['git', *a], cwd=repo)
assert git('rev-parse', 'HEAD').decode().strip() == pin['source_commit']
assert git('diff', 'HEAD') == b''
assert git('diff', '--cached') == b''
expected_untracked = set(pin['original32_sha256'])
untracked = set(git('ls-files', '--others', '--exclude-standard').decode().splitlines())
assert untracked == expected_untracked
for f, h in pin['original32_sha256'].items():
    assert sha((repo / f).read_bytes()) == h, f
    assert sha((source / f).read_bytes()) == h, f

matched = 0
for row in git('ls-files', '--stage', '-z').split(b'\0'):
    if not row:
        continue
    left, name = row.split(b'\t', 1)
    mode, expected, stage = left.decode().split()
    assert stage == '0'
    p = source / name.decode()
    data = os.readlink(p).encode() if mode == '120000' else p.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    assert blob == expected, name
    matched += 1

patterns = {
    'private_key': re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    'github_token': re.compile(rb'\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,})\b'),
    'aws_access_key': re.compile(rb'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b'),
    'google_api_key': re.compile(rb'\bAIza[0-9A-Za-z_-]{35}\b'),
    'credential_url': re.compile(rb'https?://[^\s/":@]+:[^\s/"@]+@'),
}
findings = []
def scan(name, data):
    for kind, pattern in patterns.items():
        if pattern.search(data):
            findings.append({'file': name, 'kind': kind})

total = expanded = raw_records = 0
formats = {}
def validate_records(value):
    global raw_records
    if isinstance(value, dict):
        if {'json', 'bytes', 'sha256'} <= value.keys() and isinstance(value['json'], str):
            raw = value['json'].encode()
            assert len(raw) == value['bytes'] and sha(raw) == value['sha256']
            raw_records += 1
        if 'sourceBindings' in value:
            binding = value['sourceBindings']
            assert binding['algorithm'] == 'sha256'
            for f, h in binding['files'].items():
                assert sha((source / f).read_bytes()) == h, f
        for v in value.values():
            validate_records(v)
    elif isinstance(value, list):
        for v in value:
            validate_records(v)

for relative, record in manifest['packets'].items():
    p = root / 'fixtures' / relative
    assert p.is_file() and not p.is_symlink()
    data = p.read_bytes()
    assert len(data) == record['bytes'] and sha(data) == record['sha256'], relative
    unpacked = gzip.decompress(data)
    scan(relative, unpacked)
    wrapped = json.loads(unpacked)
    assert isinstance(wrapped['format'], str)
    formats[wrapped['format']] = formats.get(wrapped['format'], 0) + 1
    assert wrapped['currentSourceManifestSha256'] == manifest['currentSourceManifestSha256']
    assert wrapped['wasmSha256'] == manifest['wasmSha256']
    before = raw_records
    validate_records(wrapped)
    assert raw_records > before, relative
    total += len(data)
    expanded += len(unpacked)
assert len(manifest['packets']) == 63
assert {str(p.relative_to(root / 'fixtures')) for p in (root / 'fixtures').rglob('*') if p.is_file()} == set(manifest['packets'])
for p in (root / 'logs').glob('*'):
    if p.is_file():
        scan('logs/' + p.name, p.read_bytes())
assert not findings, findings

other = Path('/home/ddd/Documents/Codex/2026-10-24/task-6/UndergroundBattle-next20-blue-black')
other_sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=other, text=True).strip()
other_status = subprocess.check_output(['git', 'status', '--porcelain=v1'], cwd=other, text=True)
assert other_sha == '83e564dcdda460e4603aaf2d86494da18085b80f'
assert not other_status

audit = {
    'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'source_commit': pin['source_commit'], 'tracked_source_blobs_verified': matched,
    'main_tracked_clean': True, 'original_untracked32_bytes_unchanged': True,
    'source_original32_bytes_unchanged': True,
    'canonical_fixture_files': len(manifest['packets']), 'compressed_bytes': total,
    'expanded_wrapper_bytes': expanded, 'fixture_hash_and_raw_checks': 'PASS',
    'raw_records_verified': raw_records, 'existing_fixture_formats_preserved': formats,
    'source_manifest': manifest['currentSourceManifestSha256'], 'wasm_sha256': manifest['wasmSha256'],
    'strong_secret_scan': {'fixture_wrappers_and_all_current_logs': 'PASS', 'findings': findings, 'patterns': list(patterns)},
    'archive_scope': '63 synthetic test data gzip files and two provenance manifests; no credentials, user databases, private data, build outputs, caches, or source tree',
    'donor': {'HEAD': other_sha, 'tracked_and_untracked_clean': not bool(other_status)},
}
(root / 'candidate-audit.json').write_text(json.dumps(audit, ensure_ascii=False, indent=2) + '\n')

archive = root.parent / 'checkpoints/next20-candidate-fixtures-20261025.tar.gz'
assert not archive.exists(), archive
with tarfile.open(archive, 'w:gz', compresslevel=1) as out:
    for relative in sorted(manifest['packets']):
        out.add(root / 'fixtures' / relative, arcname='fixtures/' + relative, recursive=False)
    for f in ['candidate-fixture-manifest.json', 'frozen-source.json', 'candidate-audit.json']:
        out.add(root / f, arcname=f, recursive=False)
with tarfile.open(archive) as check:
    assert len(check.getmembers()) == 66
    for entry in check:
        assert entry.isfile() and not entry.name.startswith('/') and '..' not in Path(entry.name).parts
archive_record = {'path': str(archive), 'bytes': archive.stat().st_size, 'sha256': sha(archive.read_bytes()), 'files': 66, 'source_commit': pin['source_commit']}
(root / 'fixture-archive.json').write_text(json.dumps(archive_record, indent=2) + '\n')
print(json.dumps({'audit': audit, 'archive': archive_record}, ensure_ascii=False, indent=2))
