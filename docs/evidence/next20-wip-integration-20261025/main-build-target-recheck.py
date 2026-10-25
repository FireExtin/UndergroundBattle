#!/usr/bin/env python3
"""Retry only the failed native check with the project's target-normalizing CC."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parent
WORK = ROOT / 'next20-validation-20261025'
STATE = WORK / 'validation.json'
state = json.loads(STATE.read_text())
assert state['state'] == 'complete' and state['all_commands_passed'] is False
failed = [item for item in state['runs'] if item['exit_code'] != 0]
assert len(failed) == 1 and failed[0]['name'] == 'main-workspace-all-targets-check'
assert "UnknownOperatingSystem" in Path(failed[0]['log']).read_text()
assert not (WORK / 'validation-initial.json').exists()
(WORK / 'validation-initial.json').write_bytes(STATE.read_bytes())
main = Path(state['main_repo'])
tools = main.parent / '.tools'
toolchain = tools / 'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin'
env = dict(os.environ)
env.update(PATH=str(toolchain) + ':' + str(tools / 'node-v24.22.1-linux-x64/bin') + ':' + env['PATH'],
           CARGO_HOME=str(WORK / 'cargo'), CARGO_TARGET_DIR=str(WORK / 'build/target'),
           UNDERGROUNDBATTLE_BUILD_ROOT=str(WORK / 'build'), CARGO_BUILD_JOBS='1',
           RUST_TEST_THREADS='1', CARGO_NET_OFFLINE='true', NPM_CONFIG_OFFLINE='true',
           WRANGLER_SEND_METRICS='false', CC=str(tools / 'batch3-zig-cc'),
           CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(tools / 'batch3-zig-cc'),
           ZIG_LOCAL_CACHE_DIR=str(WORK / 'zig-local'), ZIG_GLOBAL_CACHE_DIR=str(WORK / 'zig-global'))
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=main, text=True).strip() == state['source_head']
os.sched_setaffinity(0, [31])
os.nice(19)
argv = [str(tools / 'node-v24.22.1-linux-x64/bin/node'), str(main / 'tools/build-storage.mjs'), '--',
        str(toolchain / 'cargo'), 'check', '--offline', '--locked', '--workspace', '--all-targets']
log = WORK / 'main-workspace-all-targets-check-target-normalized.log'
entry = {'name': 'main-workspace-all-targets-check-target-normalized', 'argv': argv, 'cwd': str(main),
         'state': 'running', 'log': str(log), 'CC': env['CC'],
         'reason': 'Use existing project wrapper to translate x86_64-unknown-linux-gnu into the Zig target syntax'}
state['runs'].append(entry)
state['state'] = 'running'
STATE.write_text(json.dumps(state, ensure_ascii=False, indent=2) + '\n')
started = time.monotonic()
with log.open('xb') as out:
    result = subprocess.run(argv, cwd=main, env=env, stdout=out, stderr=subprocess.STDOUT)
entry.update(state='complete', exit_code=result.returncode, seconds=round(time.monotonic() - started, 3),
             log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
state.update(state='complete', latest_effective_checks_passed=result.returncode == 0)
state['resolved_attempts'] = [{'name': failed[0]['name'], 'resolution': entry['name'],
                               'cause': 'C compiler wrapper target syntax, not a source change'}]
STATE.write_text(json.dumps(state, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(entry, ensure_ascii=False), flush=True)
print('\n'.join(log.read_text(errors='replace').splitlines()[-12:]), flush=True)
raise SystemExit(result.returncode)
