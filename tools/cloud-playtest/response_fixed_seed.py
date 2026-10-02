#!/usr/bin/env python3
"""Test-only seeded empty lobby, legal HTTP prefix, then real response UI.

The production WASM Game::new creates only the host lobby. Replace neither
hands nor board state; retain HTTP-created credentials and replay legal commands.
This deliberately uses a seed fixture and never counts as a complete UI game.
"""
import argparse
import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
from pathlib import Path

from playwright.sync_api import sync_playwright

from common import Api, IsolatedService, action_payload, public_view, write_json
from response_v2 import BrowserResponse, Fixture, character_cards, evidence_view


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main(args):
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    artifacts = output / 'bin'
    artifacts.mkdir(exist_ok=True)
    bindings = artifacts / 'hegemony_wasm.mjs'
    wasm = artifacts / 'hegemony_wasm_bg.wasm'
    shutil.copy2(args.wasm_bindings, bindings)
    shutil.copy2(args.wasm, wasm)
    source_database, source_trace = Path(args.source_database).resolve(), Path(args.source_trace).resolve()
    with sqlite3.connect(source_database.as_uri() + '?mode=ro', uri=True) as connection:
        initial = connection.execute('SELECT initial_state FROM rooms WHERE id=?', (args.source_room,)).fetchone()
        assert initial, 'Historical successful room is missing'
        old = json.loads(initial[0])
        assert old['mode'] == 'duel' and old['version'] == 0 and len(old['players']) == 1
        seed_decimal = str(old['seed'])
    prefix = [json.loads(line) for line in source_trace.read_text().splitlines() if line]
    assert len(prefix) == args.expected_prefix_commands
    private_seed = output / 'private-fixed-seed.json'
    descriptor = os.open(private_seed, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'w') as stream:
        json.dump({'privateLocalOnly': True, 'seedDecimal': seed_decimal,
                   'sourceDatabase': str(source_database), 'sourceRoom': args.source_room}, stream)
    metadata = {'testType': 'fixed-seed-lobby-legal-http-prefix-real-browser-response',
                'sourceCommit': args.source_commit, 'engineVersion': args.engine_version,
                'binary': str(Path(args.binary).resolve()), 'binarySha256': sha(args.binary),
                'wasmSha256': sha(wasm), 'wasmBindingsSha256': sha(bindings),
                'testOnlyFixedSeedLobbyBootstrap': True, 'initialSeedFixture': True,
                'seedOverride': True, 'intermediateStateInjection': False,
                'countsAsCompleteUiGame': False, 'sourceDatabase': str(source_database),
                'sourceRoom': args.source_room, 'sourceTrace': str(source_trace),
                'sourceTraceSha256': sha(source_trace), 'privateSeedArtifact': str(private_seed),
                'privateSeedSha256': sha(private_seed), 'database': str(output / 'acceptance.sqlite3'),
                'port': args.port, 'checks': [], 'passed': False}
    service = IsolatedService(args.binary, args.cwd, output, args.port, args.static_dir)
    fixture = None
    run = None
    try:
        service.start()
        api = Api(service.base_url)
        status, catalog = api.request('GET', '/api/catalog')
        assert status == 200 and catalog['engineVersion'] == args.engine_version
        assert len(catalog['cards']) == 29
        status, host = api.request('POST', '/api/rooms', {
            'name': old['players'][0]['name'], 'mode': 'duel', 'deckId': old['players'][0]['deck_id']})
        assert status == 200
        metadata['roomId'] = host['roomId']
        request = {'roomId': host['roomId'], 'inviteCode': host['inviteCode'], 'mode': 'duel',
                   'name': host['view']['players'][0]['name'], 'deckId': old['players'][0]['deck_id'],
                   'seedDecimal': seed_decimal}
        process = subprocess.run(['node', str(Path(__file__).with_name('wasm_new_lobby.mjs')),
                                  str(bindings), str(wasm)], input=json.dumps(request),
                                 capture_output=True, text=True, check=True)
        envelope = json.loads(process.stdout)
        opaque_state = envelope['state']  # Store the exact Rust serialization; never re-encode it.
        assert envelope['version'] == 0 and envelope['seat'] == 0
        assert envelope['view'] == host['view'], 'The seeded lobby changed the public host view'
        service.stop()  # Store caches rooms: import only while this owned process is stopped.
        with sqlite3.connect(output / 'acceptance.sqlite3') as connection:
            connection.execute('BEGIN IMMEDIATE')
            row = connection.execute('SELECT invite,initial_state,state FROM rooms WHERE id=?', (host['roomId'],)).fetchone()
            assert row and row[0] == host['inviteCode'] and row[1] == row[2]
            before = json.loads(row[2])
            fresh = json.loads(opaque_state)  # Python preserves u64 integers exactly; use only for assertions.
            assert before['version'] == fresh['version'] == 0
            assert before['status'] == fresh['status'] == 'lobby'
            assert len(before['players']) == len(fresh['players']) == 1
            assert all(not before[k] and not fresh[k] for k in ('regions', 'world', 'stack', 'effects'))
            assert all(not p[k] for g in (before, fresh) for p in g['players']
                       for k in ('hand', 'deck', 'assets', 'graveyard', 'score_cards'))
            assert fresh['seed'] == int(seed_decimal)
            assert {k: v for k, v in before.items() if k not in ('seed', 'random')} == {
                k: v for k, v in fresh.items() if k not in ('seed', 'random')}
            seats = connection.execute('SELECT seat,token_hash FROM seats WHERE room_id=?', (host['roomId'],)).fetchall()
            assert len(seats) == 1 and seats[0][0] == 0
            assert seats[0][1] == hashlib.sha256(host['token'].encode()).hexdigest()
            for table in ('commands', 'journal'):
                assert connection.execute(f'SELECT COUNT(*) FROM {table} WHERE room_id=?', (host['roomId'],)).fetchone()[0] == 0
            connection.execute('UPDATE rooms SET initial_state=?,state=? WHERE id=?',
                               (opaque_state, opaque_state, host['roomId']))
            assert connection.execute('SELECT seat,token_hash FROM seats WHERE room_id=?', (host['roomId'],)).fetchall() == seats
        metadata['checks'].append({'name': 'empty-host-lobby-bootstrap', 'publicViewUnchanged': True,
                                   'onlySeedAndRandomChanged': True, 'credentialsPreserved': True,
                                   'priorJournalEntries': 0, 'priorCommands': 0})
        service.start()
        assert api.state(host) == envelope['view']
        status, guest = api.request('POST', '/api/rooms/join', {
            'inviteCode': host['inviteCode'], 'name': '响应乙', 'deckId': 'responders'})
        assert status == 200
        fixture = Fixture.__new__(Fixture)
        fixture.api, fixture.output, fixture.mode, fixture.max_steps = api, output, 'duel', len(prefix)
        fixture.sessions, fixture.records, fixture.probes = [host, guest], [], []
        fixture.catalog = catalog
        fixture.definitions = {card['id']: card for card in catalog['cards']}
        fixture.room_id = host['roomId']
        for index, record in enumerate(prefix):
            seat, action = record['seat'], record['action']
            before = api.state(fixture.sessions[seat])
            assert before['version'] == record['beforeVersion'], f'Prefix version diverged at {index}'
            if action['kind'] != 'choose':
                assert action_payload(action) in [action_payload(a) for a in before['legalActions']], f'Prefix action is not currently legal at {index}'
            else:
                assert before.get('pendingChoice') and before['pendingChoice']['id'] == action['choiceId'], f'Prefix choice diverged at {index}'
            after = fixture.command(seat, action)
            assert after['version'] == record['afterVersion'], f'Prefix result version diverged at {index}'
            with (output / 'prefix-validation.jsonl').open('a') as stream:
                stream.write(json.dumps({'prefixIndex': index, 'seat': seat, 'kind': action['kind'],
                                         'httpStatus': 200, 'beforeVersion': before['version'],
                                         'afterVersion': after['version'], 'currentlyLegal': True}) + '\n')
        views = fixture.views()
        match = None
        for actor, view in enumerate(views):
            for action in view['legalActions']:
                source = next((c for c in view['hand'] if c['instanceId'] == action.get('cardId')), None)
                victim = next((c for c in character_cards(view) if c['instanceId'] == action.get('targetId')
                               and c.get('cardId') == 'JC042' and c['controller'] != view['you']), None)
                if action['kind'] == 'play' and source and source.get('cardId') == 'JC091' and victim:
                    match = actor, next(i for i, own in enumerate(views) if own['you'] == victim['controller']), action, victim['instanceId']
                    break
        assert match, 'Historical legal prefix ended without the real murder/disciple declaration'
        metadata['checks'].append({'name': 'historical-prefix-validated', 'acceptedApiCommands': len(fixture.records),
                                   'everyBeforeAfterVersionMatched': True, 'preparedTurn': views[0]['turn']})
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=args.chromium, headless=True,
                                                 args=['--no-sandbox', '--disable-dev-shm-usage'])
            run = BrowserResponse(browser, fixture, service, output)
            try:
                run.open(match[0], match[1])
                run.run_chain(*match)
                assert not run.errors
                metadata.update(passed=True, uiAcceptedCommands=len(run.commands), apiProbeRequests=len(fixture.probes),
                                browserErrors=run.errors, expectedRestartNetworkErrors=run.expected_errors,
                                screenshots=run.shots, final=evidence_view(run.views()[0]))
                metadata['checks'].extend(run.checks)
            finally:
                run.close()
                browser.close()
    except Exception as error:
        metadata['failure'] = f'{type(error).__name__}: {error}'
        if run:
            metadata.update(browserErrors=run.errors, expectedRestartNetworkErrors=run.expected_errors)
        raise
    finally:
        service.stop()
        metadata.update(ownedServiceStopped=True, fixtureAcceptedCommands=len(fixture.records) if fixture else 0,
                        uiAcceptedCommands=len(run.commands) if run else 0)
        write_json(output / 'response-summary.json', metadata)
        print(json.dumps({k: metadata[k] for k in ('passed', 'fixtureAcceptedCommands', 'uiAcceptedCommands', 'ownedServiceStopped')}, ensure_ascii=False), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True)
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--output', default='/tmp/hegemony-response-v2.1-fixedseed-2026-10-02')
    parser.add_argument('--port', type=int, default=8100)
    parser.add_argument('--static-dir', default='web/dist')
    parser.add_argument('--engine-version', default='rust-v0.2.1')
    parser.add_argument('--source-commit', default='d94baaf3e7caabfdb81e1b3cc2a537fefe355fd0')
    parser.add_argument('--wasm-bindings', default='rust-game-wasm/pkg/hegemony_wasm.js')
    parser.add_argument('--wasm', default='rust-game-wasm/pkg/hegemony_wasm_bg.wasm')
    parser.add_argument('--source-database', default='/tmp/hegemony-response-v2-2026-10-02/final/acceptance.sqlite3')
    parser.add_argument('--source-room', default='c5c4d94fdeb33c891007b1a8')
    parser.add_argument('--source-trace', default='/tmp/hegemony-response-v2-2026-10-02/final/duel-fixture-commands.jsonl')
    parser.add_argument('--expected-prefix-commands', type=int, default=364)
    parser.add_argument('--chromium', default='/usr/bin/chromium')
    main(parser.parse_args())
