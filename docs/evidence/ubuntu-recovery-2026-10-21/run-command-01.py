import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
import time

root = Path(__file__).resolve().parent
name, directory, *command = sys.argv[1:]
evidence = root / 'recovery-evidence'
evidence.mkdir(exist_ok=True)
log_path = evidence / (name + '.log')
meta_path = evidence / (name + '.json')
if log_path.exists() or meta_path.exists():
    raise SystemExit('Evidence already exists; choose a new run name.')
env = dict(os.environ)
env.update({
    'PATH': str(root / '.tools/node-v24.22.1-linux-x64/bin') + ':' + str(root / '.tools/cargo/bin') + ':' + env['PATH'],
    'CARGO_HOME': str(root / '.tools/cargo'),
    'RUSTUP_HOME': str(root / '.tools/rustup'),
    'CC': str(root / '.tools/zig-cc'),
    'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER': str(root / '.tools/zig-cc'),
    'ZIG_GLOBAL_CACHE_DIR': str(root / '.tools/zig-global-cache'),
    'ZIG_LOCAL_CACHE_DIR': str(root / '.tools/zig-local-cache'),
    'WASM_BINDGEN': str(root / '.tools/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen'),
    'UNDERGROUNDBATTLE_BUILD_ROOT': str(root / 'recovery-build'),
    'WRANGLER_SEND_METRICS': 'false',
})
if os.environ.get('RECOVERY_CURRENT_CLOCK_WORKER'):
    env['CURRENT_CLOCK_WORKER'] = os.environ['RECOVERY_CURRENT_CLOCK_WORKER']
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
tic = time.monotonic()
with log_path.open('xb') as log:
    result = subprocess.run(command, cwd=directory, env=env, stdout=log, stderr=subprocess.STDOUT)
metadata = dict(name=name, cwd=directory, command=command, startedAt=started,
                endedAt=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                durationSeconds=round(time.monotonic()-tic, 3), exitCode=result.returncode,
                log=str(log_path))
meta_path.write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(metadata, ensure_ascii=False), flush=True)
print('\n'.join(log_path.read_text(errors='replace').splitlines()[-25:]), flush=True)
sys.exit(result.returncode)
