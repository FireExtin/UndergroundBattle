#!/usr/bin/env python3
"""Isolated HTTP contract/security/persistence probes; these are not UI play."""
from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import time
import urllib.request
import uuid
from pathlib import Path

from common import Api, IsolatedService, action_payload, public_view, write_json


class Events:
    def __init__(self, api, session):
        request = urllib.request.Request(api.base + f'/api/rooms/{session["roomId"]}/events', headers={'Authorization': 'Bearer ' + session['token'], 'Accept': 'text/event-stream'})
        self.response = urllib.request.urlopen(request, timeout=12)
        assert self.response.status == 200
        assert self.response.headers.get('Content-Type', '').startswith('text/event-stream')

    def next(self, version=None):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            lines = []
            while True:
                line = self.response.readline().decode().rstrip('\r\n')
                if not line: break
                if line.startswith('data:'): lines.append(line[5:].lstrip())
            if lines:
                view = json.loads('\n'.join(lines))
                if version is None or view['version'] >= version:
                    return view
        raise RuntimeError('SSE did not emit the expected accepted version')

    def close(self):
        self.response.close()


def fingerprint(database, room_id):
    # Hash authoritative persisted bytes and journal counts without exposing state.
    with sqlite3.connect(database) as connection:
        row = connection.execute('SELECT state FROM rooms WHERE id=?', (room_id,)).fetchone()
        return {'stateSha256': hashlib.sha256(row[0].encode()).hexdigest(),
                'commands': connection.execute('SELECT count(*) FROM commands WHERE room_id=?', (room_id,)).fetchone()[0],
                'journal': connection.execute('SELECT count(*) FROM journal WHERE room_id=?', (room_id,)).fetchone()[0]}


def assert_projection(view):
    assert all(card['owner'] == view['you'] for card in view['hand']), 'Other seat hand appeared in viewer hand'
    # Both opponents and teammates must receive an identity-free hidden projection.
    sensitive = {'cardId', 'text', 'cost', 'icons', 'defense', 'damage', 'wounds', 'shield', 'color', 'magic'}
    for region in view['regions']:
        for card in region['characters']:
            if card.get('faceDown') and card['controller'] != view['you']:
                assert card['name'] == '暗藏者'
                assert not sensitive.intersection(card), 'Hidden printed attributes leaked in SSE'
    def credential_key(value):
        if isinstance(value, dict):
            return any(key.lower() in {'token', 'authorization', 'bearer'} or credential_key(item) for key, item in value.items())
        if isinstance(value, list): return any(credential_key(item) for item in value)
        return False
    assert not credential_key(view), 'Credential field appeared in a game view'


