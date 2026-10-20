import datetime
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
import time

REPO = pathlib.Path('/workspace/UndergroundBattle')
ROOT = pathlib.Path(__file__).parent
RUNS = ROOT / 'runs'
RUNS.mkdir(exist_ok=True)
EXPECTED = json.loads((ROOT / 'after-source-141.json').read_text())
RESTORED = json.loads((ROOT.parent / 'runtime-restore.json').read_text())['files']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree(path):
    return {str(p.relative_to(path)): sha(p) for p in sorted(path.rglob('*')) if p.is_file()} if path.exists() else {}


def snapshot():
    compiler = {name: sha(REPO / name) for name in EXPECTED}
    extra = list((REPO / 'sites/test').glob('*.mjs')) + list((REPO / 'rust-game-wasm/tests').glob('*.mjs'))
    extra += list((REPO / 'tools').glob('*oracle.mjs'))
    extra += [REPO / n for n in ['sites/package-lock.json', 'web/package-lock.json', 'web/src/game/JC035Native89.test.jsx']]
    frozen = {item['path']: sha(pathlib.Path(item['path'])) for item in RESTORED if item['abi'] < 89}
    for item in RESTORED:
        if item['abi'] < 89:
            assert frozen[item['path']] == item['sha256'], 'Historical kernel changed: ' + item['path']
    worker = os.environ.get('HEGEMONY_CURRENT_WORKER_DIR')
    return {
        'compiler141': compiler,
        'extraSources': {str(p.relative_to(REPO)): sha(p) for p in sorted(set(extra))},
        'frozen77to88': frozen,
        'pkg89': tree(REPO / 'rust-game-wasm/pkg'),
        'current89Witnesses': {str(p.relative_to(REPO)): sha(p) for p in sorted((REPO / 'rust-game-wasm/tests').glob('*89*.fixture.json.gz'))},
        'productionSites': tree(REPO / 'sites/dist'),
        'productionWeb': tree(REPO / 'web/dist'),
        'testWorker': tree(pathlib.Path(worker)) if worker else {},
    }


stage, *command = sys.argv[1:]
allowed = set()
while command and command[0] == '--artifact-writes':
    command.pop(0)
    allowed.update(command.pop(0).split(','))
if command and command[0] == '--':
    command.pop(0)
assert re.fullmatch(r'[a-z0-9-]+', stage) and command
assert allowed <= {'pkg89', 'current89Witnesses', 'productionSites', 'productionWeb', 'testWorker'}
base = RUNS / stage
assert not base.with_suffix('.log').exists() and not base.with_suffix('.exit.json').exists(), 'Preserve existing run'
before = snapshot()
assert len(EXPECTED) == 141 and before['compiler141'] == EXPECTED, 'New source witness does not match'
patch = subprocess.check_output(['git', 'diff', '--binary', 'HEAD'], cwd=REPO)
receipt = {
    'stage': stage, 'startedUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'sourceHead': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
    'worktreeDiffSha256': hashlib.sha256(patch).hexdigest(),
    'sourceWitnessSha256': sha(ROOT / 'after-source-141.json'),
    'recorderSha256': sha(pathlib.Path(__file__)), 'command': command, 'cwd': str(REPO),
    'allowedArtifactWrites': sorted(allowed), 'childTimeout': None, 'retryAdded': False,
    'environment': {name: os.environ.get(name) for name in ['CARGO_HOME', 'RUSTUP_HOME', 'CARGO_TARGET_DIR', 'TMPDIR', 'CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL', 'HEGEMONY_CURRENT_WORKER_DIR']},
}
base.with_suffix('.inputs-before.json').write_text(json.dumps(before, sort_keys=True, indent=2) + '\n')
base.with_suffix('.started.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'stage': stage, 'state': 'started', 'sourceHead': receipt['sourceHead'], 'log': str(base.with_suffix('.log'))}), flush=True)
start = time.monotonic()
with base.with_suffix('.log').open('xb') as log:
    result = subprocess.run(command, cwd=REPO, stdout=log, stderr=subprocess.STDOUT)
after = snapshot()
base.with_suffix('.inputs-after.json').write_text(json.dumps(after, sort_keys=True, indent=2) + '\n')
changed = [name for name in before if before[name] != after[name]]
unexpected = sorted(set(changed) - allowed)
text = base.with_suffix('.log').read_text(errors='replace')
node = {key: float(value) if '.' in value else int(value) for key, value in re.findall(r'^[ℹ#]\s*(tests|suites|pass|fail|cancelled|skipped|todo|duration_ms)\s+([0-9.]+)\s*$', text, re.M)}
native = [dict(zip(['status', 'pass', 'fail', 'ignored', 'measured', 'filtered'], values)) for values in re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', text)]
receipt.update({'finishedUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'naturalExitCode': result.returncode,
                'durationSeconds': time.monotonic() - start, 'logSha256': sha(base.with_suffix('.log')),
                'source141AndFrozen77to88Unchanged': before['compiler141'] == after['compiler141'] == EXPECTED and before['frozen77to88'] == after['frozen77to88'],
                'changedSnapshotSections': changed, 'unexpectedChangedSections': unexpected,
                'nodeTestSummary': node, 'nativeTestSummaries': native})
base.with_suffix('.exit.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'stage': stage, 'state': 'finished', 'exit': result.returncode, 'unexpectedChanges': unexpected, 'node': node, 'native': native, 'durationSeconds': receipt['durationSeconds']}), flush=True)
if result.returncode:
    print(text[-9000:], flush=True)
sys.exit(result.returncode if not unexpected else 73)
