#!/usr/bin/env python3
"""Compare owned, idle production rooms across a release without new game actions.

Seat credentials are accepted only on hidden stdin. Evidence contains hashes,
public versions and original receipt identities, never private player views.
"""
import argparse
import getpass
import hashlib
import json
import os
from pathlib import Path

from playwright.sync_api import sync_playwright
from common import write_json


def digest(value):
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def main(args, groups):
    output = Path(args.output)
    result = {'testType': 'production-owned-idle-room-release-compatibility', 'stage': args.stage,
              'passed': False, 'newGameplayCommands': 0, 'newRoomsCreated': 0, 'siteBypassUsed': False, 'rooms': []}
    with sync_playwright() as playwright:
        client = playwright.request.new_context(proxy={'server': os.environ['HTTPS_PROXY']})
        def request(path, actor=None, command=None):
            response = client.fetch(args.base_url.rstrip('/') + path, method='POST' if command else 'GET',
                headers={'Content-Type': 'application/json', **({'Authorization': 'Bearer ' + actor['token']} if actor else {})},
                data=command, max_redirects=0, timeout=20000)
            assert response.status == 200, f'Unexpected HTTP {response.status}'
            return response.json()
        try:
            previous = json.loads((output / 'before.json').read_text()) if args.stage == 'after' else None
            result['health'] = request('/api/health')
            result['currentCatalogEngine'] = request('/api/catalog')['engineVersion']
            if args.stage == 'after':
                assert result['health']['engineVersion'] == args.engine
                assert result['currentCatalogEngine'] == args.engine
            for index, group in enumerate(groups):
                actors = group['sessions']; room_id = actors[0]['roomId']
                assert all(actor['roomId'] == room_id for actor in actors)
                path = '/api/rooms/' + room_id
                views = [request(path + '/state', actor) for actor in actors]
                assert all(view['versions']['engine'] == group['engine'] for view in views)
                assert len({view['version'] for view in views}) == 1
                room = {'roomId': room_id, 'engine': group['engine'], 'version': views[0]['version'],
                        'viewHashes': [digest(view) for view in views]}
                replay = request(path + '/commands', actors[group['commandSeat']], group['originalCommand'])
                room['originalReceiptHash'] = digest(replay)
                room['originalReceiptVersion'] = replay['version']
                assert [request(path + '/state', actor) for actor in actors] == views
                if args.stage == 'after':
                    old = previous['rooms'][index]
                    assert all(room[key] == old[key] for key in ('roomId', 'engine', 'version', 'viewHashes', 'originalReceiptHash', 'originalReceiptVersion'))
                    catalog = request(path + '/catalog', actors[0])
                    assert catalog['engineVersion'] == group['engine'] and len(catalog['cards']) == 29
                    keepers = next(deck for deck in catalog['decks'] if deck['id'] == 'keepers')
                    assert next(entry['count'] for entry in keepers['cards'] if entry['cardId'] == 'JC125') == 17
                    assert all(entry['cardId'] != 'JC058' for entry in keepers['cards'])
                    room['roomCatalogMatchesOriginalKernel'] = True
                result['rooms'].append(room)
            result['passed'] = True
        except Exception as error:
            result['failureType'] = type(error).__name__
        finally:
            client.dispose()
            write_json(output / (args.stage + '.json'), result)
    print(json.dumps(result, ensure_ascii=False), flush=True)
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--base-url', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--stage', choices=['before', 'after'], required=True)
    parser.add_argument('--engine', default='rust-v0.2.3')
    args = parser.parse_args()
    raise SystemExit(main(args, json.loads(getpass.getpass('Owned room groups (hidden): '))))
