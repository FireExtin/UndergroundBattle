import asyncio, json, time, datetime, os
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

OUT = Path('/workspace/deck-collection-ui-20261016')
URL = 'https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site'

def safe_url(value):
    url = urlsplit(value)
    return url.scheme + '://' + url.netloc + url.path

async def main():
    started = time.monotonic()
    result = {'url': URL, 'atUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'viewport': {'width': 1366, 'height': 900}, 'freshContext': True, 'cacheDisabled': True, 'snapshots': [], 'errors': [], 'gamePosts': []}
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', headless=True, proxy={'server': os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')} if (os.environ.get('HTTPS_PROXY') or os.environ.get('https_proxy')) else None, args=['--no-sandbox', '--disable-dev-shm-usage'])
        context = await browser.new_context(viewport=result['viewport'])
        page = await context.new_page()
        session = await context.new_cdp_session(page)
        requests = {}
        session.on('Network.requestWillBeSent', lambda e: requests.setdefault(e['requestId'], {'url': safe_url(e['request']['url']), 'method': e['request']['method'], 'type': e.get('type')}))
        def response(e):
            row = requests.setdefault(e['requestId'], {})
            row.update({'url': safe_url(e['response']['url']), 'status': e['response']['status'], 'mimeType': e['response'].get('mimeType'), 'fromDiskCache': e['response'].get('fromDiskCache', False)})
        session.on('Network.responseReceived', response)
        session.on('Network.loadingFinished', lambda e: requests.setdefault(e['requestId'], {}).update({'encodedBytes': e.get('encodedDataLength', 0)}))
        session.on('Network.loadingFailed', lambda e: requests.setdefault(e['requestId'], {}).update({'failed': e.get('errorText')}))
        page.on('pageerror', lambda e: result['errors'].append(str(e)))
        await session.send('Network.enable')
        await session.send('Network.setCacheDisabled', {'cacheDisabled': True})
        async def snapshot(name):
            dom = await page.evaluate('''() => ({title: document.title, imageElements: document.images.length, cardImageElements: [...document.images].filter(x => x.getAttribute('src')?.startsWith('/cards/')).length, loadedCardImages: [...document.images].filter(x => x.getAttribute('src')?.startsWith('/cards/') && x.complete && x.naturalWidth).length, visibleCardImages: [...document.images].filter(x => {const r=x.getBoundingClientRect();return x.getAttribute('src')?.startsWith('/cards/') && r.bottom>0 && r.top<innerHeight && r.right>0 && r.left<innerWidth}).length, catalogRows:document.querySelectorAll('[data-catalog-card]').length, libraryMounted:!!document.querySelector('.hg-deck-library'), scrollY, viewportHeight:innerHeight, cardImages:[...document.images].filter(x=>x.getAttribute('src')?.startsWith('/cards/')).map(x=>({path:x.getAttribute('src'),loading:x.loading,complete:x.complete,naturalWidth:x.naturalWidth,top:Math.round(x.getBoundingClientRect().top)}))})''')
            rows = list(requests.values())
            card = [r for r in rows if '/cards/' in r.get('url', '')]
            result['snapshots'].append({'name': name, 'elapsedSeconds': round(time.monotonic()-started, 3), 'dom': dom, 'requestsSoFar': len(rows), 'encodedBytesSoFar': sum(r.get('encodedBytes', 0) for r in rows), 'cardImageRequestsSoFar': len(card), 'cardImageEncodedBytesSoFar': sum(r.get('encodedBytes', 0) for r in card), 'cardRequests': card})
            await page.screenshot(path=str(OUT/(name+'.png')), full_page=False)
        try:
            response = await page.goto(URL, wait_until='domcontentloaded', timeout=30000)
            result['mainStatus'] = response.status if response else None
            try: await page.wait_for_load_state('networkidle', timeout=15000)
            except Exception as e: result['idleWait'] = type(e).__name__
            await page.wait_for_timeout(1500)
            await snapshot('public-home-cold-no-scroll')
            if await page.locator('.hg-deck-library').count():
                await page.locator('.hg-deck-library').scroll_into_view_if_needed()
                await page.wait_for_timeout(2000)
                await snapshot('public-home-library-visible')
        except Exception as e:
            result['navigationError'] = type(e).__name__ + ': ' + str(e)
        finally:
            result['requests'] = list(requests.values())
            result['gamePosts'] = [r for r in result['requests'] if r.get('method') == 'POST']
            await context.close()
            await browser.close()
            (OUT/'public-baseline-proxy.json').write_text(json.dumps(result, indent=2, ensure_ascii=False)+'\n')
            print(json.dumps({'mainStatus': result.get('mainStatus'), 'error':result.get('navigationError'), 'snapshots':[{k:v for k,v in s.items() if k not in ['dom','cardRequests']}|{'mountedCardImages':s['dom']['cardImageElements'],'loadedCardImages':s['dom']['loadedCardImages'],'catalogRows':s['dom']['catalogRows'],'visibleCardImages':s['dom']['visibleCardImages']} for s in result['snapshots']], 'pageErrors':result['errors'], 'posts':len(result['gamePosts'])}, ensure_ascii=False), flush=True)

asyncio.run(main())
