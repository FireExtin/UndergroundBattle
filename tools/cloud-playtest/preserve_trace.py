#!/usr/bin/env python3
"""Preserve a room's exact local replay input without seat credentials.

The raw replay file contains private authoritative cards/seed and stays local.
Only the separate summary is suitable for public acceptance reporting.
"""
import argparse
import hashlib
import json
import sqlite3
from pathlib import Path

from common import write_json


def main(args):
    source = Path(args.database).resolve()
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    with sqlite3.connect(source.as_uri() + '?mode=ro', uri=True) as connection:
        row = connection.execute('SELECT id,initial_state,state FROM rooms WHERE id=?', (args.room_id,)).fetchone()
        assert row, 'Room not found in local database'
        journal = [dict(version=version, entry=json.loads(entry)) for version, entry in connection.execute('SELECT version,entry FROM journal WHERE room_id=? ORDER BY version', (args.room_id,))]
    replay = {'privateLocalOnly': True, 'roomId': row[0], 'initialState': json.loads(row[1]), 'currentState': json.loads(row[2]), 'journal': journal}
    write_json(output / 'private-room-replay.json', replay)
    raw = Path(args.trace)
    actions = [json.loads(line) for line in raw.read_text().splitlines() if line] if raw.suffix == '.jsonl' else json.loads(raw.read_text())
    asset_samples, draws = [], []
    for record in actions:
        before, after = record.get('before'), record.get('after')
        if not before or not after: continue
        if record['kind'] == 'asset' and before['handCount'] == 1 and after['handCount'] == 0:
            asset_samples.append({'seat': record['seat'], 'turn': before['turn'], 'beforeVersion': before['version'], 'afterVersion': after['version'], 'httpStatus': record['httpStatus'], 'legalActionKinds': before['legalActionKinds']})
        if before['step'] == 'prepare' and after['step'] == 'draw':
            old = {p['id']: p for p in before['players']}
            changes = []
            for player in after['players']:
                previous = old[player['id']]
                if not previous['eliminated'] and previous['deckCount']:
                    assert player['deckCount'] == previous['deckCount'] - 1
                    assert player['handCount'] == previous['handCount'] + 1
                changes.append({'seat': player['seat'], 'handChange': player['handCount'] - previous['handCount'], 'deckChange': player['deckCount'] - previous['deckCount'], 'eliminated': player['eliminated']})
            draws.append({'turn': before['turn'], 'changes': changes})
    final = actions[-1]['after']
    summary = {'testType': 'preserved-original-real-ui-policy', 'traceActions': len(actions),
               'policySource': 'tools/cloud-playtest/initial_policy.py',
               'allRecordedCommandsAccepted': all(record['httpStatus'] == 200 for record in actions),
               'soleTopdeckAssetCount': len(asset_samples), 'soleTopdeckAssetSamples': asset_samples[:8],
               'observedDrawOneTransitions': len(draws), 'drawTransitions': draws,
               'finalTurn': final['turn'], 'finalStatus': final['status'],
               'scores': [p['score'] for p in final['players']], 'deckCounts': [p['deckCount'] for p in final['players']],
               'playersEliminated': [p['eliminated'] for p in final['players']],
               'journalEntries': len(journal),
               'privateReplaySha256': hashlib.sha256((output / 'private-room-replay.json').read_bytes()).hexdigest(),
               'privateReplayContainsNoSeatTableOrTokenHashes': True}
    write_json(output / 'original-policy-summary.json', summary)
    print(json.dumps({key: summary[key] for key in ('traceActions', 'allRecordedCommandsAccepted', 'soleTopdeckAssetCount', 'observedDrawOneTransitions', 'finalTurn', 'finalStatus')}), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--database', required=True)
    parser.add_argument('--room-id', required=True)
    parser.add_argument('--trace', required=True)
    parser.add_argument('--output', required=True)
    main(parser.parse_args())
