import os
from pathlib import Path
import selectors
import signal
import subprocess
import time

root = Path(__file__).resolve().parent
env = dict(os.environ, PORT='0', HEGEMONY_QA_STATE_DIR=str(root / 'recovery-build/qa-preview-exact-state'))
child = subprocess.Popen(['node', 'scripts/qa-preview.mjs'], env=env,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                         text=True, bufsize=1)
selector = selectors.DefaultSelector()
selector.register(child.stdout, selectors.EVENT_READ)
ready = False
engine = False
deadline = time.monotonic() + 30
try:
    while time.monotonic() < deadline and child.poll() is None:
        if not selector.select(1):
            continue
        line = child.stdout.readline()
        print(line, end='', flush=True)
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
        child.terminate()
        code = child.wait(timeout=10)
    for line in child.stdout:
        print(line, end='')
    selector.close()
if not ready or not engine or code != 0:
    raise SystemExit(f'QA readiness failed: ready={ready}, engine={engine}, exit={code}')
print('Local QA health, complete HTTP catalog and original ABI93 identity assertions passed; owned server stopped.')
