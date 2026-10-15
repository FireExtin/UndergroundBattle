#!/usr/bin/env python3
"""Reopen the same two owned paused JC095 probe profiles without gameplay clicks."""
import argparse,asyncio,json
from pathlib import Path
from playwright.async_api import async_playwright
from attachment_v026 import AttachmentRun
from common import public_view,write_json
from public_abi77_teams import ENGINE,POOL

async def main(args):
    profiles=Path(args.profiles).resolve();output=Path(args.output).resolve()
    assert not output.exists(),'Retain previous readback evidence'
    previous=json.loads((profiles/'target-summary.json').read_text())
    assert previous['passed'] and previous['goal'] and all(previous['pausedViews'])
    output.mkdir(parents=True)
    class Remote:remote=True;base_url=args.url
    result=dict(passed=False,roomId=previous['roomId'],scope='same owned paused probe profiles; no gameplay clicks',
        publicAnonymous=True,ownerBypass=False,stateInjection=False,apiMoves=False,checks=[])
    async with async_playwright() as p:
        run=AttachmentRun(p,Remote(),profiles);run.mode='duel'
        try:
            for seat in range(2):
                page=await run.new_seat(seat)
                await page.set_viewport_size({'width':1366,'height':900});await page.bring_to_front()
                await page.wait_for_function('(id)=>window.__cloudView?.roomId===id&&!!window.__cloudView.pause',arg=previous['roomId'],timeout=30000)
            run.output=output
            await run.sync(previous['final']['version'])
            for seat,view in enumerate(await run.views()):
                assert view['roomId']==previous['roomId'] and view['you']==f'p{seat}' and view['mode']=='duel'
                assert view['status']=='playing' and view.get('pause')
                assert view['version']==previous['final']['version']
                assert view['versions']['engine']==ENGINE and view['versions']['cardPool']==POOL
                assert not view.get('pendingChoice') and not view.get('waitingChoice') and not view['stack']
                entry=previous['entry'];region=view['regions'][entry['region']]
                assert region['cardId']==entry['regionCardId'] and region['influence']==previous['goal']['expectedInfluence']
                source=previous['goal']['views'][0]['sourceInstanceId']
                assert any(c['instanceId']==source and c.get('cardId')=='JC095' and c['controller']==f'p{entry["actorSeat"]}' and not c['faceDown'] for c in region['characters'])
                result['checks'].append(dict(seat=seat,ownIdentityRestored=True,paused=True,version=view['version'],
                    nonzeroConversionPersisted=True,publicView=public_view(view)))
            await run.screenshot('jc095-nonzero-paused-reopened')
            result['viewsPassed']=True
        except BaseException as error:
            result.update(errorType=type(error).__name__,failure=str(error).split('\n')[0][:300])
        finally:
            await asyncio.gather(*(context.close() for context in run.contexts),return_exceptions=True)
            result.update(gamePosts=run.post_count,browserErrors=run.errors,transportEvents=run.transport_events)
            result['passed']=bool(result.get('viewsPassed') and run.post_count==0 and not run.errors)
            write_json(output/'paused-reopen-summary.json',result)
    print(json.dumps({k:result.get(k) for k in ['passed','roomId','gamePosts','failure']}),flush=True)
    assert result['passed'],'Paused probe readback did not satisfy the recorded bounded checks'

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--url',required=True);parser.add_argument('--profiles',required=True);parser.add_argument('--output',required=True)
    asyncio.run(main(parser.parse_args()))
