#!/usr/bin/env python3
"""Run complete UI games on an owned final v2 service, recording binary identity.

Only the browser runner submits gameplay; this wrapper starts/stops its own PID.
"""
import argparse
import hashlib
import subprocess
import sys
from pathlib import Path

from common import IsolatedService, write_json


def main(args):
    output = Path(args.output).resolve()
    binary = Path(args.binary).resolve()
    metadata = {'testType': 'complete-browser-ui-on-owned-v2-service', 'binary': str(binary),
                'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'port': args.port,
                'apiGameplayCommands': 0, 'responseDeckRequested': True, 'modes': args.modes}
    service = IsolatedService(str(binary), args.cwd, output, args.port, args.static_dir)
    service.start()
    metadata['ownedPid'] = service.process.pid
    write_json(output / 'binary-and-service.json', metadata)
    try:
        runner = Path(__file__).with_name('browser_playtest.py').resolve()
        result = subprocess.run([sys.executable, str(runner), '--base-url', service.base_url, '--output', str(output),
                                 '--modes', args.modes, '--response-deck', '--max-steps', str(args.max_steps), '--chromium', args.chromium], cwd=args.cwd)
        metadata['runnerExitCode'] = result.returncode
        metadata['passed'] = result.returncode == 0
        if result.returncode:
            raise RuntimeError(f'Complete UI runner returned {result.returncode}; original traces preserved')
    finally:
        service.stop()
        metadata['ownedServiceStopped'] = True
        write_json(output / 'binary-and-service.json', metadata)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default='/tmp/hegemony-response-v2-2026-10-02/bin/final/hegemony-server')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-playtest-final-v2-2026-10-02')
    parser.add_argument('--port', type=int, default=8097)
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--modes', default='duel,teams')
    parser.add_argument('--max-steps', type=int, default=7000)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    main(parser.parse_args())
