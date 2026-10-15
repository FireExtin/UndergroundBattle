#!/usr/bin/env python3
"""Check JC060 combat grants in actual public UI response records.

The catalog argument supplies printed definitions only; no fixture state is used.
This checker is bounded to this four-seat character-only deck: no attachments,
combat-bonus actions or other combat auras are played. Exhausted participants
show zero effective icons, as the published view contract specifies.
"""
import argparse,json
from pathlib import Path
from common import write_json

def main(args):
    source=json.loads(Path(args.catalog).read_text())
    catalog=source.get('catalog',source)
    assert catalog['engineVersion']=='rust-v0.2.77-jc046-mandatory-sacrifice-candidate'
    definitions={c['id']:c for c in catalog['cards']}
    allowed={'JC046','JC060','JC095','JC084','JC085','JC088','JZ49','XQ37','WM056','JC042','XQ17','JZ31','XQ18','JC029','JZ22','JC069','JC125'}
    rows=[json.loads(line) for line in Path(args.actions).read_text().splitlines()]
    seen=set();checks=[];failures=[];transitions=[];last={}
    for row in rows:
        for side in ['before','after']:
            view=row.get(side)
            if not view:continue
            teams={p['id']:p['team'] for p in view['players']}
            cards=[c for r in view['regions'] for c in r['characters']]
            sources=[c for c in cards if not c['faceDown'] and c.get('cardId')=='JC060']
            for c in cards:
                if c['faceDown']:continue
                key=(view['version'],view['you'],c['instanceId'])
                if key in seen:continue
                seen.add(key)
                assert c['cardId'] in allowed,'This focused formula cannot cover an unplanned combat modifier'
                printed=definitions[c['cardId']]
                eligible=printed.get('magicIcon')=='None'
                advisers=[a for a in sources if teams[a['controller']]==teams[c['controller']]] if eligible else []
                grant=len(advisers)
                temporary=printed['temporaryIcons']['combat'] if teams[c['controller']]==view['firstTeam'] else 0
                expected=0 if c['exhausted'] else printed['permanentIcons']['combat']+temporary+grant
                actual=c['icons']['combat']
                record=dict(version=view['version'],viewer=view['you'],instanceId=c['instanceId'],cardId=c['cardId'],
                    controller=c['controller'],exhausted=c['exhausted'],printedMagic=printed.get('magicIcon'),
                    adviserCount=grant,expectedCombat=expected,actualCombat=actual,
                    teammateAdviser=any(a['controller']!=c['controller'] for a in advisers),
                    exhaustedAdviser=any(a['exhausted'] for a in advisers),
                    opponentControl=eligible and not grant and any(teams[a['controller']]!=teams[c['controller']] for a in sources),
                    passed=expected==actual)
                checks.append(record)
                if expected!=actual:failures.append(record)
                previous=last.get((view['you'],c['instanceId']))
                if previous and view['version']>previous['version'] and not c['exhausted'] and not previous['exhausted'] and previous['adviserCount']!=grant:
                    transitions.append(dict(instanceId=c['instanceId'],viewer=view['you'],fromVersion=previous['version'],toVersion=view['version'],fromAdvisers=previous['adviserCount'],toAdvisers=grant,fromCombat=previous['actualCombat'],toCombat=actual,passed=record['passed'] and previous['passed']))
                if not previous or view['version']>=previous['version']:last[(view['you'],c['instanceId'])]=record
    result=dict(passed=not failures,scope='actual public UI response before/after projections; printed catalog only',
        stateInjection=False,apiMoves=False,observations=len(checks),
        activeGrantObservations=sum(not c['exhausted'] and c['adviserCount']>0 for c in checks),
        teammateGrantObservations=sum(not c['exhausted'] and c['teammateAdviser'] for c in checks),
        exhaustedSourceObservations=sum(not c['exhausted'] and c['exhaustedAdviser'] for c in checks),
        opponentControlObservations=sum(not c['exhausted'] and c['opponentControl'] for c in checks),
        sourceCountTransitions=transitions,failures=failures,checks=checks)
    write_json(Path(args.output),result)
    print(json.dumps({k:v for k,v in result.items() if k not in ['checks','failures','sourceCountTransitions']}))
    assert not failures,'Actual public combat icons differ from the bounded JC060 grant formula'

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--actions',required=True);p.add_argument('--catalog',required=True);p.add_argument('--output',required=True)
    main(p.parse_args())
