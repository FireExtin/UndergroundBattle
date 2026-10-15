#!/usr/bin/env python3
"""Audit observed UI receipts without inventing identities for untraced attempts."""
import argparse,json
from collections import Counter
from pathlib import Path
from common import write_json

def game_action(action):
    return action.get('action',action)

def main(args):
    summary=json.loads(Path(args.summary).read_text())
    records=[json.loads(line) for line in Path(args.actions).read_text().splitlines()]
    identities={};confirmed=set();rejections=[];trace_attempts=[]
    def identify(command_id,seat,expected,action):
        assert command_id
        identity=(seat,expected,json.dumps(action,sort_keys=True,ensure_ascii=False))
        assert command_id not in identities or identities[command_id]==identity,'A command ID changed its original actor/version/action'
        identities[command_id]=identity
    for record in records:
        identify(record['commandId'],record['seat'],record['expectedVersion'],record['action'])
        if record['httpStatus']==200:confirmed.add(record['commandId'])
        else:rejections.append(dict(commandId=record['commandId'],seat=record['seat'],
            status=record['httpStatus'],error=record.get('error'),
            actionKind=game_action(record['action'])['kind']))
    for filename in args.trace:
        events=[json.loads(line) for line in Path(filename).read_text().splitlines()]
        requests={};results={}
        for event in events:
            sequence=event.get('sequence')
            if event['event']=='request':
                assert sequence not in requests
                requests[sequence]=event
                identify(event['commandId'],event['seat'],event['expectedVersion'],event['action'])
            elif event['event'] in ('response-body','response-body-unavailable','request-failed'):
                results[sequence]=event
        for sequence,request in requests.items():
            result=results.get(sequence,{})
            view=result.get('publicView')
            receipt=result.get('event')=='response-body' and result.get('status')==200 and bool(view)
            if receipt:
                assert view['version']>request['expectedVersion']
                confirmed.add(request['commandId'])
            trace_attempts.append(dict(trace=Path(filename).name,sequence=sequence,
                commandId=request['commandId'],seat=request['seat'],expectedVersion=request['expectedVersion'],
                actionKind=game_action(request['action'])['kind'],
                terminalEvent=result.get('event'),status=result.get('status'),
                receiptVersion=view.get('version') if view else None,
                successfulReceiptObserved=receipt,errorCode=result.get('errorCode')))
    response_status=Counter(str(r['status']) for r in summary['postResults'])
    total=summary['uiPostCount'];responses=len(summary['postResults'])
    assert total-responses==summary['unansweredUiPostAttempts']
    result=dict(scope='Observed UI commands and passive traces; no API retry or storage mutation',
        roomId=summary['roomId'],naturalGameComplete=summary['completeGame'],
        uiPostAttempts=total,observedPostResponses=responses,responseStatuses=dict(response_status),
        unobservedResponseAttempts=total-responses,
        historicalMissingAttemptIdentitiesRecovered=False,
        missingAttemptLimitation='Three pre-trace attempts lack request IDs. A later original-command receipt and empty pending storage cannot prove the identity or individual outcome of those HTTP attempts.',
        manualRecords=len(records),manualActualActionKinds=dict(Counter(game_action(r['action'])['kind'] for r in records)),
        knownDistinctCommandIds=len(identities),knownDistinctSuccessfulCommandIds=len(confirmed),
        observedRejections=rejections,tracedPostAttempts=len(trace_attempts),
        tracedAttemptsWithSuccessfulReceipt=sum(r['successfulReceiptObserved'] for r in trace_attempts),
        tracedAttemptsWithoutSuccessfulReceipt=[r for r in trace_attempts if not r['successfulReceiptObserved']],
        tracedAttemptsWithoutTerminalEvent=[r for r in trace_attempts if r['terminalEvent'] is None],
        repeatedRequestIdentityInvariantChecked=True,
        allHistoricalAttemptsReconciled=total==responses,
        attempts=trace_attempts)
    write_json(Path(args.output),result)
    print(json.dumps({k:v for k,v in result.items() if k not in ('attempts','observedRejections','manualActualActionKinds','tracedAttemptsWithoutSuccessfulReceipt')},ensure_ascii=False))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--summary',required=True);p.add_argument('--actions',required=True)
    p.add_argument('--trace',action='append',default=[]);p.add_argument('--output',required=True)
    main(p.parse_args())
