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
OUTPUT = pathlib.Path('/workspace/jc035-recovery-20261019/runs')
OUTPUT.mkdir(exist_ok=True)
EXPECTED = json.loads((REPO / 'docs/evidence/jc035-2026-10-19/runs/native-release-workspace-final.inputs-before.json').read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot():
    compiler = {name: digest(REPO / name) for name in EXPECTED}
    tests = {str(p.relative_to(REPO)): digest(p) for p in sorted((REPO / 'sites/test').glob('*.mjs'))}
    retained = json.loads(pathlib.Path('/workspace/jc035-recovery-20261019/runtime-restore.json').read_text())['files']
    runtimes = {item['path']: digest(pathlib.Path(item['path'])) for item in retained}
    return {'compiler': compiler, 'workerTestSources': tests, 'exactRuntimeFiles': runtimes}


def resources():
    import os
    filesystem = os.statvfs('/workspace')
    result = {'workspaceAvailableBytes': filesystem.f_bavail * filesystem.f_frsize}
    for name in ['memory.current', 'memory.max', 'memory.events', 'cpu.stat']:
        path = pathlib.Path('/sys/fs/cgroup') / name
        if path.exists():
            result[name] = path.read_text().strip()
    return result


stage, *command = sys.argv[1:]
if command and command[0] == '--':
    command.pop(0)
if not re.fullmatch(r'[a-z0-9-]+', stage) or not command:
    raise SystemExit('Provide a unique stage and one literal argv command')
log_path = OUTPUT / (stage + '.log')
receipt_path = OUTPUT / (stage + '.exit.json')
if log_path.exists() or receipt_path.exists():
    raise SystemExit('Preserve existing run artifacts; choose a new stage')
before = snapshot()
if before['compiler'] != EXPECTED:
    raise SystemExit('Compiler inputs differ from the fixed 141-input witness; no child launched')
source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
baseline = resources()
(OUTPUT / (stage + '.inputs-before.json')).write_text(json.dumps(before, sort_keys=True, indent=2) + '\n')
receipt = {'stage': stage, 'sourceAtLaunch': source, 'command': command, 'cwd': str(REPO), 'startedUtc': started, 'log': str(log_path), 'childTimeout': None, 'retryAdded': False, 'nativeCompilerInputs': len(EXPECTED), 'resourcesBefore': baseline}
receipt['recorderSha256'] = digest(pathlib.Path(__file__))
receipt['environment'] = {key: os.environ.get(key) for key in ['HEGEMONY_CURRENT_WORKER_DIR', 'UNDERGROUNDBATTLE_BUILD_ROOT', 'CARGO_HOME', 'RUSTUP_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL', 'TMPDIR', 'WRANGLER_SEND_METRICS']}
(OUTPUT / (stage + '.started.json')).write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'stage': stage, 'state': 'started', 'source': source, 'startedUtc': started, 'log': str(log_path)}), flush=True)
monotonic_start = time.monotonic()
with log_path.open('xb') as log:
    result = subprocess.run(command, cwd=REPO, stdout=log, stderr=subprocess.STDOUT)
after = snapshot()
(OUTPUT / (stage + '.inputs-after.json')).write_text(json.dumps(after, sort_keys=True, indent=2) + '\n')
text = log_path.read_text(errors='replace')
summary = {}
for key, value in re.findall(r'^[ℹ#]\s*(tests|suites|pass|fail|cancelled|skipped|todo|duration_ms)\s+([0-9.]+)\s*$', text, re.MULTILINE):
    summary[key] = float(value) if '.' in value else int(value)
unchanged = before == after
receipt.update({'finishedUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'naturalExitCode': result.returncode, 'durationSeconds': time.monotonic() - monotonic_start, 'logSha256': digest(log_path), 'inputsUnchangedDuringRun': unchanged, 'nativeCompilerInputsMatchPinnedWitness': after['compiler'] == EXPECTED, 'resourcesAfter': resources(), 'nodeTestSummary': summary, 'guard': 'Unchanged tools/build-storage.mjs; ordinary workspace disk target/TMPDIR, jobs1, incremental0, 1GiB disk/memory startup minimum.'})
receipt_path.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'stage': stage, 'state': 'finished', 'naturalExitCode': result.returncode, 'inputsUnchanged': unchanged, 'durationSeconds': receipt['durationSeconds'], 'nodeTestSummary': summary, 'receipt': str(receipt_path)}), flush=True)
if result.returncode:
    print(text[-7000:], flush=True)
sys.exit(result.returncode if unchanged else 73)
