"""One normal-TLS navigation, with no old table/session data and no recovery bypass."""
import asyncio, datetime, hashlib, json, re
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

OUT=Path('/workspace/bq035-publication-20261016')
URL='https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site'
EXPECTED=json.loads(Path('/workspace/bq035-publication-20261016/runtime-manifest.json').read_text())
ASSETS={k.removeprefix('client'):v['sha256'] for k,v in EXPECTED.items() if k.startswith('client/assets/')}
ASSETS['/']=EXPECTED['client/index.html']['sha256']
def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()

async def main():
    result={'startedAtUtc':now(),'url':URL,'expectedDeployedVersion':60,
        'expectedProviderSource':'6d1a1fed34b5abad208488f09055554269e3df10',
        'expectedDeploymentId':'appgdep_6ad24d37555c819187d15d9b5cb1c21a',
        'normalTls':True,'ignoreHttpsErrors':False,'bypassUsed':False,
        'freshBrowserContext':True,'oldInviteUsed':False,'realUserTableAccessed':False,
        'roomCreated':False,'roomJoined':False,'posts':0,'navigationAttempts':0,
        'uiReached':False,'versionIdentityReached':False,'fullGameCount':0,'browserErrors':[],'requestsFailed':[],
        'assetIdentityChecks':[],'consoleErrors':[],
        'gameplayChecks':{name:'not-executed' for name in ['red-black-blue-gray-decks','card-selection','repeat-clicks','refresh-recovery','continuous-pass-phase-turn-boundaries','mandatory-choice','complete-game']}}
    tasks=[]
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage'])
        result['browserVersion']=browser.version
        context=await browser.new_context(viewport={'width':1366,'height':900},locale='zh-CN',ignore_https_errors=False)
        page=await context.new_page()
        def requested(r):
            if r.method=='POST' and urlsplit(r.url).hostname==urlsplit(URL).hostname:result['posts']+=1
        def failed(r):
            result['requestsFailed'].append({'url':r.url,'method':r.method,'error':r.failure})
        async def response(r):
            path=urlsplit(r.url).path
            if path not in ASSETS or urlsplit(r.url).hostname!=urlsplit(URL).hostname:return
            try:
                b=await r.body();h=hashlib.sha256(b).hexdigest()
                result['assetIdentityChecks'].append({'path':path,'status':r.status,'sha256':h,'expectedSha256':ASSETS[path],'matches':h==ASSETS[path]})
            except Exception as e:result['browserErrors'].append('asset-read: '+str(e))
        page.on('request',requested);page.on('requestfailed',failed)
        page.on('response',lambda r:tasks.append(asyncio.create_task(response(r))))
        page.on('pageerror',lambda e:result['browserErrors'].append(str(e)))
        page.on('console',lambda msg:result['consoleErrors'].append(msg.text) if msg.type=='error' else None)
        try:
            result['navigationAtUtc']=now();result['navigationAttempts']+=1
            r=await page.goto(URL,wait_until='domcontentloaded',timeout=45000)
            result['navigationStatus']=r.status if r else None
            await page.locator('.hg-app').wait_for(state='visible',timeout=15000)
            result['uiReached']=True
            result['title']=await page.title()
            await page.screenshot(path=str(OUT/'normal-browser.png'),full_page=True)
            result['outcome']='reachable-awaiting-isolated-gameplay'
        except Exception as e:
            result['errorAtUtc']=now();result['error']=type(e).__name__+': '+str(e)
            result['outcome']='blocked-before-gameplay'
            # Captures the existing failed navigation; never navigates or clicks an interstitial.
            try:
                result['pageUrlAfterFailure']=page.url;result['titleAfterFailure']=await page.title()
                await page.screenshot(path=str(OUT/'normal-browser.png'),full_page=True,timeout=10000)
            except Exception as capture:result['screenshotError']=str(capture)
        finally:
            if tasks:await asyncio.gather(*tasks,return_exceptions=True)
            result['versionIdentityReached']=result['uiReached'] and set(x['path'] for x in result['assetIdentityChecks'] if x['matches'])==set(ASSETS)
            await context.close();await browser.close()
    result['finishedAtUtc']=now()
    (OUT/'normal-browser.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(result,ensure_ascii=False,indent=2))

asyncio.run(main())
