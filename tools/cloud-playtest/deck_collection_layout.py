"""PC production-build layout acceptance through normal browser controls only."""
import argparse
import asyncio
import datetime
import json
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url', default='http://127.0.0.1:8090')
parser.add_argument('--source', required=True)
parser.add_argument('--output', required=True)
args = parser.parse_args()
OUT = Path(args.output)
OUT.mkdir(exist_ok=False)

METRICS = '''() => {
  const box=el=>{const r=el.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom,right:r.right}};
  const grid=document.querySelector('.hg-library-card-options'), cards=[...grid.children], style=getComputedStyle(grid);
  const tops=[...new Set(cards.map(c=>Math.round(c.getBoundingClientRect().top)))];
  const first=cards.filter(c=>Math.abs(c.getBoundingClientRect().top-cards[0].getBoundingClientRect().top)<1);
  const gap=parseFloat(style.rowGap)||0,padding=(parseFloat(style.paddingTop)||0)+(parseFloat(style.paddingBottom)||0);
  const rowHeight=Math.max(...first.map(c=>c.getBoundingClientRect().height));
  return {viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio},grid:box(grid),gridClientHeight:grid.clientHeight,gridScrollHeight:grid.scrollHeight,
    firstRowHeight:rowHeight,rowGap:gap,rowCapacity:(grid.clientHeight-padding+gap)/(rowHeight+gap),rowTops:tops.slice(0,4),
    count:cards.length,loadedImages:grid.querySelectorAll('img').length,deferred:grid.querySelectorAll('[data-face-state="deferred"]').length,
    overflowX:document.documentElement.scrollWidth>innerWidth,firstRow:first.map(card=>({id:card.dataset.catalogCard,card:box(card),
      face:box(card.querySelector('.hg-library-face')),badge:box(card.querySelector('.hg-library-card-badge')),add:box(card.querySelector('.hg-library-grid-counter button:last-child'))}))};
}'''

VISIBLE_FACES = '''() => {
  const grid=document.querySelector('.hg-library-card-options'), g=grid.getBoundingClientRect(), cards=[...grid.children];
  const first=cards.filter(c=>Math.abs(c.getBoundingClientRect().top-cards[0].getBoundingClientRect().top)<1);
  const header=document.querySelector('.hg-header').getBoundingClientRect();
  return first.map(card=>{
    const img=card.querySelector('.hg-library-face img'),face=card.querySelector('.hg-library-face').getBoundingClientRect(),badge=card.querySelector('.hg-library-card-badge').getBoundingClientRect();
    if(!img||!img.naturalWidth)return {id:card.dataset.catalogCard,loaded:false};
    const r=img.getBoundingClientRect(),scale=Math.min(r.width/img.naturalWidth,r.height/img.naturalHeight),w=img.naturalWidth*scale,h=img.naturalHeight*scale;
    const content={x:r.x+(r.width-w)/2,y:r.y+(r.height-h)/2,width:w,height:h};
    const points=[.08,.5,.92].map(x=>({x:content.x+w*x,y:content.y+h*.94}));
    return {id:card.dataset.catalogCard,loaded:true,objectFit:getComputedStyle(img).objectFit,
      fullFaceInsideGrid:face.top>=g.top-.5&&face.bottom<=g.bottom+.5,
      fullFaceInsideViewport:face.top>=header.bottom-.5&&face.bottom<=innerHeight+.5,
      badgeBelowFace:badge.top>=face.bottom-.5,bottomHitTests:points.map(p=>document.elementFromPoint(p.x,p.y)===img)};
  });
}'''


