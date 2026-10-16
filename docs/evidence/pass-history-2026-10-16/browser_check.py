"""Two fresh local browsers against the real ABI77 production Worker/D1 and built UI.
Only newly created local test rooms; no production access or opaque-state mutation.
Session tokens stay in process/browser memory and are never written to evidence.
"""
import asyncio, datetime, hashlib, json, uuid, traceback, re
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright

OUT=Path('/workspace/pass-context-evidence-20261016')
ORIGIN='http://127.0.0.1:8129'
ASSET_ROOT=Path('/workspace/pass-context-abi77-review-20261016/sites/dist/client/assets')
ASSETS={'/assets/'+file.name:hashlib.sha256(file.read_bytes()).hexdigest() for file in ASSET_ROOT.iterdir() if file.suffix in ['.js','.css']}
def stamp():return datetime.datetime.now(datetime.timezone.utc).isoformat()

async def main():
    result={'scope':'local-production-bundle-only','startedAtUtc':stamp(),'publicSiteVisited':False,
        'userTablesAccessed':False,'freshBrowserContexts':2,'ignoreHttpsErrors':False,'bypassUsed':False,
        'wasmSha256':'b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c',
        'featureSourceCommit':'8a27c1ca9cfc065cc03a543d73f14b140ac93515',
        'checks':{},'browserErrors':[],'commandsFromUi':[],'assetIdentityChecks':[]}
    asset_tasks=[]
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage'])
        result['browserVersion']=browser.version
        api=await p.request.new_context(base_url=ORIGIN)
        sessions=[];contexts=[];pages=[]
        try:
            async def request(route,body=None,seat=None):
                headers={'Authorization':'Bearer '+sessions[seat]['token']} if seat is not None else {}
                response=await (api.post('/api/'+route,data=body,headers=headers) if body is not None else api.get('/api/'+route,headers=headers))
                if response.status!=200:raise AssertionError('Local API status '+str(response.status)+' for '+route.split('/')[-1])
                return await response.json()
            catalog=await request('catalog')
            assert catalog['engineVersion']=='rust-v0.2.77-jc046-mandatory-sacrifice-candidate' and len(catalog['cards'])==143
            draft={'id':'pass-browser','name':'本地让过检查','description':'','societyId':None,
                'cards':[{'cardId':'JC125','count':50}],'updatedAt':'',
                **{key:catalog[key] for key in ['rulesVersion','cardPoolVersion','engineVersion']}}
            nickname='同名玩家'*10 # The public 40-character limit, identical at both seats.
            host=await request('rooms',{'name':nickname,'mode':'duel','deckDraft':draft,'requestId':str(uuid.uuid4())})
            guest=await request('rooms/join',{'name':nickname,'inviteCode':host['inviteCode'],'deckDraft':draft,'requestId':str(uuid.uuid4())})
            sessions=[{key:session[key] for key in ['roomId','inviteCode','token','seat']} for session in sorted([host,guest],key=lambda session:session['seat'])]
            room=sessions[0]['roomId']
            async def view(seat=0):return await request('rooms/'+room+'/state',seat=seat)
            async def submit(seat,action):
                before=await view(seat)
                if action['kind'] not in ['game','beginResponse','passResponse','cancelAndPass','submitResponse','pauseRoom','resumeRoom']:action={'kind':'game','action':action}
                return await request('rooms/'+room+'/commands',{'commandId':str(uuid.uuid4()),'expectedVersion':before['version'],'action':action},seat)
            for seat in [0,1]:await submit(seat,{'kind':'ready'})
            await submit(0,{'kind':'start'})
            for guard in range(8):
                states=[await view(seat) for seat in [0,1]]
                chooser=next(((seat,state['pendingChoice']) for seat,state in enumerate(states) if state.get('pendingChoice')),None)
                if not chooser:break
                seat,choice=chooser
                assert choice['kind']=='mulligan'
                await submit(seat,{'kind':'choose','choiceId':choice['id'],'selected':[]})
            assert (await view())['step']=='prepare'
            for seat in [0,1]:
                context=await browser.new_context(viewport={'width':1440,'height':1000},locale='zh-CN',ignore_https_errors=False)
                await context.add_init_script('const s='+json.dumps(sessions[seat])+'; localStorage.setItem("hegemony.session.v1",JSON.stringify(s));localStorage.setItem("hegemony.seats.v1",JSON.stringify([s]));localStorage.setItem("hegemony.screen.v1","table");localStorage.setItem("hegemony.autoPass.v1","false");')
                page=await context.new_page()
                page.on('pageerror',lambda error:result['browserErrors'].append(str(error)))
                async def asset(response):
                    path=urlsplit(response.url).path
                    if path not in ASSETS:return
                    digest=hashlib.sha256(await response.body()).hexdigest()
                    result['assetIdentityChecks'].append({'path':path,'sha256':digest,'expectedSha256':ASSETS[path],'matches':digest==ASSETS[path]})
                page.on('response',lambda response:asset_tasks.append(asyncio.create_task(asset(response))))
                def observed(req,seat=seat):
                    if req.method=='POST' and req.url.endswith('/commands'):
                        body=req.post_data_json;action=body['action'];result['commandsFromUi'].append({'seat':seat,'kind':action['kind'],'gameKind':action.get('action',{}).get('kind')})
                page.on('request',observed)
                await page.goto(ORIGIN,wait_until='networkidle')
                await page.get_by_role('switch',name='无可用行动时自动让过').wait_for()
                contexts.append(context);pages.append(page)
            async def sync():
                for page in pages:
                    await page.reload(wait_until='domcontentloaded')
                    await page.get_by_role('switch',name='无可用行动时自动让过').wait_for()
            async def can_pass():
                for seat in [0,1]:
                    state=await view(seat)
                    if any(action['kind']=='pass' for action in state['legalActions']):return seat,state
                raise AssertionError('No ordinary pass')
            async def both_notice(fragment):
                for guard in range(100):
                    if all([await page.get_by_label('最近让过').count() for page in pages]):
                        texts=[await page.get_by_label('最近让过').inner_text() for page in pages]
                        if texts[0]==texts[1] and fragment in texts[0]:return texts[0]
                    await asyncio.sleep(.1)
                raise AssertionError('Both polling clients did not converge on '+fragment)
            seat,before=await can_pass();expected_actor=before['players'][seat]['name']+f'（席位 {seat+1}）'
            await pages[seat].get_by_role('button',name='让过',exact=True).evaluate('(button)=>{button.click();button.click()}')
            text=await both_notice(expected_actor+' · 第 1 回合 · 开始阶段 · 重置与准备 · 全局')
            assert len(result['commandsFromUi'])==1
            result['checks']['manualRepeatClickOneCommand']=True
            result['checks']['bothClientsGlobalPass']=text
            await sync();seat,before=await can_pass()
            await pages[seat].get_by_role('button',name='让过',exact=True).click()
            for guard in range(20):
                if (await view())['step']=='draw':break
                await asyncio.sleep(.1)
            assert (await view())['step']=='draw'
            await sync()
            result['checks']['globalHistoryAfterPhaseChange']=await both_notice('开始阶段 · 重置与准备 · 全局')
            for page in pages:await page.get_by_role('switch',name='无可用行动时自动让过').check()
            for guard in range(80):
                if (await view())['phase']=='action':break
                await asyncio.sleep(.1)
            assert (await view())['phase']=='action'
            result['checks']['automaticEmptyPassBothClients']=await both_notice('开始阶段 · 抓牌 · 全局')
            for page in pages:await page.get_by_role('switch',name='无可用行动时自动让过').uncheck()
            current=None
            for seat in [0,1]:
                own=await view(seat)
                play=next((action for action in own['legalActions'] if action['kind']=='deploy' and action.get('region')==0),None)
                if play:current=(seat,play);break
            assert current
            seat,play=current
            await submit(seat,{key:value for key,value in play.items() if key not in ['id','label','description']})
            state=await view();member=next(member for member in state['responseWindow']['members'] if member['status']=='undecided')
            responder=next(player['seat'] for player in state['players'] if player['id']==member['playerId'])
            await pages[responder].reload(wait_until='domcontentloaded')
            await pages[responder].get_by_role('button',name='不连锁，让过').click(timeout=4000)
            result['checks']['manualResponseGlobalContext']=await both_notice('行动阶段 · 团队')
            assert '全局 · 响应让过' in result['checks']['manualResponseGlobalContext']
            for page in pages:await page.get_by_label('最近让过').filter(has_text='响应超时，自动让过').wait_for(timeout=12000)
            result['checks']['serverTimeoutBothClients']=await both_notice('响应超时，自动让过')
            # Advance through real authenticated commands to a regional window.
            for guard in range(100):
                state=await view()
                if state['step']=='region:0:combat:before' and not state.get('responseWindow') and not state.get('pendingChoice') and not state.get('waitingChoice'):break
                for seat in [0,1]:
                    own=await view(seat)
                    if own.get('pendingChoice'):
                        choice=own['pendingChoice'];await submit(seat,{'kind':'choose','choiceId':choice['id'],'selected':[option['id'] for option in choice['options'][:choice['min']]]});break
                    window=own.get('responseWindow')
                    if window and any(member['playerId']==own['you'] and member['status']=='undecided' for member in window['members']):
                        await submit(seat,{'kind':'passResponse','windowId':window['id']});break
                    if not window and any(action['kind']=='pass' for action in own['legalActions']):await submit(seat,{'kind':'pass'});break
                else:raise AssertionError('No natural continuation')
            assert (await view())['step']=='region:0:combat:before'
            await sync();seat,before=await can_pass();region=next(region for region in before['regions'] if region['index']==0)
            await pages[seat].get_by_role('button',name='让过',exact=True).click()
            result['checks']['regionalBothClients']=await both_notice('战斗 · 奖励前快速窗口 · 地区 1 · '+region['name'])
            await sync();seat,before=await can_pass()
            await pages[seat].get_by_role('button',name='让过',exact=True).click()
            await sync()
            assert (await view())['step']=='region:0:combat:after'
            result['checks']['regionalHistoryAfterStepChangeAndBothRefresh']=await both_notice('战斗 · 奖励前快速窗口 · 地区 1 · '+region['name'])
            histories=[(await view(seat)).get('passHistory') for seat in [0,1]]
            assert histories[0]==histories[1]
            assert len({record['id'] for record in histories[0]})==len(histories[0])
            result['checks']['recordIdsUniqueAfterRefresh']=True
            result['checks']['sameFortyCharacterNameSeatsDistinguished']=True
            result['publicRecordCount']=len(histories[0])
            result['visualChecks']=[]
            for viewport in [{'width':1440,'height':1000},{'width':1366,'height':900}]:
                for index,page in enumerate(pages):
                    await page.set_viewport_size(viewport)
                    await page.evaluate('()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))')
                    metrics=await page.evaluate('''()=>{
                      const notice=document.querySelector('.hg-latest-pass'),status=notice.parentElement;
                      const round=status.querySelector('.hg-round'),opponent=document.querySelector('.hg-opponent-edge');
                      const box=element=>{const b=element.getBoundingClientRect();return {left:b.left,right:b.right,top:b.top,bottom:b.bottom,width:b.width,height:b.height}};
                      const item=document.querySelector('li[data-pass-record]'),records=document.querySelector('.hg-table-records');
                      return {notice:box(notice),status:box(status),round:box(round),opponent:box(opponent),
                        scrollWidth:notice.scrollWidth,clientWidth:notice.clientWidth,recordColor:getComputedStyle(item).color,
                        recordBackground:getComputedStyle(records).backgroundColor};
                    }''')
                    assert metrics['notice']['left']>=metrics['status']['left']-1
                    assert metrics['notice']['right']<=metrics['status']['right']+1
                    assert metrics['notice']['top']>=metrics['round']['bottom']-1
                    assert metrics['notice']['bottom']<=metrics['opponent']['top']+1
                    assert metrics['scrollWidth']<=metrics['clientWidth']+1
                    await page.get_by_text('记录 ·',exact=False).first.click()
                    palette=await page.evaluate('''()=>{
                      const records=document.querySelector('.hg-table-records'),item=records.querySelector('li[data-pass-record]');
                      return {recordsOpen:records.open,recordColor:getComputedStyle(item).color,recordBackground:getComputedStyle(records).backgroundColor};
                    }''')
                    assert palette['recordsOpen']
                    metrics.update(palette)
                    await page.get_by_text('记录 ·',exact=False).first.click()
                    def luminance(values):
                        channels=[value/255 for value in values[:3]]
                        channels=[value/12.92 if value<=.04045 else ((value+.055)/1.055)**2.4 for value in channels]
                        return sum(channel*weight for channel,weight in zip(channels,[.2126,.7152,.0722]))
                    fg=[float(value) for value in re.findall(r'[\d.]+',metrics['recordColor'])]
                    bg=[float(value) for value in re.findall(r'[\d.]+',metrics['recordBackground'])]
                    alpha=bg[3] if len(bg)>3 else 1
                    worst_background=[channel*alpha+255*(1-alpha) for channel in bg[:3]]
                    a,b=luminance(fg),luminance(worst_background)
                    contrast=(max(a,b)+.05)/(min(a,b)+.05)
                    assert contrast>=4.5
                    result['visualChecks'].append({'seat':index,'viewport':viewport,**metrics,'minimumContrastOnWhiteBackdrop':contrast,'passed':True})
            for index,page in enumerate(pages):
                await page.get_by_text('记录 ·',exact=False).first.click()
                await page.screenshot(path=str(OUT/('browser-host-records.png' if index==0 else 'browser-peer-records.png')),full_page=True)
                await page.get_by_text('记录 ·',exact=False).first.click()
                await page.screenshot(path=str(OUT/('browser-host.png' if index==0 else 'browser-peer.png')),full_page=True)
            assert not result['browserErrors']
            await asyncio.gather(*asset_tasks)
            assert result['assetIdentityChecks'] and all(item['matches'] for item in result['assetIdentityChecks'])
            assert set(ASSETS)=={item['path'] for item in result['assetIdentityChecks']}
            result['outcome']='passed'
        except Exception as error:
            result['outcome']='failed';result['error']=type(error).__name__+': '+str(error).split('\n')[0]
            result['failureLine']=traceback.extract_tb(error.__traceback__)[-1].lineno
            for index,page in enumerate(pages):
                try:await page.screenshot(path=str(OUT/f'browser-failure-{index}.png'),full_page=True)
                except Exception:pass
        finally:
            for context in contexts:await context.close()
            await api.dispose();await browser.close()
    result['finishedAtUtc']=stamp();(OUT/'browser-result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(result,ensure_ascii=False,indent=2))
    if result['outcome']!='passed':raise SystemExit(1)
asyncio.run(main())
