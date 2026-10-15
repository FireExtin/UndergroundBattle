#!/usr/bin/env python3
"""Validate actual JC046 choices without treating hidden cards as characters."""
import argparse,json
from pathlib import Path
from common import write_json

def main(args):
    summary=json.loads(Path(args.summary).read_text())
    rows=[json.loads(line) for line in Path(args.actions).read_text().splitlines()]
    by_choice={}
    for row in rows:
        choice=row['before'].get('pendingChoice')
        if choice:by_choice[choice['id']]=row
    privacy={row['choiceId']:row for row in summary['choicePrivacy']}
    sequences=[]
    for frame,sequence in summary['jc046Sequences'].items():
        assert sequence['choiceIds']
        first=by_choice[sequence['choiceIds'][0]]['before']
        live=[c for region in first['regions'] for c in region['characters'] if not c['faceDown'] and c['kind']=='character']
        eligible=sorted({int(c['controller'][1:]) for c in live})
        prompted=sequence['seats']
        checks=[]
        for seat,choice_id in zip(prompted,sequence['choiceIds']):
            row=by_choice[choice_id];view=row['before'];choice=view['pendingChoice']
            assert row['seat']==seat and row['httpStatus']==200
            assert choice['min']==choice['max']==1 and not choice['allowDecline']
            assert choice['playerId']==view['you']==f'p{seat}'
            candidates=[c for region in view['regions'] for c in region['characters'] if c['controller']==view['you'] and not c['faceDown'] and c['kind']=='character']
            assert choice['optionCount']==len(candidates)
            action=row['action'];action=action.get('action',action)
            assert action['kind']=='choose' and len(action['selected'])==1
            selected=action['selected'][0];assert selected in {c['instanceId'] for c in candidates}
            assert row['after']['version']==view['version']+1
            p=privacy[choice_id]
            assert p['fourViewsChecked'] and p['otherSeatsHaveNoPrivateChoice']
            assert all(n==0 for i,n in enumerate(p['viewerPrivateOptionCounts']) if i!=seat)
            assert any(c['name']=='four-seat-choice-refresh-restores-own-draft' and c['choiceId']==choice_id for c in summary['checks'])
            assert any(c['name']=='four-seat-rapid-choice-clicks-one-command' and c['choiceId']==choice_id for c in summary['checks'])
            checks.append(dict(seat=seat,choiceId=choice_id,versionBefore=view['version'],versionAfter=row['after']['version'],optionCount=len(candidates),selected=selected,privateOtherViewsChecked=3,refreshAndDoubleClickChecked=True,passed=True))
        # The currently observed character-only traces have no intervening
        # character creation or removal beyond each selected sacrifice.
        assert prompted==eligible,(frame,prompted,eligible)
        later=[v for row in rows for v in [row.get('before'),row.get('after')] if v and v['version']>=checks[-1]['versionAfter'] and not v['stackCount'] and not v.get('pendingChoice') and not v.get('waitingChoice')]
        sequences.append(dict(frameId=frame,eligibleAtFirstChoice=eligible,promptedSeats=prompted,
            skippedSeatsWithNoFaceupCharacter=[i for i in range(4) if i not in eligible],
            allEligibleHandled=True,fourEligibleSeatsObserved=eligible==[0,1,2,3],
            resolutionObservedAfterChoices=bool(later),checks=checks,passed=True))
    result=dict(passed=all(s['resolutionObservedAfterChoices'] for s in sequences),
        scope='actual four-player table; only face-up controlled characters qualify',
        sourceContract='rust-game/src/jc046.rs jc046_options: controller + !face_down + character',
        stateInjection=False,apiMoves=False,sequenceCount=len(sequences),
        fourEligibleSeatSequenceObserved=any(s['fourEligibleSeatsObserved'] for s in sequences),
        sequences=sequences,
        limitation='Successful eligible-seat subsets are not relabeled as a four-eligible-seat scenario.')
    write_json(Path(args.output),result)
    print(json.dumps({k:v for k,v in result.items() if k!='sequences'}))
    assert result['passed'],'An observed sequence has not yet reached a subsequent resolved public state'

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--summary',required=True);p.add_argument('--actions',required=True);p.add_argument('--output',required=True)
    main(p.parse_args())
