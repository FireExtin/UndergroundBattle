#!/usr/bin/env python3
"""Read-only normal-browser reopening of the same four owned terminal seats."""
import argparse,asyncio,json
from pathlib import Path
from playwright.async_api import async_playwright
from attachment_v026 import AttachmentRun
from api_acceptance import assert_projection
from common import public_view,write_json

ENGINE='rust-v0.2.77-jc046-mandatory-sacrifice-candidate'
POOL='limited-v2.70-jc046-mandatory-sacrifice-candidate'
PRIVATE={'cardId','text','cost','icons','defense','damage','wounds','shield','color','magic',
 'currentDomains','currentSubtypes','currentRenown','currentCombatGlory','currentBarrier',
 'currentDamagePrevention','currentSpiritProtection','currentKill','currentRetreat',
 'convertedTemporaryIcons','currentPrintedDefense'}

async def main(args):
    profiles=Path(args.profiles).resolve();output=Path(args.output).resolve()
    assert not output.exists(),'Preserve prior evidence; select a new read-only output'
    output.mkdir(parents=True)
    previous=json.loads((profiles/'teams-summary.json').read_text())
    assert previous['passed'] and previous['completeGame'] and previous['final']['status']=='finished'
    class Remote:remote=True;base_url=args.url
    result=dict(passed=False,scope='same four owned browser profiles, terminal page reopen and reload',
        publicAnonymous=True,ownerBypass=False,stateInjection=False,apiMoves=False,
        roomId=previous['roomId'],viewport=[1366,900],checks=[],browserErrors=[])
    async with async_playwright() as p:
        # This opens each original profile in its original seat, never copies a token.
        run=AttachmentRun(p,Remote(),profiles)
        try:
            for seat in range(4):
                page=await run.new_seat(seat)
                await page.set_viewport_size({'width':1366,'height':900});await page.bring_to_front()
                await page.wait_for_function('(id)=>window.__cloudView?.roomId===id',arg=previous['roomId'],timeout=30000)
            run.output=output
            for stage in ['persistent-profile-reopen','four-page-reload']:
                if stage=='four-page-reload':
                    for page in run.pages:await page.reload(wait_until='domcontentloaded')
                await run.sync(previous['final']['version'])
                for seat,v in enumerate(await run.views()):
                    assert v['roomId']==previous['roomId'] and v['you']==f'p{seat}' and v['mode']=='teams'
                    assert v['status']=='finished' and v['winnerTeam']==previous['final']['winnerTeam']
                    assert v['version']==previous['final']['version']
                    assert [x['score'] for x in v['players']]==[x['score'] for x in previous['final']['players']]
                    assert v['versions']['engine']==ENGINE and v['versions']['cardPool']==POOL
                    assert not v.get('pendingChoice') and not v.get('waitingChoice')
                    assert_projection(v)
                    concealed=[]
                    for region in v['regions']:
                        for card in region['characters']:
                            if card['faceDown'] and card['controller']!=v['you']:
                                leaked=sorted(PRIVATE.intersection(card))
                                assert not leaked,f'Hidden fields exposed: {leaked}'
                                concealed.append(dict(instanceId=card['instanceId'],controller=card['controller'],privateFieldsAbsent=True))
                    result['checks'].append(dict(stage=stage,seat=seat,ownIdentityRestored=True,
                        terminalStateUnchanged=True,otherHiddenCards=concealed,publicView=public_view(v)))
            assert run.post_count==0,'Terminal profile restoration emitted an unexpected game POST'
            assert not run.errors
            result.update(passed=True,gamePosts=run.post_count,browserErrors=run.errors,transportEvents=run.transport_events)
            await run.screenshot('four-seat-terminal-reopened-p0')
        except BaseException as e:
            result.update(failure=f'{type(e).__name__}: {e}',gamePosts=run.post_count,browserErrors=run.errors)
            raise
        finally:
            await asyncio.gather(*(context.close() for context in run.contexts),return_exceptions=True)
            write_json(output/'four-seat-reopen-summary.json',result)
    print(json.dumps(dict(passed=True,checks=len(result['checks']),gamePosts=result['gamePosts'],roomId=result['roomId'])),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--url',required=True);p.add_argument('--profiles',required=True);p.add_argument('--output',required=True)
    asyncio.run(main(p.parse_args()))
