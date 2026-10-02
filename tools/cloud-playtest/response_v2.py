#!/usr/bin/env python3
"""Real response UI on legally prepared, explicitly labeled v2 API fixtures.

No seed override, state injection, custom cards or full-game claim. All response
declarations, team passes and pending sacrifices are actual browser controls.
"""
import argparse
import collections
import hashlib
import json
import re
import time
import uuid
from pathlib import Path

from playwright.sync_api import sync_playwright

from api_acceptance import assert_projection, fingerprint
from browser_playtest import OBSERVE_FETCH
from common import Api, IsolatedService, action_payload, public_view, write_json


def evidence_view(view):
    return {**public_view(view), 'stack': view.get('stack', [])}


def character_cards(view):
    return [c for region in view['regions'] for c in region['characters']]


class Fixture:
    def __init__(self, api, output, mode, max_steps):
        self.api, self.output, self.mode, self.max_steps = api, output, mode, max_steps
        self.sessions, self.records, self.probes = [], [], []
        status, self.catalog = api.request('GET', '/api/catalog')
        assert status == 200 and self.catalog['engineVersion'] == 'rust-v0.2.0'
        assert next(d for d in self.catalog['decks'] if d['id'] == 'responders')['cardCount'] == 50
        self.definitions = {c['id']: c for c in self.catalog['cards']}
        status, host = api.request('POST', '/api/rooms', {'name': '响应甲', 'mode': mode, 'deckId': 'responders'})
        assert status == 200
        self.sessions.append(host)
        for index in range(1, 2 if mode == 'duel' else 4):
            status, seat = api.request('POST', '/api/rooms/join', {'inviteCode': host['inviteCode'], 'name': ('响应乙', '响应丙', '响应丁')[index - 1], 'deckId': 'responders'})
            assert status == 200
            self.sessions.append(seat)
        self.room_id = host['roomId']
        for seat in range(len(self.sessions)): self.command(seat, {'kind': 'ready'})
        self.command(0, {'kind': 'start'})

    def views(self):
        return [self.api.state(session) for session in self.sessions]

    def command(self, seat, action):
        session = self.sessions[seat]
        before = self.api.state(session)
        body = {'commandId': str(uuid.uuid4()), 'expectedVersion': before['version'], 'action': action_payload(action)}
        status, after = self.api.request('POST', f'/api/rooms/{self.room_id}/commands', body, token=session['token'])
        assert status == 200, f'Fixture {action["kind"]} returned HTTP {status}: {after.get("message")}'
        record = {'seat': seat, 'beforeVersion': before['version'], 'afterVersion': after['version'], 'httpStatus': status,
                  'action': body['action']}
        self.records.append(record)
        with (self.output / f'{self.mode}-fixture-commands.jsonl').open('a') as stream:
            stream.write(json.dumps(record, ensure_ascii=False) + '\n')
        return after

    def choose(self, seat, view):
        choice = view['pendingChoice']
        action = {'kind': 'choose', 'choiceId': choice['id']}
        if choice['kind'] == 'mulligan':
            # Retain response cards and colored loyalty sources; replace plain
            # filler with the ordinary single mulligan, never reshuffle by admin.
            protected = {'JC042', 'JC091', 'JC049', 'JZ54'}
            action['selected'] = [o['id'] for o in choice['options'] if o.get('card', {}).get('cardId') not in protected
                                  and o.get('card', {}).get('color', '') not in ('红', '黑')]
        elif choice['kind'] in ('investigation', 'order'):
            action.update(top=[o['id'] for o in choice['options']], bottom=[])
        elif choice['kind'] == 'discard':
            seen = collections.Counter()
            keep = {}
            for option in choice['options']:
                definition = option.get('card', {}).get('cardId')
                seen[definition] += 1
                keep[option['id']] = (100 if definition in ('JC042', 'JC091') else 50 if definition in ('JC049', 'JZ54') else 0) if seen[definition] == 1 else 10
            def keep_score(option):
                return keep[option['id']]
            action['selected'] = [o['id'] for o in sorted(choice['options'], key=keep_score)[:choice.get('min', 0)]]
        else:
            count = choice.get('min', 0)
            action['selected'] = [o['id'] for o in choice['options'][:count]]
        return self.command(seat, action)

    def asset(self, view):
        owned = [c for c in view['assets'] if c['controller'] == view['you']]
        if len(owned) >= 3: return None
        colors = {c.get('color', self.definitions.get(c.get('cardId'), {}).get('color')) for c in owned}
        missing = {'红', '黑'} - colors
        hand = {c['instanceId']: c for c in view['hand']}
        counts = collections.Counter(c.get('cardId') for c in view['hand'])
        choices = [a for a in view['legalActions'] if a['kind'] == 'asset'
                   and not (hand[a['cardId']].get('cardId') in ('JC042', 'JC091')
                            and counts[hand[a['cardId']].get('cardId')] <= 1)]
        required = [a for a in choices if hand[a['cardId']].get('color', self.definitions[hand[a['cardId']]['cardId']]['color']) in missing]
        if required: return required[0]
        # Reserve a resource slot for each still-missing loyalty color.
        if len(owned) < 3 - len(missing) and choices: return choices[0]
        return None

    def prepare(self, optional_sacrifice=False, limit=None):
        limit = limit or self.max_steps
        for step in range(limit):
            views = self.views()
            pending = next(((seat, view) for seat, view in enumerate(views) if view.get('pendingChoice')), None)
            if pending:
                self.choose(*pending)
                continue
            if views[0]['status'] != 'playing': raise RuntimeError('Fixture finished before the target response window')
            if not optional_sacrifice and views[0]['turn'] > 15:
                diagnostic = {'turn': views[0]['turn'], 'reason': 'Bounded preparation did not reach the response by turn 15',
                              'seats': [{'seat': seat, 'assetColors': [c.get('color') for c in view['assets'] if c['controller'] == view['you']],
                                         'primaryCardPresent': any(c.get('cardId') == 'JC042' for c in view['hand']),
                                         'attackCardPresent': any(c.get('cardId') == 'JC091' for c in view['hand']),
                                         'publicBoardCount': len(character_cards(view))} for seat, view in enumerate(views)]}
                write_json(self.output / f'{self.mode}-prepare-diagnostic.json', diagnostic)
                raise RuntimeError('Stopped response preparation at turn 15; inspect the bounded diagnostic')
            # Stop only at a genuine service-provided legal declaration.
            for attacker, view in enumerate(views):
                for action in view['legalActions']:
                    source = next((c for c in view['hand'] if c['instanceId'] == action.get('cardId')), None)
                    if not source or action['kind'] != 'play': continue
                    if optional_sacrifice:
                        if source.get('cardId') != 'JZ54': continue
                        defender = next((seat for seat, own in enumerate(views) if own['you'] == action.get('targetId')), None)
                        if defender is None or views[defender]['players'][defender]['team'] == view['players'][attacker]['team']: continue
                        victims = [c for c in character_cards(views[defender]) if c['controller'] == views[defender]['you']
                                   and c.get('kind') == 'character' and not c.get('faceDown')]
                        if victims: return attacker, defender, action, victims[0]['instanceId']
                    else:
                        target = next((c for c in character_cards(view) if c['instanceId'] == action.get('targetId') and c.get('cardId') == 'JC042'), None)
                        if source.get('cardId') == 'JC091' and target and target['controller'] != view['you']:
                            defender = next(seat for seat, own in enumerate(views) if own['you'] == target['controller'])
                            if views[defender]['players'][defender]['team'] != view['players'][attacker]['team']:
                                return attacker, defender, action, target['instanceId']
            acted = False
            for seat, view in enumerate(views):
                action = self.asset(view)
                if action:
                    self.command(seat, action); acted = True; break
            if not acted and not views[0]['stack']:
                has_disciple = any(c.get('cardId') == 'JC042' for c in character_cards(views[0]))
                for seat, view in enumerate(views):
                    hand = {c['instanceId']: c for c in view['hand']}
                    wanted = 'JC125' if optional_sacrifice else 'JC042'
                    if not optional_sacrifice and has_disciple: break
                    if optional_sacrifice and any(c['controller'] == view['you'] and c.get('kind') == 'character' for c in character_cards(view)): continue
                    actions = [a for a in view['legalActions'] if a['kind'] == 'deploy' and hand[a['cardId']].get('cardId') == wanted]
                    if not actions: continue
                    if not optional_sacrifice:
                        opponents = [own for own in views if own['players'][int(own['you'][1:])]['team'] != view['players'][seat]['team']]
                        ready_murder = any(any(c.get('cardId') == 'JC091' for c in own['hand'])
                                           and sum(c['controller'] == own['you'] and not c['exhausted'] for c in own['assets']) >= 3
                                           and any(c['controller'] == own['you'] and c.get('color') == '黑' for c in own['assets']) for own in opponents)
                        if not ready_murder: continue
                    action = next((a for a in actions if a.get('region') == (0 if self.mode == 'duel' else 2)), actions[0])
                    self.command(seat, action); acted = True; break
            if not acted:
                for seat, view in enumerate(views):
                    action = next((a for a in view['legalActions'] if a['kind'] == 'pass'), None)
                    if action: self.command(seat, action); acted = True; break
            assert acted, 'Fixture cannot progress using normal legal commands'
            if step % 150 == 0:
                print(json.dumps({'mode': self.mode, 'fixtureStep': step, 'turn': views[0]['turn'], 'apiCommands': len(self.records), 'optionalSacrifice': optional_sacrifice}), flush=True)
        raise RuntimeError(f'No requested legal response in bounded {limit}-command fixture')


