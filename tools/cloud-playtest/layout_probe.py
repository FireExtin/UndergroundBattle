#!/usr/bin/env python3
"""Latest-build layout check with an explicitly labeled legal-command API fixture.

This does not count as newcomer testing or UI gameplay. Normal API commands
create a real engine damage choice; browser checks its rendered labels/geometry.
No injected state, custom cards, replayed seed, or persisted seat credentials.
"""
import argparse
import collections
import json
import re
import uuid
from pathlib import Path

from playwright.sync_api import sync_playwright

from browser_playtest import UiRun
from common import Api, IsolatedService, action_payload, public_view, write_json


def choice_action(choice):
    options = choice['options']
    kind = choice['kind']
    result = {'kind': 'choose', 'choiceId': choice['id']}
    if kind in ('investigation', 'order'):
        result.update(top=[o['id'] for o in options], bottom=[])
    elif kind == 'region_return':
        result['selected'] = [o['id'] for o in options]
    elif kind == 'mulligan':
        result['selected'] = []
    else:
        count = max(choice.get('min', 0), 1 if options else 0)
        result['selected'] = [o['id'] for o in options[:count]]
    return result


def main(args):
    output = Path(args.output).resolve()
    service = IsolatedService(args.binary, args.cwd, output, args.port, args.static_dir)
    counts = collections.Counter()
    errors, shots, measurements = [], [], []
    service.start()
    try:
        api = Api(service.base_url)
        status, catalog = api.request('GET', '/api/catalog')
        assert status == 200
        status, host = api.request('POST', '/api/rooms', {'name': '布局甲', 'mode': 'teams', 'deckId': catalog['decks'][0]['id']})
        assert status == 200
        sessions = [host]
        for index, name in enumerate(('布局乙', '布局丙', '布局丁'), 1):
            status, session = api.request('POST', '/api/rooms/join', {'inviteCode': host['inviteCode'], 'name': name, 'deckId': catalog['decks'][index]['id']})
            assert status == 200
            sessions.append(session)

        def command(seat, action):
            session = sessions[seat]
            view = api.state(session)
            status, _ = api.request('POST', f'/api/rooms/{session["roomId"]}/commands',
                                    {'commandId': str(uuid.uuid4()), 'expectedVersion': view['version'], 'action': action_payload(action)}, token=session['token'])
            assert status == 200, f'Legal layout fixture command {action["kind"]} returned {status}'
            counts[action['kind']] += 1

        for seat in range(4): command(seat, {'kind': 'ready'})
        command(0, {'kind': 'start'})
        policy = UiRun(None, service.base_url, output, args.max_steps)
        policy.mode = 'teams'
        policy.catalog = {c['id']: c for c in catalog['cards']}
        policy.pages = [None] * 4
        pending = None
        for step in range(args.max_steps):
            views = [api.state(session) for session in sessions]
            chosen = next(((seat, view) for seat, view in enumerate(views) if view.get('pendingChoice')), None)
            if chosen:
                seat, view = chosen
                choice = view['pendingChoice']
                if choice['kind'] == 'damage':
                    pending = (seat, view)
                    break
                command(seat, choice_action(choice))
            else:
                acted = False
                for seat, view in enumerate(views):
                    action = policy.policy(seat, view)
                    if action:
                        command(seat, action)
                        policy.coverage[action['kind']] += 1
                        acted = True
                        break
                assert acted, 'Fixture has no progressing legal action'
        assert pending, 'Bounded fixture did not reach a real damage choice'
        seat, view = pending
        players = {p['id']: p['name'] for p in view['players']}
        expected_owners = [players[o['card']['owner']] for o in view['pendingChoice']['options']]
        credentials = {key: sessions[seat][key] for key in ('roomId', 'inviteCode', 'token', 'seat')}
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
            for width, height in ((390, 844), (1366, 900)):
                context = browser.new_context(viewport={'width': width, 'height': height}, is_mobile=width == 390, has_touch=width == 390)
                context.add_init_script('localStorage.setItem("hegemony.session.v1", ' + json.dumps(json.dumps(credentials)) + ');')
                page = context.new_page()
                page.on('pageerror', lambda error: errors.append({'kind': 'pageerror', 'message': str(error)}))
                page.on('console', lambda message: errors.append({'kind': 'console', 'message': message.text}) if message.type == 'error' else None)
                page.goto(service.base_url, wait_until='domcontentloaded')
                panel = page.locator('.hg-choice')
                panel.locator('.hg-damage-row').first.wait_for(state='visible')
                goto = page.get_by_role('button', name='前往完成选择 ↑', exact=True)
                if goto.count(): goto.click()
                else: panel.scroll_into_view_if_needed()
                targets = panel.locator('[data-choice-target-number]')
                assert targets.count() == len(expected_owners)
                target_labels = []
                for index in range(targets.count()):
                    target = targets.nth(index)
                    assert target.get_attribute('data-choice-target-number') == str(index + 1)
                    assert f'目标 {index + 1}' in target.inner_text()
                    assert f'{expected_owners[index]} 拥有' in target.inner_text()
                    target_labels.append({'number': index + 1, 'ownerLabelVisible': True})
                assert not page.evaluate('document.documentElement.scrollWidth > innerWidth + 1')
                file = f'layout-actual-damage-{width}.png'
                page.screenshot(path=str(output / file), full_page=False)
                shots.append({'file': file, 'privateLocalArtifact': True})
                if width == 390:
                    page.locator('.hg-hand .hg-card').first.click()
                    inspector = page.locator('.hg-inspector-active')
                    inspector.wait_for(state='visible')
                    background = inspector.evaluate('(element) => getComputedStyle(element).backgroundColor')
                    assert background == 'rgb(36, 44, 50)', f'Mobile inspector background is not opaque: {background}'
                    file = 'layout-mobile-inspector-390.png'
                    page.screenshot(path=str(output / file), full_page=False)
                    shots.append({'file': file, 'privateLocalArtifact': True})
                    inspector.get_by_role('button', name=re.compile('放大文字与图标')).click()
                    dialog = page.get_by_role('dialog', name=re.compile('放大阅读'))
                    dialog.wait_for(state='visible')
                    box = dialog.bounding_box()
                    assert box and box['x'] >= 0 and box['x'] + box['width'] <= width
                    file = 'layout-mobile-reader-390.png'
                    page.screenshot(path=str(output / file), full_page=False)
                    shots.append({'file': file, 'privateLocalArtifact': True})
                measurements.append({'viewportWidth': width, 'targetLabels': target_labels,
                                     'documentHorizontalOverflow': False, **({'inspectorBackground': background} if width == 390 else {})})
                context.close()
            browser.close()
        assert not errors
        write_json(output / 'layout-summary.json', {'testType': 'browser-layout-with-legal-api-fixture', 'passed': True,
                   'countsAsUiGameplay': False, 'countsAsNewcomerTest': False, 'stateInjection': False,
                   'fixtureAcceptedCommands': dict(counts), 'measurements': measurements, 'browserErrors': errors,
                   'screenshots': shots, 'publicState': public_view(view)})
        print(json.dumps({'passed': True, 'fixtureCommands': sum(counts.values()), 'output': str(output)}))
    except Exception as error:
        write_json(output / 'layout-summary.json', {'testType': 'browser-layout-with-legal-api-fixture', 'passed': False,
                   'failure': f'{type(error).__name__}: {error}', 'browserErrors': errors, 'screenshots': shots})
        raise
    finally:
        service.stop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default='target/debug/hegemony-server')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-layout')
    parser.add_argument('--port', type=int, default=8095)
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--max-steps', type=int, default=1200)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    main(parser.parse_args())
