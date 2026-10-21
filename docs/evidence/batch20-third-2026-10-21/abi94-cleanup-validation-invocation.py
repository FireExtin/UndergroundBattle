from pathlib import Path
import os, subprocess, time, json, sys

repo = Path.cwd()
root = repo.parent
assert repo.name == 'UndergroundBattle-batch3'
env = dict(os.environ)
env.update(PATH=str(root / '.tools/node-v24.22.1-linux-x64/bin') + ':' + env['PATH'],
           RAYON_NUM_THREADS='2', UV_THREADPOOL_SIZE='2', WRANGLER_SEND_METRICS='false')
mode = sys.argv[1]
tasks = {
    'build': [
        ('cross-inspections', repo, ['node', 'tools/check-batch3-abi94-smoke.mjs', str(root / 'batch3-abi94-native-smoke-cleanup.json')]),
        ('cross-cleanup', repo, ['node', 'tools/check-batch3-abi94-smoke.mjs', str(root / 'batch3-abi94-native-cleanup-p2.json'), 'docs/evidence/batch20-third-2026-10-21/abi94-bq026-cleanup.current.json.gz']),
        ('sites-build', repo / 'sites', ['npm', 'run', 'build']),
        ('clock-build', repo, ['node', 'tools/build-current-clock-worker.mjs', 'sites/dist/current-clock']),
    ],
    'sites': [
        ('sites-targeted', repo / 'sites', ['node', '--test', '--test-isolation=none', '--test-concurrency=1',
           'test/kernel-binding.test.mjs', 'test/kernel-routing.test.mjs', 'test/current-kernel-http.test.mjs',
           'test/batch3-abi94-integration.test.mjs', 'test/storage-retry.test.mjs']),
    ],
}
results = []
for label, cwd, command in tasks[mode]:
    log = root / f'batch3-cleanup-p2-{label}.log'
    assert not log.exists(), 'Preserve previous results; do not overwrite logs'
    start = time.monotonic()
    with log.open('w') as output:
        result = subprocess.run(command, cwd=cwd, env=env, stdout=output, stderr=subprocess.STDOUT)
    results.append(dict(label=label, command=command, cwd=str(cwd), exitCode=result.returncode,
                        seconds=round(time.monotonic() - start, 3), log=log.name))
    print(json.dumps(results[-1]), flush=True)
    if result.returncode:
        break
record = root / f'batch3-cleanup-p2-{mode}-results.json'
assert not record.exists()
record.write_text(json.dumps(results, indent=2) + '\n')
sys.exit(results[-1]['exitCode'])
