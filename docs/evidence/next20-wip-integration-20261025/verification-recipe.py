#!/usr/bin/env python3
"""Offline validation with disk-backed outputs, without production wiring."""
import ast
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent
MAIN = Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
TOOLS = MAIN.parent / '.tools'
WORK = ROOT / 'next20-validation-20261025'
assert not WORK.exists(), 'Inspect prior validation rather than overwriting it'
WORK.mkdir()
STATE = WORK / 'validation.json'
MERGE = json.loads((ROOT / 'NEXT20_MERGE_RECEIPT_20261025.json').read_text())
assert MERGE['state'] == 'MERGED_AS_DISABLED_WIP'
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=MAIN, text=True).strip() == MERGE['head']
os.sched_setaffinity(0, [31])
os.nice(19)
toolchain = TOOLS / 'rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin'
env = dict(os.environ)
env.update(PATH=str(toolchain) + ':' + str(TOOLS / 'node-v24.22.1-linux-x64/bin') + ':' + env['PATH'],
           CARGO_HOME=str(WORK / 'cargo'), CARGO_TARGET_DIR=str(WORK / 'build/target'),
           UNDERGROUNDBATTLE_BUILD_ROOT=str(WORK / 'build'), CARGO_BUILD_JOBS='1',
           RUST_TEST_THREADS='1', CARGO_NET_OFFLINE='true', NPM_CONFIG_OFFLINE='true',
           WRANGLER_SEND_METRICS='false', CC=str(TOOLS / 'zig-cc'),
           CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=str(TOOLS / 'zig-cc'),
           ZIG_LOCAL_CACHE_DIR=str(WORK / 'zig-local'), ZIG_GLOBAL_CACHE_DIR=str(WORK / 'zig-global'))
state = {'state': 'running', 'source_head': MERGE['head'], 'main_repo': str(MAIN),
         'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
         'offline': True, 'affinity': [31], 'jobs': 1, 'tests_threads': 1,
         'runtime_wiring': 'NOT_ENABLED', 'checks': [], 'runs': [],
         'fixture_scope': 'Temporary limited wiring and test catalogue only; not production card admission'}

def save():
    STATE.write_text(json.dumps(state, ensure_ascii=False, indent=2) + '\n')

def run(name, argv, cwd=MAIN):
    log = WORK / (name + '.log')
    entry = {'name': name, 'argv': [str(x) for x in argv], 'cwd': str(cwd),
             'state': 'running', 'log': str(log)}
    state['runs'].append(entry)
    save()
    started = time.monotonic()
    with log.open('xb') as out:
        result = subprocess.run(argv, cwd=cwd, env=env, stdout=out, stderr=subprocess.STDOUT)
    entry.update(state='complete', exit_code=result.returncode,
                 seconds=round(time.monotonic() - started, 3),
                 log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
    save()
    print(json.dumps(entry, ensure_ascii=False), flush=True)
    print('\n'.join(log.read_text(errors='replace').splitlines()[-6:]), flush=True)
    return result.returncode

save()
try:
    # Only the non-secret registry is copied, never the original Cargo configuration.
    shutil.copytree(TOOLS / 'cargo/registry', WORK / 'cargo/registry')
    harness = MAIN / 'docs/NEXT20_BLUE_VAMPIRES_HARNESS_2026-10-25.py'
    original = harness.read_text()
    ast.parse(original)
    old = "root=Path('/tmp/next20-blue-black-harness')"
    assert original.count(old) == 1
    fixture = WORK / 'fixture'
    adapted = original.replace(old, 'root=Path(' + repr(str(fixture)) + ')')
    adapted_file = WORK / 'isolated-harness.py'
    adapted_file.write_text(adapted)
    state['checks'].append({'name': 'harness_syntax_and_path_redirect', 'status': 'PASS',
                            'original_sha256': hashlib.sha256(harness.read_bytes()).hexdigest(),
                            'only_change': 'Hardcoded fixture output root redirected to a fresh disk-backed validation folder'})
    sources = [MAIN / 'rust-game/src' / n for n in ['next20_blue_vampires.rs', 'next20_blue_vampires_tests.rs',
                                                       'next20_black_wraiths.rs', 'next20_black_wraiths_tests.rs']]
    run('rustfmt-check', [toolchain / 'rustfmt', '--check', '--edition', '2021', *sources])
    run('patch-apply-check-only', ['git', 'apply', '--check', 'docs/NEXT20_BLUE_VAMPIRES_PARTIAL_CORE_WIRING_2026-10-25.patch'])
    node = TOOLS / 'node-v24.22.1-linux-x64/bin/node'
    wrapper = MAIN / 'tools/build-storage.mjs'
    run('main-workspace-all-targets-check', [node, wrapper, '--', toolchain / 'cargo', 'check', '--offline', '--locked', '--workspace', '--all-targets'])
    if run('fixture-generation', ['python3', adapted_file, MAIN]) == 0:
        for name, scope in [('isolated-blue', 'next20_blue_vampires_tests::'), ('isolated-black', 'next20_black_wraiths_tests::')]:
            run(name, [node, wrapper, '--', toolchain / 'cargo', 'test', '--offline', '--locked',
                       '--manifest-path', fixture / 'rust-game/Cargo.toml', '--lib', '--no-default-features',
                       scope, '--', '--test-threads=1'], cwd=fixture)
    state['state'] = 'complete'
    state['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    state['all_commands_passed'] = all(item['exit_code'] == 0 for item in state['runs'])
    save()
    print(json.dumps({'state': state['state'], 'all_commands_passed': state['all_commands_passed'], 'receipt': str(STATE)}), flush=True)
except BaseException as error:
    state.update(state='failed', error=repr(error))
    save()
    raise
