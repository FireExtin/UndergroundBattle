#!/usr/bin/env python3
"""Natural persistent-browser seat re-entry regression; no API game writes."""
import argparse, asyncio, json
from pathlib import Path
from playwright.async_api import async_playwright
from attachment_v026 import AttachmentRun
from common import IsolatedService, write_json

async def run(args):
    output=Path(args.output).resolve()
    service=IsolatedService(args.binary,args.cwd,output,args.port,args.static_dir)
    service.start()
    result={'passed':False,'stateInjection':False,'apiMoves':False}
    try:
        async with async_playwright() as p:
            qa=AttachmentRun(p,service,output)
            try:
                page=await qa.new_seat(0)
                await page.get_by_label('你的称呼').fill('重返原席回归')
                await page.get_by_role('button',name='创建牌桌 →',exact=True).click()
                await page.wait_for_function('window.__cloudView?.status === "lobby"')
                before=await qa.view(0); code=before['inviteCode']
                action=next(a for a in before['legalActions'] if a['kind']=='ready')
                await qa.legal_button(0,action)
                before=await qa.view(0); player=next(p for p in before['players'] if p['id']==before['you'])
                for iteration in range(2):
                    await page.get_by_role('button',name='返回大厅 / 新建牌桌',exact=True).click()
                    await page.get_by_role('button',name='邀请码加入',exact=True).click()
                    await page.get_by_label('你的称呼').fill('重返原席回归' if not iteration else '新的称呼不应替换原席')
                    await page.get_by_label('邀请码',exact=True).fill(code)
                    await page.get_by_role('button',name='加入牌桌 →',exact=True).click()
                    await page.locator('.hg-room-controls').wait_for()
                    after=await qa.view(0)
                    assert after['you']==before['you'], 'Returning through the same invite consumed another seat'
                    assert len(after['players'])==1 and after['version']==before['version']
                    assert next(p for p in after['players'] if p['id']==after['you'])==player
                await page.reload(wait_until='domcontentloaded')
                await page.locator('.hg-room-controls').wait_for()
                restored=await qa.view(0)
                assert restored['you']==before['you'] and restored['version']==before['version']
                await page.get_by_role('button',name='上手指南 ?').click()
                await page.get_by_role('dialog',name='上手指南').wait_for()
                await page.keyboard.press('Escape')
                await page.get_by_role('dialog',name='上手指南').wait_for(state='hidden')
                assert not qa.errors
                assert qa.post_count==2, 'Re-entry must issue no create/join/ready mutation'
                history=await page.evaluate('JSON.parse(localStorage.getItem("hegemony.seats.v1") || "[]").map(s=>({roomId:s.roomId,seat:s.seat}))')
                result.update(passed=True,roomId=before['roomId'],reentries=2,occupants=1,seat=0,version=restored['version'],ready=player['ready'],deckId=player['deckId'],history=history,guideEscape=True)
            except Exception as error:
                result['failure']=f'{type(error).__name__}: {error}'
            finally:
                result.update(uiPostCount=qa.post_count,postResults=qa.post_results,browserErrors=qa.errors)
                write_json(output/'seat-return-summary.json',result)
                for c in qa.contexts:await c.close()
    finally:service.stop()
    print(json.dumps(result,ensure_ascii=False))
    assert result['passed'],result.get('failure')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',required=True);p.add_argument('--cwd',default='.');p.add_argument('--static-dir',required=True)
    p.add_argument('--output',required=True);p.add_argument('--port',type=int,default=8111)
    asyncio.run(run(p.parse_args()))