class BrowserResponse:
    def __init__(self, browser, fixture, service, output):
        self.fixture, self.service, self.output = fixture, service, output
        self.pages, self.contexts, self.errors, self.expected_errors, self.commands, self.shots, self.checks = [], [], [], [], [], [], []
        self.restart_active = False
        self.browser = browser

    def open(self, attacker, defender):
        for seat, session in enumerate(self.fixture.sessions):
            width = 390 if seat == defender else 1366 if seat == attacker else (390 if seat % 2 else 1366)
            context = self.browser.new_context(viewport={'width': width, 'height': 844 if width == 390 else 900}, is_mobile=width == 390, has_touch=width == 390, locale='zh-CN')
            credentials = {key: session[key] for key in ('roomId', 'inviteCode', 'token', 'seat')}
            context.add_init_script('localStorage.setItem("hegemony.session.v1", ' + json.dumps(json.dumps(credentials)) + ');')
            context.add_init_script(OBSERVE_FETCH)
            page = context.new_page()
            page.on('pageerror', lambda error, seat=seat: self.errors.append({'seat': seat, 'kind': 'pageerror', 'message': str(error)}))
            def console(message, seat=seat):
                if message.type != 'error': return
                entry = {'seat': seat, 'kind': 'console', 'message': message.text}
                if self.restart_active and any(code in message.text for code in ('ERR_CONNECTION_REFUSED', 'ERR_EMPTY_RESPONSE', 'ERR_NETWORK_CHANGED', 'ERR_INCOMPLETE_CHUNKED_ENCODING')):
                    self.expected_errors.append({**entry, 'cause': 'owned-service-restart'})
                else: self.errors.append(entry)
            page.on('console', console)
            self.contexts.append(context); self.pages.append(page)
            page.goto(self.service.base_url, wait_until='domcontentloaded')
            page.wait_for_function('window.__cloudView?.status === "playing"', timeout=30000)
        self.sync(self.fixture.views()[0]['version'])

    def views(self): return [page.evaluate('window.__cloudView') for page in self.pages]

    def sync(self, version):
        for seat, page in enumerate(self.pages):
            page.wait_for_function('(v) => window.__cloudView?.version >= v', arg=version, timeout=30000)
            view = page.evaluate('window.__cloudView')
            assert view['you'] == f'p{seat}'
            assert_projection(view)
        self.pages[0].wait_for_timeout(55)

    def screenshot(self, name, seat):
        file = f'{self.fixture.mode}-{name}-seat{seat + 1}.png'
        self.pages[seat].screenshot(path=str(self.output / file), full_page=False)
        self.shots.append({'file': file, 'seat': seat, 'privateLocalArtifact': True})

    def receive(self, seat, response, before, automatic=False):
        after = response.json()
        body = json.loads(response.request.post_data)
        assert response.ok, f'UI command rejected HTTP {response.status}: {after.get("message")}'
        record = {'seat': seat, 'automatic': automatic, 'httpStatus': response.status, 'action': body['action'],
                  'before': evidence_view(before), 'after': evidence_view(after)}
        self.commands.append(record)
        with (self.output / f'{self.fixture.mode}-ui-commands.jsonl').open('a') as stream:
            stream.write(json.dumps(record, ensure_ascii=False) + '\n')
        self.sync(after['version'])
        return body, after

    def submit(self, seat, button):
        before = self.views()[seat]
        with self.pages[seat].expect_response(lambda r: r.request.method == 'POST' and '/commands' in r.url, timeout=20000) as pending:
            button.click()
        return self.receive(seat, pending.value, before)

    def action(self, seat, action):
        page = self.pages[seat]
        identity = action.get('cardId') or action.get('targetId')
        if identity:
            page.locator('[data-card-instance=' + json.dumps(identity) + ']').first.click()
        button = page.locator('[data-action-id=' + json.dumps(action['id']) + ']')
        button.wait_for(state='visible')
        return self.submit(seat, button)

    def response_geometry(self, seat, name):
        page = self.pages[seat]
        panel = page.locator('.hg-hand-dock .hg-response-window')
        panel.wait_for(state='visible')
        box = panel.bounding_box(); viewport = page.viewport_size
        assert box and 0 <= box['x'] and box['x'] + box['width'] <= viewport['width'] + 1
        assert 0 <= box['y'] and box['y'] + box['height'] <= viewport['height'] + 1, 'Response window outside screen'
        assert not page.evaluate('document.documentElement.scrollWidth > innerWidth + 1'), 'Document horizontal overflow'
        self.checks.append({'name': name, 'viewportWidth': viewport['width'], 'responseWindowInsideViewport': True,
                            'responseState': panel.get_attribute('data-response-state'), 'box': {k: round(v, 2) for k, v in box.items()}})

    def restart(self, label, command=None):
        before = self.views()
        self.pages[0].reload(wait_until='domcontentloaded')
        self.sync(before[0]['version'])
        assert self.views() == before, 'Refresh changed accepted response state'
        self.restart_active = True
        self.service.stop()
        self.pages[0].get_by_text('连接中断 · 自动重连', exact=True).wait_for(timeout=25000)
        self.screenshot(label + '-disconnected', 0)
        self.service.start()
        for page in self.pages: page.get_by_text('牌桌已连接', exact=True).wait_for(timeout=35000)
        self.sync(before[0]['version'])
        restored = self.views()
        assert restored == before, 'Restart changed full per-seat accepted views'
        self.restart_active = False
        if command:
            seat, body, ack = command
            database = self.output / 'acceptance.sqlite3'
            before_db = fingerprint(database, self.fixture.room_id)
            session = self.fixture.sessions[seat]
            status, replay = self.fixture.api.request('POST', f'/api/rooms/{self.fixture.room_id}/commands', body, token=session['token'])
            assert status == 200 and replay == ack
            assert fingerprint(database, self.fixture.room_id) == before_db
            assert self.fixture.views() == restored
            self.fixture.probes.append({'kind': 'restart-command-dedupe', 'label': label, 'passed': True, 'persistedStateUnchanged': True})
        self.checks.append({'name': label, 'refreshExactViews': True, 'restartExactViews': True, 'independentContexts': len(self.pages)})

    def all_pass_once(self, count, name):
        seats = []
        while len(self.views()[0]['stack']) == count and not self.views()[0].get('waitingChoice'):
            views = self.views()
            chosen = next(((seat, next((a for a in view['legalActions'] if a['kind'] == 'pass'), None))
                           for seat, view in enumerate(views) if any(a['kind'] == 'pass' for a in view['legalActions'])), None)
            assert chosen and len(seats) < len(self.pages), 'Stack did not resolve after every living seat passed'
            self.action(*chosen)
            seats.append(chosen[0])
            if len(seats) < len(self.pages): assert len(self.views()[0]['stack']) == count
        assert len(seats) == len(self.pages) and len(set(seats)) == len(self.pages)
        assert len(self.views()[0]['stack']) == count - 1 or self.views()[0].get('waitingChoice'), 'One full pass round did not resolve precisely its top object'
        self.checks.append({'name': name, 'passingSeats': seats, 'stackBefore': count, 'stackAfter': len(self.views()[0]['stack']), 'exactlyOneTopResolved': True})

    def lower_auto_pass(self, attacker):
        seats = []
        automatic = False
        while self.views()[0]['stack']:
            views = self.views()
            own = views[attacker]
            if not automatic and len(own['legalActions']) == 1 and own['legalActions'][0]['kind'] == 'pass':
                switch = self.pages[attacker].get_by_role('switch', name='无可用行动时自动让过')
                with self.pages[attacker].expect_response(lambda r: r.request.method == 'POST' and '/commands' in r.url, timeout=5000) as pending:
                    switch.check()
                body, _ = self.receive(attacker, pending.value, own, automatic=True)
                assert body['action']['kind'] == 'pass'
                switch.uncheck()
                automatic = True; seats.append(attacker)
            else:
                chosen = next(((seat, next((a for a in view['legalActions'] if a['kind'] == 'pass'), None))
                               for seat, view in enumerate(views) if any(a['kind'] == 'pass' for a in view['legalActions'])), None)
                assert chosen and len(seats) < len(self.pages)
                self.action(*chosen); seats.append(chosen[0])
            if len(seats) < len(self.pages): assert len(self.views()[0]['stack']) == 1
        assert automatic and len(seats) == len(self.pages) and len(set(seats)) == len(self.pages)
        self.checks.append({'name': 'lower-sole-pass-auto-advances', 'passingSeats': seats, 'automaticSeat': attacker, 'exactlyOneTopResolved': True})

    def run_chain(self, attacker, defender, murder, target):
        self.restart('before-murder')
        before = self.views()[attacker]
        unspent = sum(c['controller'] == before['you'] and not c['exhausted'] for c in before['assets'])
        assert unspent == 3
        self.action(attacker, murder)
        declared = self.views()
        assert len(declared[0]['stack']) == 1 and declared[0]['stack'][0]['cardId'] == 'JC091'
        assert declared[defender]['stack'][0]['targetSummaries'][0]['valid']
        self.response_geometry(defender, 'opponent-sees-murder-before-passes')
        self.screenshot('murder-before-passes-phone', defender)
        # Murder's team still owns priority until all its living members pass.
        murderer_team = before['players'][attacker]['team']
        first_passes = []
        while self.views()[0]['priorityTeam'] == murderer_team:
            chosen = next((seat for seat, view in enumerate(self.views()) if view['players'][seat]['team'] == murderer_team
                           and any(a['kind'] == 'pass' for a in view['legalActions'])), None)
            assert chosen is not None
            action = next(a for a in self.views()[chosen]['legalActions'] if a['kind'] == 'pass')
            self.action(chosen, action); first_passes.append(chosen)
            assert len(self.views()[0]['stack']) == 1
        assert len(first_passes) == len(self.pages) // 2
        fast = next(a for a in self.views()[defender]['legalActions'] if a['kind'] == 'activate' and a.get('cardId') == target and a.get('abilityId') == 'reduce-next')
        switch = self.pages[defender].get_by_role('switch', name='无可用行动时自动让过')
        stable = self.views()[0]['version']
        switch.check()
        self.pages[defender].wait_for_timeout(1250)
        assert all(view['version'] == stable for view in self.views()), 'Auto-pass swallowed a real fast response'
        panel = self.pages[defender].locator('.hg-response-window')
        assert panel.get_attribute('data-response-state') == 'respond'
        panel.get_by_role('button', name='查看响应牌', exact=True).click()
        self.pages[defender].locator('.hg-inspector-active').wait_for(state='visible')
        # Pick the intended public source through its regular board card; the CTA
        # may initially choose another genuinely available fast card.
        self.pages[defender].locator('[data-card-instance=' + json.dumps(target) + ']').first.click()
        button = self.pages[defender].locator('[data-action-id=' + json.dumps(fast['id']) + ']')
        button.wait_for(state='visible')
        box = button.bounding_box(); response_box = panel.bounding_box()
        assert box and response_box and box['y'] >= 0 and box['y'] + box['height'] <= response_box['y'] + 1, 'Fast response button overlaps fixed response area'
        self.response_geometry(defender, 'fast-response-and-inspector-reachable-phone')
        self.screenshot('fast-response-drawer-phone', defender)
        self.checks.append({'name': 'auto-pass-retains-real-fast-action', 'abilityId': fast['abilityId'], 'unchangedVersion': stable, 'fastButtonAboveResponseWindow': True})
        switch.uncheck()
        body, ack = self.submit(defender, button)
        sacrificed = self.views()
        assert len(sacrificed[0]['stack']) == 2
        assert all(c['instanceId'] != target for c in character_cards(sacrificed[0]))
        assert any(c.get('cardId') == 'JC042' and c['owner'] == f'p{defender}' for c in sacrificed[0]['graveyard'])
        old = sacrificed[0]['stack'][0]['targetSummaries'][0]
        assert old['instanceId'] == target and not old['valid'] and old['status'] == 'missing'
        self.restart('mid-response', (defender, body, ack))
        self.all_pass_once(2, 'all-seats-pass-resolves-only-independent-top')
        for seat in (attacker, defender):
            missing = self.pages[seat].locator('.hg-response-window [data-target-instance=' + json.dumps(target) + ']')
            assert missing.get_attribute('data-target-valid') == 'false'
            assert missing.get_attribute('data-target-status') == 'missing'
            assert '原目标已离场' in missing.inner_text()
            self.response_geometry(seat, 'original-target-missing-visible')
            self.screenshot('original-target-missing', seat)
        self.lower_auto_pass(attacker)
        after = self.views()[attacker]
        assert not after['stack']
        assert sum(c['controller'] == after['you'] and not c['exhausted'] for c in after['assets']) == 0
        assert any(c.get('cardId') == 'JC091' and c['owner'] == after['you'] for c in after['graveyard'])
        assert any('谋杀' in entry['text'] and '整个卡牌或能力效果取消' in entry['text'] and '费用不退' in entry['text'] for entry in after['log'])
        self.checks.append({'name': 'murder-invalid-no-refund-and-buried', 'originalTarget': target, 'assetsSpent': 3, 'assetsUnspentAfter': 0})

    def pending_sacrifice(self):
        # An optional small continuation, still using ordinary commands. A large
        # second preparation is intentionally left to kernel/SQLite integration.
        if not any(c.get('cardId') == 'JZ54' for view in self.fixture.views() for c in view['hand']):
            return {'attempted': False, 'reason': 'No normally dealt JZ54 in the current hands'}
        try:
            attacker, defender, action, _ = self.fixture.prepare(optional_sacrifice=True, limit=420)
        except RuntimeError as error:
            if 'bounded 420-command fixture' not in str(error): raise
            return {'attempted': True, 'browserVerified': False, 'reason': 'No legal JZ54 victim window within 420 normal preparation commands', 'requiresKernelSQLiteEvidence': True}
        self.sync(self.fixture.views()[0]['version'])
        self.action(attacker, action)
        self.all_pass_once(1, 'jz54-all-seats-pass-enters-effect-choice')
        views = self.views(); choice = views[defender]['pendingChoice']
        assert choice and choice['kind'] == 'target' and choice['min'] == choice['max'] == 1
        assert all(not view.get('pendingChoice') for seat, view in enumerate(views) if seat != defender)
        assert views[attacker]['stack'][-1]['targetSummaries'][0]['status'] == 'guardAccepted'
        self.screenshot('jz54-private-sacrifice-choice', defender)
        self.restart('mid-effect-choice')
        panel = self.pages[defender].locator('.hg-choice')
        go = self.pages[defender].get_by_role('button', name='前往完成选择 ↑', exact=True)
        if go.count(): go.click()
        panel.locator('[data-choice-option]').first.click()
        body, ack = self.submit(defender, panel.get_by_role('button', name='确认选择', exact=True))
        before_db = fingerprint(self.output / 'acceptance.sqlite3', self.fixture.room_id)
        restored = self.fixture.views()
        session = self.fixture.sessions[defender]
        status, replay = self.fixture.api.request('POST', f'/api/rooms/{self.fixture.room_id}/commands', body, token=session['token'])
        assert status == 200 and replay == ack
        assert fingerprint(self.output / 'acceptance.sqlite3', self.fixture.room_id) == before_db and self.fixture.views() == restored
        assert not self.views()[0]['stack'] and not any(v.get('waitingChoice') for v in self.views())
        self.fixture.probes.append({'kind': 'effect-choice-dedupe', 'passed': True, 'persistedStateUnchanged': True})
        return {'attempted': True, 'passed': True, 'pendingChoiceRestored': True, 'guardAcceptedVisible': True, 'uiChoiceCompleted': True, 'choiceDedupeNoReplay': True}

    def close(self):
        for context in self.contexts: context.close()