async def run_viewport(browser, width, height, scale):
    result={'viewport':{'width':width,'height':height,'deviceScaleFactor':scale},'checks':[],'metrics':{},'screenshots':[],'pageErrors':[],'posts':[]}
    name=f'{width}x{height}-dpr{scale}'
    context=await browser.new_context(viewport={'width':width,'height':height},device_scale_factor=scale,locale='zh-CN')
    page=await context.new_page()
    page.on('pageerror',lambda e:result['pageErrors'].append(str(e)))
    page.on('request',lambda r:result['posts'].append(urlsplit(r.url).path) if r.method=='POST' else None)

    def check(label, value):
        result['checks'].append({'name':label,'passed':bool(value)})
        assert value,label

    async def snapshot(label):
        filename=name+'-'+label+'.png'
        await page.screenshot(path=str(OUT/filename),full_page=False)
        result['screenshots'].append(filename)

    async def measure(label):
        await page.wait_for_timeout(150)
        value=await page.evaluate(METRICS);result['metrics'][label]=value
        check(label+'-at-least-2.5-card-rows',value['rowCapacity']>=2.499)
        check(label+'-no-horizontal-overflow',not value['overflowX'])
        check(label+'-badges-outside-image',all(c['badge']['y']>=c['face']['bottom']-.5 for c in value['firstRow']))
        return value

    try:
        response=await page.goto(args.url,wait_until='networkidle')
        check('production-root-200',response.status==200)
        check('home-has-no-card-images',await page.locator('img[src^="/cards/"]').count()==0)
        await page.get_by_role('button',name='卡牌收藏与组牌',exact=False).click()
        await page.locator('[data-catalog-card]').first.wait_for()
        compact=await measure('default-compact')
        check('catalog-keeps-all-133-buildable-rows',compact['count']==133)
        check('far-card-images-remain-deferred',compact['deferred']>80 and compact['loadedImages']<40)
        await page.get_by_role('combobox',name='派系',exact=True).select_option('black')
        await page.get_by_role('button',name='舒展',exact=True).click()
        comfortable=await measure('black-comfortable')
        check('black-filter-keeps-25-real-cards',comfortable['count']==25)
        await snapshot('entry')
        grid=page.locator('.hg-library-card-options')
        await grid.evaluate('(el)=>{const header=document.querySelector(".hg-header").getBoundingClientRect();window.scrollBy(0,el.getBoundingClientRect().top-header.bottom-16)}')
        await page.wait_for_function('''() => {const g=document.querySelector('.hg-library-card-options');const first=g.children[0].getBoundingClientRect().top;return [...g.children].filter(c=>Math.abs(c.getBoundingClientRect().top-first)<1).every(c=>{const i=c.querySelector('img');return i&&i.complete&&i.naturalWidth>0})}''')
        faces=await page.evaluate(VISIBLE_FACES);result['firstRowFaces']=faces
        check('whole-first-row-images-visible',all(f['loaded'] and f['fullFaceInsideGrid'] and f['fullFaceInsideViewport'] for f in faces))
        check('full-images-use-contain',all(f['objectFit']=='contain' for f in faces))
        check('lower-left-defense-and-card-bottom-uncovered',all(f['badgeBelowFace'] and all(f['bottomHitTests']) for f in faces))
        await snapshot('whole-first-row')
        first=page.locator('[data-catalog-card]').first
        first_id=await first.get_attribute('data-catalog-card')
        add=first.locator('.hg-library-grid-counter button').last
        await add.dblclick(delay=50)
        check('two-real-add-clicks-produce-two-copies',await first.get_attribute('data-selected-count')=='2')
        for _ in range(3):
            await first.locator('.hg-library-card-preview').click()
            await page.get_by_role('dialog',name='卡面预览',exact=False).wait_for()
            check('one-preview-dialog-after-repeated-open',await page.get_by_role('dialog').count()==1)
            await page.keyboard.press('Escape')
        await page.get_by_role('button',name='牌组设置与模板',exact=False).click()
        await page.get_by_role('textbox',name='牌组名称',exact=True).fill('收藏网格高度验证')
        await page.get_by_role('button',name='保存牌组',exact=True).click()
        await page.reload(wait_until='networkidle')
        await page.get_by_role('button',name='牌组设置与模板',exact=False).click()
        check('saved-name-restores',await page.get_by_role('textbox',name='牌组名称',exact=True).input_value()=='收藏网格高度验证')
        check('saved-two-copies-restore',await page.locator(f'[data-catalog-card="{first_id}"]').get_attribute('data-selected-count')=='2')
        check('faction-and-density-restore',await page.get_by_role('combobox',name='派系',exact=True).input_value()=='black' and await page.get_by_role('button',name='舒展',exact=True).get_attribute('aria-pressed')=='true')
        row=page.locator(f'[data-catalog-card="{first_id}"]')
        await row.locator('.hg-library-grid-counter button').last.click()
        await page.reload(wait_until='networkidle')
        check('unsaved-third-copy-restores',await page.locator(f'[data-catalog-card="{first_id}"]').get_attribute('data-selected-count')=='3')
        await measure('after-reload')
        await grid.evaluate('(el)=>{const header=document.querySelector(".hg-header").getBoundingClientRect();window.scrollBy(0,el.getBoundingClientRect().top-header.bottom-16)}')
        await snapshot('restored')
        await page.get_by_role('button',name='紧凑',exact=True).click()
        await measure('switched-back-compact')
        last=page.locator('[data-catalog-card]').last.locator('.hg-library-card-preview')
        await page.locator('[data-catalog-card]').first.locator('.hg-library-card-preview').focus()
        await page.keyboard.press('End')
        check('keyboard-end-reaches-last-card',await last.evaluate('(el)=>document.activeElement===el'))
        await page.keyboard.press('Home')
        check('keyboard-home-returns-to-first-card',await page.locator('[data-catalog-card]').first.locator('.hg-library-card-preview').evaluate('(el)=>document.activeElement===el'))
        await page.get_by_role('navigation',name='大厅导航').get_by_role('button',name='首页',exact=True).click()
        check('leaving-collection-unmounts-card-grid',await page.locator('[data-catalog-card]').count()==0)
        check('no-game-posts',not result['posts'])
        check('no-page-errors',not result['pageErrors'])
        result['passed']=True
    except Exception as e:
        result.update(passed=False,failure=type(e).__name__+': '+str(e))
        try:await snapshot('failure')
        except Exception:pass
    finally:
        await context.close()
    return result


async def main():
    report={'source':args.source,'kind':'local-production-browser-layout','atUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'results':[]}
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage'])
        for width,height,scale in [(2048,1122,1),(1638,898,1.25),(1366,900,1),(1366,768,1)]:
            result=await run_viewport(browser,width,height,scale);report['results'].append(result)
            (OUT/'result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
            print(json.dumps({'viewport':result['viewport'],'passed':result['passed'],'checks':len(result['checks']),'failure':result.get('failure')},ensure_ascii=False),flush=True)
            if not result['passed']:break
        await browser.close()
    report['passed']=len(report['results'])==4 and all(r['passed'] for r in report['results'])
    (OUT/'result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    return 0 if report['passed'] else 1


raise SystemExit(asyncio.run(main()))
