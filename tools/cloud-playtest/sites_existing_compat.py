#!/usr/bin/env python3
"""Hash owned existing projections around a deployment; replay only an existing receipt."""
import argparse
import asyncio
import getpass
import hashlib
import json
import os
from pathlib import Path
from playwright.async_api import async_playwright


def fingerprint(value):
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


async def main(args, payload):
    output = Path(args.output); output.mkdir(parents=True, exist_ok=True)
    result = {'phase': args.phase, 'passed': False, 'newGameplayCommands': 0, 'checks': []}
    async with async_playwright() as playwright:
        request = await playwright.request.new_context(proxy={'server': os.environ['HTTPS_PROXY']})
        try:
            for session in payload['sessions']:
                url = args.base_url + '/api/rooms/' + session['roomId']
                headers = {'Authorization': 'Bearer ' + session['token']}
                response = await request.get(url + '/state', headers=headers, max_redirects=0, timeout=20000)
                assert response.status == 200
                view = await response.json()
                assert view['you'] == 'p' + str(session['seat'])
                assert view['versions']['engine'] == 'rust-v0.2.1'
                result['checks'].append({'roomId': session['roomId'], 'seat': session['seat'],
                    'version': view['version'], 'engine': view['versions']['engine'], 'viewSha256': fingerprint(view)})
            if args.phase == 'after':
                before = json.loads((output / 'before.json').read_text())
                assert before['passed'] and before['checks'] == result['checks']
                result['allExistingViewsExact'] = True
                session = payload['sessions'][0]
                assert session['roomId'] == '8438277a645d91a656f57ce9' and session['seat'] == 0
                url = args.base_url + '/api/rooms/' + session['roomId']
                headers = {'Authorization': 'Bearer ' + session['token']}
                response = await request.post(url + '/commands', headers=headers, data=payload['command'], max_redirects=0, timeout=20000)
                assert response.status == 200
                receipt = await response.json()
                assert fingerprint(receipt) == result['checks'][0]['viewSha256']
                response = await request.get(url + '/state', headers=headers, max_redirects=0, timeout=20000)
                assert response.status == 200 and fingerprint(await response.json()) == fingerprint(receipt)
                result['originalReceiptExactWithoutStateChange'] = True
                response = await request.get(args.base_url + '/api/health', max_redirects=0, timeout=20000)
                assert response.status == 200
                health = await response.json(); assert health['engineVersion'] == 'rust-v0.2.2'
                result['newKernelHealth'] = health['engineVersion']
            result['passed'] = True
        except Exception as error:
            result['failureType'] = type(error).__name__
        finally:
            await request.dispose()
    (output / (args.phase + '.json')).write_text(json.dumps(result, ensure_ascii=False, indent=2))
    print(json.dumps(result, ensure_ascii=False), flush=True)
    return result['passed']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--phase', choices=['before', 'after'], required=True)
    args = parser.parse_args()
    payload = json.loads(getpass.getpass('Own existing seats and original command (input hidden): '))
    raise SystemExit(0 if asyncio.run(main(args, payload)) else 1)
