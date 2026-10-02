#!/usr/bin/env python3
"""Bounded acceptance of the actual private Worker and remote D1 protocol.

No seed override, storage injection or complete-game claim. Credentials remain
in memory; a separate native Sites database read verifies persisted row counts.
"""
import argparse
import concurrent.futures
import getpass
import json
import urllib.error
import urllib.request
import uuid
from pathlib import Path

from common import write_json


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, *arguments):
        raise RuntimeError('Private Site unexpectedly redirected; credential was not forwarded')


class SitesApi:
    def __init__(self, base, bypass):
        self.base, self.bypass = base.rstrip('/'), bypass

    def request(self, method, path, body=None, token=None):
        headers = {'Content-Type': 'application/json', 'OAI-Sites-Authorization': 'Bearer ' + self.bypass}
        if token:
            headers['Authorization'] = 'Bearer ' + token
        request = urllib.request.Request(self.base + path, data=None if body is None else json.dumps(body).encode(), method=method, headers=headers)
        try:
            response = urllib.request.build_opener(NoRedirect).open(request, timeout=20)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read()
            try:
                value = json.loads(raw) if raw else None
            except ValueError:
                value = {'error': 'non_json_response'}
            return response.status, value

    def state(self, session):
        status, view = self.request('GET', f'/api/rooms/{session["roomId"]}/state', token=session['token'])
        assert status == 200, f'View returned {status}'
        return view


def main(args, bypass):
    api = SitesApi(args.base_url, bypass)
    output = Path(args.output)
    evidence = {'testType': 'actual-deployed-worker-remote-d1-protocol', 'siteUrl': args.base_url,
                'sourceCommit': args.source_commit, 'seedOverride': False, 'intermediateStateInjection': False,
                'countsAsCompleteUiGame': False, 'checks': [], 'passed': False}
    key = lambda: str(uuid.uuid4())
    try:
        status, health = api.request('GET', '/api/health')
        assert status == 200 and health.get('service') == 'hegemony-worker', f'Health {status}: {health}'
        status, catalog = api.request('GET', '/api/catalog')
        assert status == 200 and catalog['engineVersion'] == 'rust-v0.2.1' and len(catalog['cards']) == 29
        assert catalog['entryIdempotency'] is True
        create = {'name': '远端并发甲', 'mode': 'teams', 'deckId': 'responders', 'requestId': key()}
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            hosts = list(pool.map(lambda _: api.request('POST', '/api/rooms', create), range(2)))
            assert all(status == 200 for status, _ in hosts) and hosts[0] == hosts[1]
            host = hosts[0][1]
            assert api.request('POST', '/api/rooms', {**create, 'name': '异意图'})[0] == 409
            joins = [{'inviteCode': host['inviteCode'], 'name': f'远端并发{i}', 'deckId': 'responders', 'requestId': key()} for i in range(1, 4)]
            joined = list(pool.map(lambda body: api.request('POST', '/api/rooms/join', body), joins))
            assert all(status == 200 for status, _ in joined), [status for status, _ in joined]
            players = [host, *(value for _, value in joined)]
            assert len({player['seat'] for player in players}) == 4
            assert api.request('POST', '/api/rooms/join', joins[0]) == joined[0]
            evidence['checks'].append('concurrent-create-and-three-distinct-joins-with-original-receipts')
            endpoint = f'/api/rooms/{host["roomId"]}/commands'
            command = {'commandId': key(), 'expectedVersion': 3, 'action': {'kind': 'ready'}}
            identical = list(pool.map(lambda _: api.request('POST', endpoint, command, host['token']), range(2)))
            assert all(status == 200 for status, _ in identical) and identical[0] == identical[1]
            assert api.request('POST', endpoint, command)[0] == 401
            assert api.request('POST', endpoint, command, 'b' * 64)[0] == 401
            assert api.request('POST', endpoint, command, players[1]['token'])[0] == 409
            assert api.request('POST', endpoint, {**command, 'action': {'kind': 'pass'}}, host['token'])[0] == 409
            evidence['checks'].append('auth-before-dedupe-identical-command-once-changed-intent-and-other-seat-rejected')
            races = list(pool.map(lambda session: api.request('POST', endpoint,
                {'commandId': key(), 'expectedVersion': 4, 'action': {'kind': 'ready'}}, session['token']), players[1:3]))
            assert sorted(status for status, _ in races) == [200, 409], [status for status, _ in races]
            current = api.state(host)
            assert current['version'] == 5
            stale = api.request('POST', endpoint, {'commandId': key(), 'expectedVersion': 4, 'action': {'kind': 'ready'}}, host['token'])
            assert stale[0] == 409 and stale[1]['view']['version'] == 5
            assert api.request('POST', endpoint, command, host['token']) == identical[0]
            assert api.state(host) == current
            assert api.request('GET', f'/api/rooms/{host["roomId"]}/state?afterVersion=5', token=host['token'])[0] == 204
            evidence['checks'].append('two-different-commands-one-CAS-winner-stale-rejected-original-ACK-stable-no-state-change')
            _, duel = api.request('POST', '/api/rooms', {'name': '远端末席甲', 'mode': 'duel', 'deckId': 'watchers', 'requestId': key()})
            last = {'inviteCode': duel['inviteCode'], 'name': '远端末席乙', 'deckId': 'responders', 'requestId': key()}
            guests = list(pool.map(lambda _: api.request('POST', '/api/rooms/join', last), range(2)))
            assert all(status == 200 for status, _ in guests) and guests[0] == guests[1]
            assert len(api.state(duel)['players']) == 2 and api.state(duel)['version'] == 1
            evidence['checks'].append('last-seat-duplicate-join-recovers-one-original-seat-and-credential')
        evidence.update(passed=True, rooms=[{'roomId': host['roomId'], 'mode': 'teams', 'version': 5,
            'expectedPersistedCounts': {'seats': 4, 'commands': 2, 'journal': 5, 'entry_receipts': 4}},
            {'roomId': duel['roomId'], 'mode': 'duel', 'version': 1,
            'expectedPersistedCounts': {'seats': 2, 'commands': 0, 'journal': 1, 'entry_receipts': 2}}])
    except Exception as error:
        evidence['failure'] = f'{type(error).__name__}: {error}'
        raise
    finally:
        write_json(output / 'protocol-summary.json', evidence)
        print(json.dumps(evidence, ensure_ascii=False), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--output', required=True)
    arguments = parser.parse_args()
    main(arguments, getpass.getpass('Private Site QA bearer (input hidden): '))
