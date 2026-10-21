import os
from pathlib import Path
import selectors
import signal
import subprocess
import tempfile
import time

root = Path(__file__).resolve().parent
state = tempfile.mkdtemp(prefix='qa-preview-exact-', dir=root / 'recovery-build')
env = dict(os.environ, PORT='0', HEGEMONY_QA_STATE_DIR=state)
child = subprocess.Popen(['node', 'scripts/qa-preview.mjs'], env=env,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
selector = selectors.DefaultSelector()
selector.register(child.stdout, selectors.EVENT_READ)
ready = False
engine = False
deadline = time.monotonic() + 30
buffer = ''
forced = False
try:
    while time.monotonic() < deadline and child.poll() is None:
        if not selector.select(1):
            continue
        chunk = os.read(child.stdout.fileno(), 65536)
        if not chunk:
            break
        buffer += chunk.decode(errors='replace')
        lines = buffer.split('\n')
        buffer = lines.pop()
        for line in lines:
            print(line, flush=True)
            ready |= line.startswith('READY http://127.0.0.1:')
            engine |= line.startswith('ENGINE rust-v0.2.93-batch20-next ')
        if ready and engine:
            break
finally:
    if child.poll() is None:
        child.send_signal(signal.SIGINT)
    try:
        code = child.wait(timeout=10)
    except subprocess.TimeoutExpired:
        forced = True
        child.terminate()
        code = child.wait(timeout=10)
    print(buffer + child.stdout.read().decode(errors='replace'), end='')
    selector.close()
if not ready or not engine or forced or code not in (0, 130, -signal.SIGINT):
    raise SystemExit(f'QA readiness failed: ready={ready}, engine={engine}, exit={code}')
print(f'Local QA health, complete HTTP catalog and original ABI93 identity assertions passed; owned server stopped (exit {code}). State: {state}')