def main(args):
    output = Path(args.output).resolve(); output.mkdir(parents=True, exist_ok=True)
    binary = Path(args.binary).resolve()
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    service = IsolatedService(str(binary), args.cwd, output, args.port, args.static_dir)
    cases = []
    service.start()
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
            for mode in args.modes:
                fixture = Fixture(Api(service.base_url), output, mode, args.max_steps)
                actor, defender, action, target = fixture.prepare()
                print(json.dumps({'mode': mode, 'preparedTurn': fixture.views()[0]['turn'], 'fixtureCommands': len(fixture.records), 'attacker': actor, 'defender': defender}), flush=True)
                run = BrowserResponse(browser, fixture, service, output)
                try:
                    run.open(actor, defender)
                    run.run_chain(actor, defender, action, target)
                    optional = run.pending_sacrifice() if mode == 'duel' and not args.skip_optional else {'attempted': False}
                    assert not run.errors
                    result = {'mode': mode, 'roomId': fixture.room_id, 'passed': True, 'testType': 'response-ui-with-legal-api-preparation',
                              'countsAsCompleteUiGame': False, 'stateInjection': False, 'seedOverride': False,
                              'independentBrowserContexts': len(run.pages), 'fixtureAcceptedCommands': len(fixture.records),
                              'uiAcceptedCommands': len(run.commands), 'apiProbeRequests': len(fixture.probes), 'apiProbes': fixture.probes,
                              'checks': run.checks, 'optionalJZ54': optional, 'browserErrors': run.errors,
                              'expectedRestartNetworkErrors': run.expected_errors, 'screenshots': run.shots,
                              'final': evidence_view(run.views()[0])}
                    cases.append(result)
                    write_json(output / f'{mode}-response-summary.json', result)
                except Exception as error:
                    for seat in range(len(run.pages)): run.screenshot('failure', seat)
                    write_json(output / f'{mode}-response-summary.json', {'passed': False, 'failure': f'{type(error).__name__}: {error}',
                               'roomId': fixture.room_id, 'fixtureAcceptedCommands': len(fixture.records), 'uiAcceptedCommands': len(run.commands),
                               'checks': run.checks, 'browserErrors': run.errors, 'expectedRestartNetworkErrors': run.expected_errors, 'screenshots': run.shots})
                    raise
                finally:
                    run.close()
            browser.close()
        write_json(output / 'response-summary.json', {'passed': True, 'engineVersion': 'rust-v0.2.0', 'binary': str(binary), 'binarySha256': digest,
                   'port': args.port, 'cases': cases, 'ownedServiceStopped': True})
        print(json.dumps({'passed': True, 'binarySha256': digest, 'cases': [{'mode': c['mode'], 'fixture': c['fixtureAcceptedCommands'], 'ui': c['uiAcceptedCommands']} for c in cases]}), flush=True)
    except Exception as error:
        write_json(output / 'response-summary.json', {'passed': False, 'engineVersion': 'rust-v0.2.0', 'binary': str(binary),
                   'binarySha256': digest, 'port': args.port, 'cases': cases, 'failure': f'{type(error).__name__}: {error}',
                   'ownedServiceStopped': True})
        raise
    finally:
        service.stop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default='/tmp/hegemony-response-v2-2026-10-02/bin/hegemony-server')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--output', default='/tmp/hegemony-response-v2-2026-10-02')
    parser.add_argument('--port', type=int, default=8096)
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--max-steps', type=int, default=3200)
    parser.add_argument('--modes', nargs='+', choices=('duel', 'teams'), default=['duel', 'teams'])
    parser.add_argument('--skip-optional', action='store_true')
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    main(parser.parse_args())
