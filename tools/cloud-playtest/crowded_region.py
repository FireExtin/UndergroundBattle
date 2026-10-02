#!/usr/bin/env python3
"""Read-only pointer regression on a preserved naturally played desktop room."""
import argparse, asyncio, json
from pathlib import Path
from playwright.async_api import async_playwright
from browser_playtest import OBSERVE_FETCH
from common import IsolatedService, write_json

async def run(args):
    output=Path(args.output).resolve();service=IsolatedService(args.binary,args.cwd,output,args.port,args.static_dir)
    service.start(); result={'passed':False,'stateInjection':False,'apiMoves':False,'gamePosts':0}
    try:
        async with async_playwright() as p:
            context=await p.chromium.launch_persistent_context(str(output/'profile-p0'),executable_path='/usr/bin/chromium',headless=True,viewport={'width':1180,'height':760},args=['--no-sandbox','--disable-dev-shm-usage'])
            try:
                await context.add_init_script(OBSERVE_FETCH)
                page=context.pages[0] if context.pages else await context.new_page()
                async def guard(route):
                    assert route.request.method!='POST','This pointer probe must not write the game'
                    await route.continue_()
                await context.route('**/api/**',guard)
                await page.goto(service.base_url,wait_until='domcontentloaded')
                await page.wait_for_function('window.__cloudView?.status === "playing"')
                identity=await page.evaluate('({roomId:window.__cloudView.roomId,you:window.__cloudView.you,version:window.__cloudView.version})')
                assert identity['roomId']==args.room_id and identity['you']=='p0'
                checks=[]
                cards=page.locator('.hg-region-characters [data-card-instance]')
                for i in range(await cards.count()):
                    tile=cards.nth(i); key=await tile.get_attribute('data-card-instance')
                    await tile.scroll_into_view_if_needed()
                    hit=await tile.evaluate('(e)=>{const r=e.getBoundingClientRect();const h=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest("[data-card-instance]");return {target:e.dataset.cardInstance,hit:h?.dataset.cardInstance,width:r.width,height:r.height}}')
                    assert hit['hit']==key, f'Pointer center covered: {key} by {hit["hit"]}'
                    await tile.click(timeout=3000)
                    await page.locator('.hg-inspector').wait_for(state='visible')
                    assert await tile.get_attribute('aria-pressed')=='true'
                    await page.get_by_role('button',name='关闭卡牌详情',exact=True).click()
                    checks.append(hit)
                result.update(passed=True,identity=identity,pointerChecks=checks,viewport=[1180,760])
            except Exception as error:result['failure']=str(error).split('\n')[0]
            finally:await context.close()
    finally:service.stop()
    write_json(output/args.summary_name,result);print(json.dumps(result,ensure_ascii=False));assert result['passed'],result.get('failure')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True);p.add_argument('--cwd',default='.');p.add_argument('--static-dir',required=True);p.add_argument('--output',required=True);p.add_argument('--port',type=int,required=True);p.add_argument('--room-id',required=True);p.add_argument('--summary-name',default='crowded-region-summary.json')
    asyncio.run(run(p.parse_args()))
