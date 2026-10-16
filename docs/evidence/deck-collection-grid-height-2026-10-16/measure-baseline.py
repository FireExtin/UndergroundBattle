import asyncio
import datetime
import hashlib
import json
from pathlib import Path
from playwright.async_api import async_playwright

OUT = Path('/workspace/collection-layout-20261016/baseline')
METRICS = '''() => {
  const box = el => { const r=el.getBoundingClientRect(); return {x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom}; };
  const grid=document.querySelector('.hg-library-card-options');
  const cards=[...grid.children];
  const tops=[...new Set(cards.map(x=>Math.round(x.getBoundingClientRect().top)))];
  const first=cards[0], face=first.querySelector('.hg-library-face'), badge=first.querySelector('.hg-library-card-badge');
  return { viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio},page:box(document.querySelector('main')),catalog:box(grid.parentElement),grid:box(grid),gridClientHeight:grid.clientHeight,gridScrollHeight:grid.scrollHeight,rowPitch:tops.length>1?tops[1]-tops[0]:null,firstCard:box(first),face:box(face),badge:box(badge),add:box(first.querySelector('.hg-library-grid-counter button:last-child')),image:box(face.querySelector('img')),loadedImages:grid.querySelectorAll('img').length,deferred:grid.querySelectorAll('[data-face-state="deferred"]').length,overflowX:document.documentElement.scrollWidth>innerWidth};
}'''

async def main():
    results=[]
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage'])
        for width,height,scale in [(2048,1122,1),(1638,898,1.25),(1366,900,1),(1366,768,1)]:
            context=await browser.new_context(viewport={'width':width,'height':height},device_scale_factor=scale,locale='zh-CN')
            page=await context.new_page()
            await page.goto('http://127.0.0.1:8090',wait_until='networkidle')
            await page.get_by_role('button',name='卡牌收藏与组牌',exact=False).click()
            await page.get_by_role('combobox',name='派系',exact=True).select_option('black')
            await page.get_by_role('button',name='舒展',exact=True).click()
            await page.wait_for_timeout(500)
            result=await page.evaluate(METRICS)
            name=f'{width}x{height}-dpr{scale}'
            await page.screenshot(path=str(OUT/(name+'.png')))
            result['screenshot']=name+'.png';results.append(result)
            await context.close()
        await browser.close()
    (OUT/'measurements.json').write_text(json.dumps({'source':'736fbc3fb5afe0f34da6e467f1718e221249de4f','kind':'local-production-baseline','results':results},indent=2)+'\n')
    for r in results:
        print(json.dumps(r),flush=True)

asyncio.run(main())