def run(args):
    output = Path(args.output).resolve()
    service = IsolatedService(args.binary, args.cwd, output, args.port, args.static_dir)
    records, streams = [], []
    def record(name, **evidence):
        records.append({'name': name, 'passed': True, **evidence})
        write_json(output / 'api-summary.json', {'testType': 'isolated-http-acceptance', 'passed': True, 'checks': records})

    service.start()
    try:
        api = Api(service.base_url)
        status, catalog = api.request('GET', '/api/catalog')
        assert status == 200 and len(catalog['decks']) == 4
        decks = catalog['decks']
        status, host = api.request('POST', '/api/rooms', {'name': 'Probe A', 'mode': 'teams', 'deckId': decks[0]['id']})
        assert status == 200
        sessions = [host]
        for index in range(1, 4):
            status, session = api.request('POST', '/api/rooms/join', {'inviteCode': host['inviteCode'], 'name': 'Probe ' + chr(65 + index), 'deckId': decks[index]['id']})
            assert status == 200
            sessions.append(session)
        path = f'/api/rooms/{host["roomId"]}'
        database = output / 'acceptance.sqlite3'
        before = fingerprint(database, host['roomId'])
        for endpoint in ('/state', '/events'):
            status, _ = api.request('GET', path + endpoint)
            assert status == 401
            status, _ = api.request('GET', path + endpoint, token='invalid-seat-token')
            assert status == 401
        status, _ = api.request('POST', path + '/commands', {'commandId': 'unauthorized', 'expectedVersion': 3, 'action': {'kind': 'ready'}})
        assert status == 401 and fingerprint(database, host['roomId']) == before
        status, other = api.request('POST', '/api/rooms', {'name': 'Foreign room', 'mode': 'duel', 'deckId': decks[0]['id']})
        assert status == 200
        status, _ = api.request('GET', path + '/state', token=other['token'])
        assert status == 401
        record('authentication-room-bound-and-noop', rejectedEndpoints=5, crossRoomRejected=True, persistedStateUnchanged=True)

        def command(seat, action, command_id=None, expected=None, extra=None):
            session = sessions[seat]
            view = api.state(session)
            body = {'commandId': command_id or str(uuid.uuid4()), 'expectedVersion': view['version'] if expected is None else expected, 'action': action_payload(action)}
            if extra: body.update(extra)
            status, response = api.request('POST', path + '/commands', body, token=session['token'])
            return status, response, body

        # Same command ID must replay its saved ACK, including after later changes.
        status, ready, body = command(0, {'kind': 'ready'}, command_id='acceptance-dedup')
        assert status == 200
        committed = fingerprint(database, host['roomId'])
        status, duplicate = api.request('POST', path + '/commands', body, token=host['token'])
        assert status == 200 and duplicate == ready and fingerprint(database, host['roomId']) == committed
        status, _, _ = command(1, {'kind': 'ready'})
        assert status == 200
        changed = fingerprint(database, host['roomId'])
        status, duplicate = api.request('POST', path + '/commands', body, token=host['token'])
        assert status == 200 and duplicate == ready and fingerprint(database, host['roomId']) == changed
        status, _, _ = command(1, {'kind': 'ready'}, command_id='acceptance-dedup')
        assert status == 400 and fingerprint(database, host['roomId']) == changed
        record('duplicate-id-saved-response-no-replay', persistedStateUnchanged=True, crossSeatReuseRejected=True)

        for seat in (2, 3):
            status, _, _ = command(seat, {'kind': 'ready'})
            assert status == 200
        stable = fingerprint(database, host['roomId'])
        stale = api.state(host)['version'] - 1
        status, conflict, _ = command(0, {'kind': 'ready'}, expected=stale)
        assert status == 409 and conflict['view']['version'] == stale + 1
        assert fingerprint(database, host['roomId']) == stable
        record('stale-version-409-noop', persistedStateUnchanged=True, viewerSpecificCurrentView=True)
        status, _, _ = command(0, {'kind': 'start'})
        assert status == 200
        view = api.state(host)
        pending = view['pendingChoice']
        assert pending and pending['kind'] == 'mulligan'

        # Supplying another actor cannot override authority derived from Bearer.
        stable = fingerprint(database, host['roomId'])
        spoof = {'kind': 'choose', 'choiceId': pending['id'], 'selected': []}
        status, _, _ = command(1, spoof, extra={'actor': 'p0', 'seat': 0, 'playerId': 'p0'})
        assert status == 400 and fingerprint(database, host['roomId']) == stable
        record('spoofed-actor-cannot-submit-owner-choice', persistedStateUnchanged=True)
        invalid = {'kind': 'choose', 'choiceId': pending['id'], 'selected': ['nonexistent-private-option']}
        status, _, _ = command(0, invalid)
        assert status == 400 and fingerprint(database, host['roomId']) == stable
        record('invalid-choice-strict-noop', persistedStateUnchanged=True)

        for seat, session in enumerate(sessions):
            stream = Events(api, session); streams.append(stream)
            event = stream.next(view['version'])
            assert_projection(event)
            assert event['you'] == f'p{seat}'
            if seat:
                assert event['pendingChoice'] is None and event['waitingChoice']['playerId'] == 'p0'
            else:
                assert event['pendingChoice'] == pending
        record('sse-private-hand-and-private-pending-choice', independentViews=4, teammateAndOpponentChecked=True)

        saved_views = [api.state(session) for session in sessions]
        for stream in streams: stream.close()
        streams.clear()
        service.restart()
        assert fingerprint(database, host['roomId']) == stable
        for seat, session in enumerate(sessions):
            restored = api.state(session)
            assert restored == saved_views[seat], 'Restart altered accepted state/identity/pending choice'
            stream = Events(api, session); streams.append(stream)
            assert stream.next(restored['version']) == restored
        # Command ACK dedup also remains intact after a process restart.
        status, duplicate = api.request('POST', path + '/commands', body, token=host['token'])
        assert status == 200 and duplicate == ready and fingerprint(database, host['roomId']) == stable
        record('service-restart-restores-choice-identities-and-dedup', independentViews=4, sameFullAcceptedViews=True, persistedStateUnchanged=True)

        # Advance only through enumerated legal actions to observe a hidden board
        # projection. This is an API acceptance fixture, not browser-play evidence.
        concealed = None
        for step in range(120):
            views = [api.state(session) for session in sessions]
            current = max(views, key=lambda v: v['version'])
            choices = [(seat, v) for seat, v in enumerate(views) if v.get('pendingChoice')]
            if choices:
                seat, v = choices[0]
                choice = v['pendingChoice']
                assert choice['kind'] == 'mulligan', 'Unexpected choice before initial development'
                action = {'kind': 'choose', 'choiceId': choice['id'], 'selected': []}
            else:
                all_legal = [(seat, action) for seat, v in enumerate(views) for action in v['legalActions']]
                chosen = next(((seat, a) for seat, a in all_legal if a['kind'] == 'conceal'), None)
                chosen = chosen or next(((seat, a) for seat, a in all_legal if a['kind'] == 'asset'), None)
                chosen = chosen or next(((seat, a) for seat, a in all_legal if a['kind'] == 'pass'), None)
                assert chosen, 'No initial-development legal action'
                seat, action = chosen
            status, accepted, _ = command(seat, action)
            assert status == 200
            for viewer, stream in enumerate(streams):
                event = stream.next(accepted['version'])
                assert event['you'] == f'p{viewer}'
                assert_projection(event)
                if action['kind'] == 'conceal':
                    hidden = [c for r in event['regions'] for c in r['characters'] if c.get('faceDown')]
                    assert hidden
                    if viewer != seat:
                        assert all('cardId' not in c for c in hidden)
            if action['kind'] == 'conceal':
                concealed = seat
                record('sse-live-broadcast-hidden-teammate-and-enemy', independentViews=4, controllerSeat=seat, allAcceptedVersionsObserved=True)
                break
        assert concealed is not None, 'Did not reach a legal conceal action'
        result = {'testType': 'isolated-http-acceptance', 'passed': True, 'checks': records, 'final': public_view(api.state(host))}
        write_json(output / 'api-summary.json', result)
        print(json.dumps({'passed': True, 'checks': len(records), 'output': str(output)}), flush=True)
    except Exception as error:
        write_json(output / 'api-summary.json', {'testType': 'isolated-http-acceptance', 'passed': False, 'checks': records, 'failure': f'{type(error).__name__}: {error}'})
        raise
    finally:
        for stream in streams: stream.close()
        service.stop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default='target/debug/hegemony-server')
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--output', default='/tmp/hegemony-cloud-acceptance')
    parser.add_argument('--port', type=int, default=8091)
    parser.add_argument('--static-dir', default='web/dist')
    run(parser.parse_args())
