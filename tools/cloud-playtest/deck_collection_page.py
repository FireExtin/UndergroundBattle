import argparse, asyncio, datetime, json, re, time
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

parser = argparse.ArgumentParser(description='Normal-controls local collection UI/network acceptance; never submits a game command.')
parser.add_argument('--url', default='http://127.0.0.1:8090')
parser.add_argument('--output', required=True)
parser.add_argument('--source', required=True)
args = parser.parse_args()
OUT = Path(args.output)
BASE = args.url
OUT.mkdir(exist_ok=False)

def safe_url(url):
    value = urlsplit(url)
    return value.scheme + '://' + value.netloc + value.path

async def main():
    result = {'kind': 'local-production-only', 'source':args.source, 'atUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'viewport': {'width':1366,'height':900}, 'checks': [], 'snapshots': [], 'errors': [], 'posts': []}
    requests = {}
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', headless=True, args=['--no-sandbox','--disable-dev-shm-usage'])
        context = await browser.new_context(viewport=result['viewport'], locale='zh-CN')
        page = await context.new_page()
        session = await context.new_cdp_session(page)
        session.on('Network.requestWillBeSent',lambda e: requests.setdefault(e['requestId'],{'url':safe_url(e['request']['url']),'method':e['request']['method'],'type':e.get('type')}))
        def response(e):
            requests.setdefault(e['requestId'],{}).update({'url':safe_url(e['response']['url']),'status':e['response']['status'],'mimeType':e['response'].get('mimeType')})
        session.on('Network.responseReceived',response)
        session.on('Network.loadingFinished',lambda e: requests.setdefault(e['requestId'],{}).update({'encodedBytes':e.get('encodedDataLength',0)}))
        session.on('Network.loadingFailed',lambda e: requests.setdefault(e['requestId'],{}).update({'failed':e.get('errorText')}))
        page.on('pageerror',lambda e: result['errors'].append(str(e)))
        await session.send('Network.enable')
        await session.send('Network.setCacheDisabled',{'cacheDisabled':True})
        async def snapshot(name):
            dom=await page.evaluate('''() => ({hash:location.hash,cardRows:document.querySelectorAll('[data-catalog-card]').length,cardImageElements:[...document.images].filter(x=>x.getAttribute('src')?.startsWith('/cards/')).length,loadedCardImages:[...document.images].filter(x=>x.getAttribute('src')?.startsWith('/cards/')&&x.complete&&x.naturalWidth).length,deferredFaces:document.querySelectorAll('[data-face-state="deferred"]').length,libraryMounted:!!document.querySelector('.hg-deck-library'),overflow:document.documentElement.scrollWidth>innerWidth,images:[...document.images].filter(x=>x.getAttribute('src')?.startsWith('/cards/')).map(x=>({path:x.getAttribute('src'),top:Math.round(x.getBoundingClientRect().top),bottom:Math.round(x.getBoundingClientRect().bottom)}))})''')
            card=[r for r in requests.values() if '/cards/' in r.get('url','')]
            result['snapshots'].append({'name':name,'dom':dom,'requestsSoFar':len(requests),'encodedBytesSoFar':sum(r.get('encodedBytes',0) for r in requests.values()),'cardRequestsSoFar':len(card),'cardEncodedBytesSoFar':sum(r.get('encodedBytes',0) for r in card),'cardRequests':card})
            await page.screenshot(path=str(OUT/(name+'.png')),full_page=False)
        async def check(name, condition):
            result['checks'].append({'name':name,'passed':bool(condition)})
            assert condition,name
        async def nav(name):
            await page.get_by_role('navigation',name='大厅导航').get_by_role('button',name=name,exact=True).click()
        try:
            await page.goto(BASE,wait_until='networkidle')
            await page.wait_for_timeout(800)
            await snapshot('home-cold')
            await check('homepage-has-no-card-images',result['snapshots'][-1]['dom']['cardImageElements']==0)
            await check('homepage-has-no-card-requests',result['snapshots'][-1]['cardRequestsSoFar']==0)
            await page.get_by_role('button',name=re.compile('卡牌收藏与组牌')).click()
            await page.locator('.hg-library-page').wait_for()
            await page.wait_for_timeout(1600)
            await snapshot('collection-first-visible')
            first=result['snapshots'][-1]
            await check('collection-keeps-full-keyboard-grid',first['dom']['cardRows']==133)
            await check('collection-defers-most-images',first['dom']['deferredFaces']>100)
            await check('collection-bounded-initial-images',0<first['cardRequestsSoFar']<20)
            await check('collection-no-horizontal-overflow',not first['dom']['overflow'])
            add_visible=await page.locator('.hg-library-card-options .hg-library-grid-counter button').nth(1).evaluate('(el)=>{const r=el.getBoundingClientRect(),g=el.closest("ul").getBoundingClientRect();return r.bottom<=Math.min(innerHeight,g.bottom)&&r.top>=Math.max(0,g.top)}')
            await check('initial-row-add-control-visible',add_visible)
            await session.send('Network.setCacheDisabled',{'cacheDisabled':False})
            await check('defaults-to-real-buildable-cards',await page.get_by_role('combobox',name='组牌准入',exact=True).input_value()=='implemented' and await page.locator('[data-catalog-card]:not([data-card-admission="buildable"])').count()==0)
            for faction in ['red','black','blue','gray']:
                await page.get_by_role('combobox',name='派系',exact=True).select_option(faction)
                await check('buildable-markers-'+faction,await page.locator('[data-catalog-card]').count()>0 and await page.locator('[data-catalog-card]:not([data-card-admission="buildable"])').count()==0)
            await page.get_by_role('combobox',name='派系',exact=True).select_option('all')
            await page.get_by_role('combobox',name='组牌准入',exact=True).select_option('all')
            view_only=page.locator('[data-catalog-card][data-card-admission="view-only"]')
            await check('view-only-cards-cannot-be-added',await view_only.count()>0 and await view_only.first.locator('.hg-library-grid-counter button').nth(1).is_disabled())
            await page.get_by_role('combobox',name='组牌准入',exact=True).select_option('implemented')
            buttons=page.locator('.hg-library-card-options .hg-library-card-preview')
            await buttons.nth(0).focus()
            await page.keyboard.press('ArrowRight')
            await check('grid-right-arrow-focus',await buttons.nth(1).evaluate('(el)=>el===document.activeElement'))
            await page.keyboard.press('End')
            await check('grid-end-focus',await buttons.last.evaluate('(el)=>el===document.activeElement'))
            await page.keyboard.press('Home')
            await check('grid-home-focus',await buttons.nth(0).evaluate('(el)=>el===document.activeElement'))
            await buttons.nth(0).click()
            dialog=page.get_by_role('dialog',name=re.compile('卡面预览'))
            await dialog.wait_for()
            await check('detail-original-image',await dialog.get_by_role('img').count()==1)
            await check('detail-explains-card-admission',await dialog.get_by_text('卡牌准入：可组牌',exact=False).count()==1)
            await page.keyboard.press('Escape')
            await check('detail-closes-and-restores-focus',await dialog.count()==0 and await buttons.nth(0).evaluate('(el)=>el===document.activeElement'))
            await page.get_by_role('button',name=re.compile('牌组设置与模板')).click()
            await page.get_by_role('button',name='复制为草稿',exact=True).click()
            await page.get_by_role('textbox',name='牌组名称',exact=True).fill('收藏页本地验证')
            await page.get_by_role('combobox',name='派系',exact=True).select_option('black')
            await page.get_by_role('combobox',name='卡牌类型',exact=True).select_option('character')
            await page.get_by_role('combobox',name='印刷费用',exact=True).select_option('1')
            await page.get_by_role('combobox',name='特性',exact=True).select_option('subtype:人类')
            await page.get_by_role('searchbox',name='检索卡牌',exact=True).fill('WM056')
            await check('real-field-filter-intersection',await page.locator('[data-catalog-card]').count()==1 and await page.locator('[data-catalog-card="WM056"]').count()==1)
            await page.get_by_role('button',name='添加 赃物贩子（WM056）',exact=True).click()
            count=page.get_by_role('spinbutton',name='赃物贩子（WM056）张数',exact=True)
            await count.fill('4')
            await check('copy-limit-shown',await page.locator('.hg-library-issues').filter(has_text='赃物贩子').count()>0)
            await check('invalid-copy-count-cannot-select',await page.get_by_role('button',name='保存并选择此牌组',exact=True).is_disabled())
            await check('admission-separate-from-current-deck-limit',await page.locator('[data-catalog-card="WM056"]').get_attribute('data-card-admission')=='buildable' and '当前牌组：' in await page.locator('[data-catalog-card="WM056"] .hg-library-warning').inner_text())
            await count.fill('3')
            await check('valid-deck-can-select',await page.get_by_role('button',name='保存并选择此牌组',exact=True).is_enabled())
            await page.get_by_role('combobox',name='派系',exact=True).select_option('red')
            await check('filter-keeps-selected-count',await count.input_value()=='3')
            await page.get_by_role('combobox',name='派系',exact=True).select_option('black')
            await page.get_by_role('button',name='保存牌组',exact=True).click()
            await page.reload(wait_until='networkidle')
            await check('reload-returns-to-collection',await page.locator('.hg-library-page').count()==1)
            await page.get_by_role('button',name=re.compile('牌组设置与模板')).click()
            await check('saved-deck-and-count-restore',await page.get_by_role('textbox',name='牌组名称',exact=True).input_value()=='收藏页本地验证' and await page.get_by_role('spinbutton',name='赃物贩子（WM056）张数',exact=True).input_value()=='3')
            await check('filters-restore',await page.get_by_role('combobox',name='特性',exact=True).input_value()=='subtype:人类' and await page.get_by_role('searchbox',name='检索卡牌',exact=True).input_value()=='WM056')
            await page.get_by_role('button',name='保存并选择此牌组',exact=True).click()
            await page.get_by_role('heading',name='准备入席',exact=True).wait_for()
            await check('selected-draft-enters-play-page',await page.get_by_text('当前牌组：收藏页本地验证',exact=False).count()==1)
            await nav('卡牌收藏')
            await page.get_by_role('spinbutton',name='赃物贩子（WM056）张数',exact=True).fill('2')
            await nav('首页')
            await page.wait_for_timeout(600)
            await snapshot('home-after-collection')
            await check('hidden-collection-has-no-images-or-rows',result['snapshots'][-1]['dom']['cardImageElements']==0 and result['snapshots'][-1]['dom']['cardRows']==0)
            await nav('卡牌收藏')
            await check('navigation-keeps-unsaved-work',await page.get_by_role('spinbutton',name='赃物贩子（WM056）张数',exact=True).input_value()=='2')
            await page.reload(wait_until='networkidle')
            await check('reload-keeps-unsaved-work',await page.get_by_role('spinbutton',name='赃物贩子（WM056）张数',exact=True).input_value()=='2')
            await snapshot('collection-restored')
            await nav('入席')
            await page.reload(wait_until='networkidle')
            await check('reload-keeps-explicit-selected-snapshot',await page.get_by_text('当前牌组：收藏页本地验证',exact=False).count()==1)
            snapshot_count=await page.evaluate('JSON.parse(sessionStorage.getItem("hegemony.lobbyDeckSelection.v1")).draft.cards.find(c=>c.cardId==="WM056").count')
            await check('editing-does-not-mutate-selected-snapshot',snapshot_count==3)
            await nav('卡牌收藏')
            await page.go_back()
            await check('browser-back-navigation',await page.get_by_role('heading',name='准备入席',exact=True).count()==1)
            await check('no-browser-errors',not result['errors'])
            await check('no-gameplay-posts',not [r for r in requests.values() if r.get('method')=='POST'])
            result['passed']=True
        except Exception as e:
            result['passed']=False
            result['failure']=type(e).__name__+': '+str(e)
            try: await page.screenshot(path=str(OUT/'failure.png'),full_page=False)
            except Exception: pass
        finally:
            result['posts']=[r for r in requests.values() if r.get('method')=='POST']
            result['requests']=list(requests.values())
            await context.close()
            await browser.close()
            (OUT/'result.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
            print(json.dumps({'passed':result['passed'],'failure':result.get('failure'),'checks':result['checks'],'snapshots':[{k:v for k,v in s.items() if k not in ['dom','cardRequests']}|{'imageElements':s['dom']['cardImageElements'],'deferredFaces':s['dom']['deferredFaces'],'cardRows':s['dom']['cardRows']} for s in result['snapshots']]},ensure_ascii=False),flush=True)

    return 0 if result['passed'] else 1

raise SystemExit(asyncio.run(main()))
