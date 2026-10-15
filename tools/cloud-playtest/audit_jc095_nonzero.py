#!/usr/bin/env python3
"""Recompute the bounded online JC095 proof and distinguish HTTP attempts from intents."""
import argparse,json
from collections import Counter
from pathlib import Path
from common import write_json

def action(value):return value.get('action',value)
def lines(path):return [json.loads(line) for line in Path(path).read_text().splitlines()]

def main(args):
    summary=json.loads(Path(args.summary).read_text())
    reopen=json.loads(Path(args.reopen).read_text())
    history=json.loads(Path(args.history).read_text())
    assert summary['passed'] and summary['boundedMechanismProbe'] and all(summary['pausedViews'])
    assert summary['roomId']!=history['originalRoomId']
    assert reopen['passed'] and reopen['gamePosts']==0 and not reopen['browserErrors']
    requests={};terminal={}
    for event in lines(args.trace):
        sequence=event.get('sequence')
        if event['event']=='request':
            assert sequence not in requests;requests[sequence]=event
        elif event['event'] in ('response-body','request-failed','response-body-unavailable'):terminal[sequence]=event
    groups={}
    for sequence,request in requests.items():
        key=request['commandId'];identity=(request['seat'],request['expectedVersion'],json.dumps(request['action'],sort_keys=True,separators=(',',':')))
        group=groups.setdefault(key,dict(identity=identity,requests=[],successes=[],rejections=[]))
        assert group['identity']==identity,'A reused ID changed actor, expected version or intent'
        end=terminal.get(sequence,{})
        row=dict(sequence=sequence,seat=request['seat'],commandId=key,expectedVersion=request['expectedVersion'],
            action=request['action'],requestedAtEpochSeconds=request['observedAt'],terminalEvent=end.get('event'),
            cause=end.get('cause'),httpStatus=end.get('status'),errorCode=end.get('errorCode'),
            receiptVersion=(end.get('publicView') or {}).get('version'))
        group['requests'].append(row)
        if end.get('event')=='response-body' and end.get('status')==200:
            view=end['publicView'];assert view['you']==f'p{request["seat"]}' and view['version']>request['expectedVersion']
            group['successes'].append(row)
        elif end.get('event')=='response-body':
            assert end['status']==409 and end['errorCode']=='window_expired','Unexpected HTTP rejection'
            assert any(r['commandId']==key for r in summary['expiryRaces'])
            group['rejections'].append(row)
    failures=[];unresolved=[]
    for key,group in groups.items():
        if not group['successes'] and not group['rejections']:unresolved.append(key)
        for row in group['requests']:
            if row['terminalEvent']!='response-body':
                failures.append(dict(**row,logicalOutcomeKnown=bool(group['successes']),
                    sameOriginalIdSuccessfulReceipts=[dict(sequence=r['sequence'],status=r['httpStatus'],version=r['receiptVersion']) for r in group['successes']]))
    assert not unresolved and all(sequence in terminal for sequence in requests)
    assert all(r['terminalEvent']=='request-failed' and r['cause']=='ERR_ABORTED' and r['sameOriginalIdSuccessfulReceipts'] for r in failures)
    records=lines(args.actions);by_id={r['commandId']:r for r in records};entry=summary['entry'];goal=summary['goal']
    deploy=by_id[entry['deployCommandId']];accept=by_id[entry['triggerCommandId']]
    assert deploy['seat']==entry['actorSeat'] and deploy['expectedVersion']==entry['beforeEntryVersion']
    assert action(deploy['action'])['kind']=='deploy' and action(deploy['action'])['region']==entry['region']
    assert accept['seat']==entry['actorSeat'] and action(accept['action'])['kind']=='choose'
    assert action(accept['action'])['choiceId']==entry['triggerChoiceId'] and action(accept['action'])['selected']==['accept']
    assert deploy['httpStatus']==accept['httpStatus']==200 and entry['enemyBefore']>0
    expected=list(entry['beforeInfluence']);expected[entry['actorTeam']]+=entry['enemyBefore'];expected[1-entry['actorTeam']]=0
    assert expected==goal['expectedInfluence']
    for check in goal['views']:
        region=check['publicView']['regions'][entry['region']]
        assert region['cardId']==entry['regionCardId'] and region['influence']==expected
        assert any(c['instanceId']==check['sourceInstanceId'] and c.get('cardId')=='JC095' and c['controller']==f'p{entry["actorSeat"]}' for c in region['characters'])
    assert len(goal['views'])==len(reopen['checks'])==2
    assert len({r['version'] for r in goal['views']})==1
    for check in reopen['checks']:
        region=check['publicView']['regions'][entry['region']]
        assert check['paused'] and check['version']==summary['final']['version']
        assert region['cardId']==entry['regionCardId'] and region['influence']==expected
    entry_responses=sum(r['endpoint'] in ('rooms','rooms/join') for r in summary['postResults'])
    assert entry_responses==2 and summary['uiPostCount']==len(requests)+entry_responses
    assert len(failures)==summary['uiPostCount']-len(summary['postResults'])
    result=dict(passed=True,scope='one actual nonzero counter conversion and its paused two-seat readback',
        roomId=summary['roomId'],siteVersion=summary['siteVersion'],turn=summary['final']['turn'],
        actorSeat=entry['actorSeat'],regionCardId=entry['regionCardId'],beforeInfluence=entry['beforeInfluence'],afterInfluence=expected,
        convertedEnemyMarkers=entry['enemyBefore'],conversionViewVersion=goal['views'][0]['version'],pausedReadbackVersion=summary['final']['version'],
        uiPostAttempts=summary['uiPostCount'],observedResponses=len(summary['postResults']),
        responseStatuses=dict(Counter(str(r['status']) for r in summary['postResults'])),entryResponses=entry_responses,
        tracedCommandHttpAttempts=len(requests),distinctOriginalCommandIds=len(groups),
        distinctAcceptedCommandIds=sum(bool(g['successes']) for g in groups.values()),
        distinctRejectedCommandIds=sum(bool(g['rejections']) and not g['successes'] for g in groups.values()),
        failedAttemptCount=len(failures),failedTransportAttempts=failures,unresolvedLogicalCommandIds=unresolved,
        allCapturedIntentIdentitiesStable=True,allTrackedRequestsHaveTerminalEvent=True,
        deployAndAcceptCommandMappingsVerified=True,pausedReadbackGamePosts=reopen['gamePosts'],
        historicalOldTableMissingHttpAttemptsNotIncluded=history['unchangedCounters']['historicalMissingResponses'],
        historicalOriginalCommandDurableVersion=history['persistedCommandMetadata']['version'],
        limitation='Observed aborts remain aborted HTTP attempts; same-ID receipts verify their logical outcomes. The old four-seat three unidentifiable attempts remain a separate historical limitation. This proves one actual counter case, not every JC095 edge case.')
    write_json(Path(args.output),result)
    print(json.dumps({k:v for k,v in result.items() if k!='failedTransportAttempts'},ensure_ascii=False))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('summary','trace','actions','reopen','history','output'):parser.add_argument('--'+name,required=True)
    main(parser.parse_args())
