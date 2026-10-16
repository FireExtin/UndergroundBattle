"""Local-only, real compiled Worker/D1 acceptance for pass-until-phase controls."""
import argparse
import asyncio
import datetime
import json
import re
from pathlib import Path
from urllib.parse import urlsplit
from playwright.async_api import async_playwright, expect

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url',default='http://127.0.0.1:8113')
parser.add_argument('--source',required=True)
parser.add_argument('--output',required=True)
args=parser.parse_args()
assert urlsplit(args.url).hostname in {'localhost','127.0.0.1'}, 'Never run against real player tables'
out=Path(args.output);out.mkdir(exist_ok=False)


async def main():
    result={'source':args.source,'atUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'kind':'isolated-production-worker-d1-browser','checks':[],'commands':[],'pageErrors':[],
            'realUserTableAccessed':False,'remoteDeployment':False}
    views={'host':[],'guest':[]}
    def check(name,value):
        result['checks'].append({'name':name,'passed':bool(value)});assert value,name
    def latest(label):return max(enumerate(views[label]),key=lambda item:(item[1]['version'],item[0]))[1]
    def commands(label):return [c for c in result['commands'] if c['player']==label]
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage'])
        contexts={label:await browser.new_context(viewport=size,locale='zh-CN') for label,size in [('host',{'width':1366,'height':900}),('guest',{'width':390,'height':844})]}
        pages={label:await context.new_page() for label,context in contexts.items()}
        host=pages['host'];guest=pages['guest']
        for label,page in pages.items():
            page.on('pageerror',lambda error:result['pageErrors'].append(str(error)))
            async def response(r,label=label):
                if '/api/rooms/' not in urlsplit(r.url).path:return
                try:
                    d=await r.json();v=d.get('view',d)
                    if isinstance(v,dict) and 'players' in v and 'you' in v:views[label].append(v)
                except Exception:pass
            def request(r,label=label):
                if r.method!='POST' or not urlsplit(r.url).path.endswith('/commands'):return
                assert urlsplit(r.url).hostname in {'localhost','127.0.0.1'}
                d=r.post_data_json;action=d['action']
                result['commands'].append({'player':label,'commandId':d['commandId'],'expectedVersion':d['expectedVersion'],
                    'kind':action['kind'],'nestedKind':action.get('action',{}).get('kind'),'windowId':action.get('windowId')})
            page.on('response',response);page.on('request',request)
        async def make_deck(page):
            await page.goto(args.url,wait_until='networkidle')
            await page.get_by_role('navigation',name='大厅导航').get_by_role('button',name='卡牌收藏',exact=True).click()
            await page.get_by_role('textbox',name='牌组名称',exact=True).fill('阶段让过隔离测试')
            await page.get_by_role('button',name='添加 无知路人（JC125）',exact=True).click()
            await page.get_by_role('spinbutton',name='无知路人（JC125）张数',exact=True).fill('50')
            await page.get_by_role('button',name='保存并选择此牌组',exact=True).click()
        async def begin(page,target):
            await page.get_by_label('连续让过的目标阶段',exact=True).select_option(target)
            await page.get_by_role('button',name='一键连续让过',exact=True).click()
            await expect(page.locator('[data-continuous-pass]')).to_have_attribute('data-continuous-pass','active')
        async def manual_pass(page,label,seen):
            view=latest(label);key=(view['version'],view.get('responseWindow',{}).get('id') if view.get('responseWindow') else None)
            if key in seen:return
            button=page.get_by_role('button',name='不连锁，让过',exact=True)
            if await button.count() and await button.is_enabled():
                seen.add(key);await button.click();return
            button=page.locator('.hg-dock-action [data-action-kind="pass"]')
            if await button.count() and await button.is_enabled():seen.add(key);await button.click()
        async def until(predicate,timeout=65,drive_guest=True):
            end=asyncio.get_running_loop().time()+timeout;seen=set()
            while not predicate():
                assert asyncio.get_running_loop().time()<end,'phase progression timed out'
                if drive_guest:await manual_pass(guest,'guest',seen)
                await asyncio.sleep(.12)
        try:
            await make_deck(host)
            await host.get_by_role('textbox',name='你的称呼',exact=True).fill('阶段测试房主')
            await host.get_by_role('button',name='创建牌桌 →',exact=True).click()
            await host.locator('.hg-invite strong').wait_for();invite=await host.locator('.hg-invite strong').inner_text()
            await make_deck(guest)
            await guest.goto(args.url+'/?invite='+invite,wait_until='networkidle')
            await guest.get_by_role('textbox',name='你的称呼',exact=True).fill('阶段测试伙伴')
            await guest.get_by_role('button',name='加入牌桌 →',exact=True).click()
            await expect(host.locator('.hg-seat-empty')).to_have_count(0)
            await host.get_by_role('button',name='准备',exact=True).click()
            await expect(guest.locator('.hg-room-ready > span')).to_have_text(re.compile(r'1\s*/\s*2'))
            await guest.get_by_role('button',name='准备',exact=True).click()
            await expect(host.locator('.hg-room-ready > span')).to_have_text(re.compile(r'2\s*/\s*2'))
            await host.locator('.hg-room-ready').get_by_role('button',name=re.compile('开始')).click()
            await host.get_by_role('dialog',name='待完成的选择',exact=True).wait_for()
            check('cannot-arm-during-mandatory-initial-choice',await host.locator('[aria-label="让过到指定阶段"]').get_by_role('button',name='一键连续让过',exact=True,include_hidden=True).is_disabled())
            for page in [host,guest]:await page.get_by_role('dialog',name='待完成的选择',exact=True).get_by_role('button',name='保留全部手牌',exact=True).click()
            for page in [host,guest]:await expect(page.get_by_role('dialog',name='待完成的选择',exact=True)).to_have_count(0)
            await expect(host.locator('.hg-phase-line')).not_to_contain_text('再调度')
            before=len(commands('host'))
            await host.get_by_role('button',name='一键连续让过',exact=True).dblclick()
            await expect(host.locator('[data-continuous-pass]')).to_have_attribute('data-continuous-pass','off')
            await host.wait_for_timeout(800)
            check('double-click-cancels-without-extra-command',len(commands('host'))==before)
            await begin(host,'action')
            await until(lambda:latest('host')['phase']=='action')
            await expect(host.get_by_label('连续让过状态',exact=True)).to_contain_text('已到达行动阶段')
            boundary=latest('host')['version'];await host.wait_for_timeout(800)
            check('stops-before-action-phase-command',not any(c['expectedVersion']>=boundary for c in commands('host')[before:]))
            check('host-run-never-arms-guest',await guest.locator('[data-continuous-pass]').get_attribute('data-continuous-pass')=='off')
            before=len(commands('host'));await begin(host,'end');await host.reload(wait_until='networkidle')
            await expect(host.get_by_label('连续让过状态',exact=True)).to_contain_text('已关闭')
            await host.wait_for_timeout(800)
            check('refresh-does-not-resume-or-submit-a-new-pass',len(commands('host'))==before)
            # Follow the real initiative rather than assuming the host acts first.
            acting_team=int(latest('host')['step'].split(':')[1])
            host_team=next(player['team'] for player in latest('host')['players'] if player['id']==latest('host')['you'])
            attacker_label='host' if host_team==acting_team else 'guest'
            responder_label='guest' if attacker_label=='host' else 'host'
            attacker=pages[attacker_label];responder=pages[responder_label]
            # Exercise actual mobile action hit targets regardless of initiative.
            await attacker.set_viewport_size({'width':390,'height':844})
            # Manual, zero-cost deployment creates a genuine response-intent window.
            await attacker.locator('.hg-hand .hg-card').first.click()
            deploy=attacker.locator('.hg-inspector').get_by_role('button',name=re.compile('派遣')).first
            await deploy.scroll_into_view_if_needed()
            result['mobileActionLayout']=await deploy.evaluate('''button => {
                const inspector=button.closest('.hg-inspector'),dock=document.querySelector('.hg-hand-dock');
                const b=button.getBoundingClientRect(),i=inspector.getBoundingClientRect(),d=dock.getBoundingClientRect();
                const hit=document.elementFromPoint(b.x+b.width/2,b.y+b.height/2);
                return {buttonCenterReachable:button===hit||button.contains(hit),inspectorBottom:i.bottom,dockTop:d.top,
                    viewportWidth:innerWidth,position:getComputedStyle(inspector).position,dockHeight:getComputedStyle(document.querySelector('.hg-table')).getPropertyValue('--hg-dock-height')};
            }''')
            check('mobile-manual-card-action-is-not-covered-by-dock',result['mobileActionLayout']['buttonCenterReachable'] and result['mobileActionLayout']['inspectorBottom']<=result['mobileActionLayout']['dockTop'])
            await deploy.click()
            target_region=attacker.locator('[data-region-available="true"] .hg-region-center').first
            await target_region.evaluate("element => { element.scrollIntoView({block:'start',inline:'nearest'}); window.scrollBy(0,-96); }")
            result['mobileTargetLayout']=await target_region.evaluate('''button => {
                const b=button.getBoundingClientRect(),hit=document.elementFromPoint(b.x+b.width/2,b.y+b.height/2);
                const i=document.querySelector('.hg-inspector').getBoundingClientRect(),d=document.querySelector('.hg-hand-dock').getBoundingClientRect();
                return {targetCenterReachable:button===hit||button.contains(hit),targetTop:b.top,targetBottom:b.bottom,
                    inspectorTop:i.top,inspectorBottom:i.bottom,dockTop:d.top};
            }''')
            check('mobile-board-target-remains-reachable-while-choosing',result['mobileTargetLayout']['targetCenterReachable'])
            await target_region.click()
            await attacker.locator('[aria-label="确认目标行动"] [data-action-kind="deploy"]').click()
            await responder.get_by_role('button',name='不连锁，让过',exact=True).wait_for()
            before=len(commands(responder_label));await begin(responder,'end')
            await until(lambda:any(c['kind']=='passResponse' for c in commands(responder_label)[before:]),drive_guest=False,timeout=15)
            await expect(responder.locator('.hg-response-window')).to_have_attribute('data-response-state','passed')
            check('continuous-run-declines-own-real-response-window',commands(responder_label)[before]['kind']=='passResponse')
            await contexts[responder_label].set_offline(True)
            await expect(responder.get_by_label('连续让过状态',exact=True)).to_contain_text('连接中断',timeout=12000)
            await contexts[responder_label].set_offline(False)
            await expect(responder.get_by_role('button',name='一键连续让过',exact=True)).to_be_enabled(timeout=12000)
            check('read-disconnection-stops-run-after-own-pass',await responder.locator('[data-continuous-pass]').get_attribute('data-continuous-pass')=='off')
            decline=attacker.get_by_role('button',name='不连锁，让过',exact=True)
            if await decline.count() and await decline.is_enabled():await decline.click()
            await until(lambda:not latest('host').get('responseWindow'),drive_guest=False,timeout=15)
            await host.set_viewport_size({'width':1366,'height':900})
            await guest.set_viewport_size({'width':390,'height':844})
            await begin(host,'confrontation')
            await until(lambda:latest('host')['phase']=='confrontation')
            await expect(host.get_by_label('连续让过状态',exact=True)).to_contain_text('已到达对抗阶段')
            check('stops-at-confrontation-mobility',latest('host')['step']=='mobility')
            turn=latest('host')['turn'];await begin(host,'end')
            await host.screenshot(path=str(out/'desktop-active.png'),full_page=True)
            await until(lambda:latest('host')['turn']>turn,timeout=100)
            await expect(host.get_by_label('连续让过状态',exact=True)).to_contain_text('本回合已结束')
            boundary=latest('host')['version'];count=len(commands('host'));await host.wait_for_timeout(800)
            check('end-target-does-not-pass-in-next-turn',len(commands('host'))==count)
            # Both players independently arm the next turn; hand-limit cleanup must stop them.
            await until(lambda:latest('guest')['turn']==latest('host')['turn'],drive_guest=False,timeout=10)
            auto_start_index=len(result['commands'])
            await begin(host,'end');await begin(guest,'end')
            await until(lambda:bool(latest('host').get('pendingChoice') or latest('host').get('waitingChoice')),drive_guest=False,timeout=100)
            for page in [host,guest]:await expect(page.locator('[data-continuous-pass]')).to_have_attribute('data-continuous-pass','off',timeout=8000)
            check('real-hand-limit-choice-stops-both-independent-runs',bool(latest('guest').get('pendingChoice') or latest('host').get('pendingChoice')))
            await guest.screenshot(path=str(out/'phone-mandatory-stop.png'),full_page=True)
            last_commands=result['commands'][auto_start_index:]
            check('automation-never-pays-selects-or-composes',all(c['kind']=='passResponse' or c['kind']=='game' and c['nestedKind']=='pass' for c in last_commands))
            unique={}
            for c in result['commands']:
                key=c['commandId'];payload=(c['player'],c['expectedVersion'],c['kind'],c['nestedKind'],c['windowId'])
                if key in unique:assert unique[key]==payload
                unique[key]=payload
            check('duplicate-identities-never-change-actor-or-payload',True)
            check('no-page-errors',not result['pageErrors']);result['passed']=True
        except Exception as e:
            result.update(passed=False,failure=type(e).__name__+': '+str(e))
            for label,page in pages.items():
                try:await page.screenshot(path=str(out/(label+'-failure.png')),full_page=True)
                except Exception:pass
        finally:
            result['finalViews']={label:[{k:v[k] for k in ['version','you','turn','phase','step','status']} for v in history[-3:]] for label,history in views.items()}
            for c in contexts.values():await c.close()
            await browser.close()
    (out/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'passed':result['passed'],'checks':len(result['checks']),'commands':len(result['commands']),'failure':result.get('failure')},ensure_ascii=False))
    return 0 if result['passed'] else 1


raise SystemExit(asyncio.run(main()))
