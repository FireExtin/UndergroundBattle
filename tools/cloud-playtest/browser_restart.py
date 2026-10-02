#!/usr/bin/env python3
"""Browser reconnection to its exact pending choice after an owned service restart.

The room is an explicitly labeled API fixture; no gameplay success is attributed
to this script. The normal playtest creates and plays rooms entirely through UI.
"""
import argparse
import json
import uuid
from pathlib import Path

from playwright.sync_api import sync_playwright

from browser_playtest import OBSERVE_FETCH
from common import Api, IsolatedService, public_view, write_json


def main(args):
    output = Path(args.output).resolve()
    service = IsolatedService(args.binary, args.cwd, output, args.port, args.static_dir)
    service.start()
    try:
        api = Api(service.base_url)
        status, catalog = api.request('GET', '/api/catalog')
        assert status == 200
        status, host = api.request('POST', '/api/rooms', {'name': 'Recovery A', 'mode': 'duel', 'deckId': catalog['decks'][0]['id']})
        assert status == 200
        status, guest = api.request('POST', '/api/rooms/join', {'name': 'Recovery B', 'inviteCode': host['inviteCode'], 'deckId': catalog['decks'][1]['id']})
        assert status == 200
        for session, action in ((host, {'kind': 'ready'}), (guest, {'kind': 'ready'}), (host, {'kind': 'start'})):
            view = api.state(session)
            status, _ = api.request('POST', f'/api/rooms/{session["roomId"]}/commands', {'commandId': str(uuid.uuid4()), 'expectedVersion': view['version'], 'action': action}, token=session['token'])
            assert status == 200
        before = api.state(host)
        choice_id = before['pendingChoice']['id']
        credentials = {key: host[key] for key in ('roomId', 'inviteCode', 'token', 'seat')}
        errors = []
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
            context = browser.new_context(viewport={'width': 1512, 'height': 1040})
            context.add_init_script(OBSERVE_FETCH)
            # Fixture credentials remain in this browser's local storage only.
            context.add_init_script('localStorage.setItem("hegemony.session.v1", ' + json.dumps(json.dumps(credentials)) + ');')
            page = context.new_page()
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.goto(service.base_url, wait_until='domcontentloaded')
            page.wait_for_function('(id) => window.__cloudView && window.__cloudView.pendingChoice?.id === id', arg=choice_id)
            page.locator('.hg-choice').wait_for(state='visible')
            page.screenshot(path=str(output / 'restart-before-private-choice.png'), full_page=True)
            service.stop()
            page.get_by_text('连接中断 · 自动重连', exact=True).wait_for(timeout=22000)
            page.screenshot(path=str(output / 'restart-disconnected-private-choice.png'), full_page=True)
            service.start()
            page.get_by_text('牌桌已连接', exact=True).wait_for(timeout=30000)
            page.wait_for_function('(id) => window.__cloudView && window.__cloudView.pendingChoice?.id === id', arg=choice_id)
            restored = page.evaluate('window.__cloudView')
            assert restored == before, 'Browser reconnected to changed accepted state'
            page.screenshot(path=str(output / 'restart-restored-private-choice.png'), full_page=True)
            # Finish the surviving choice through its ordinary visible UI control.
            with page.expect_response(lambda response: response.request.method == 'POST' and '/commands' in response.url) as response_info:
                page.get_by_role('button', name='保留全部手牌', exact=True).click()
            response = response_info.value
            assert response.ok
            after = response.json()
            assert after['version'] == before['version'] + 1
            assert after.get('pendingChoice') is None
            result = {'testType': 'browser-recovery-with-isolated-api-fixture', 'passed': True,
                      'sameAcceptedViewAfterRestart': True, 'samePrivateChoiceAndInstanceIds': True,
                      'uiChoiceContinued': True, 'browserErrors': errors,
                      'before': public_view(before), 'after': public_view(after),
                      'screenshots': ['restart-before-private-choice.png', 'restart-disconnected-private-choice.png', 'restart-restored-private-choice.png']}
            write_json(output / 'browser-restart-summary.json', result)
            print(json.dumps({'passed': True, 'uiChoiceContinued': True, 'output': str(output)}), flush=True)
            browser.close()
    except Exception as error:
        write_json(output / 'browser-restart-summary.json', {'passed': False, 'failure': f'{type(error).__name__}: {error}'})
        raise
    finally:
        service.stop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default='target/debug/hegemony-server')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-browser-restart')
    parser.add_argument('--port', type=int, default=8092)
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    main(parser.parse_args())
