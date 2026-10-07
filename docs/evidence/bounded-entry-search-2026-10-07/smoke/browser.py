"""Finite post-Go visual smoke: one fresh duel, actual UI, no state edits."""
import hashlib, json, re
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).parent
BASE = 'http://127.0.0.1:8120'
commands, errors, checks, creds = [], [], [], []
result = {'scope':'post-Go current production client + compiled Worker50 + local D1; short lobby/table visual smoke', 'maxNewTables':1, 'actualNewTables':0, 'maxUiCommands':10, 'noStateInjection':True, 'naturalNewCardMechanismsVerified':False}

with sync_playwright() as p:
    browser = p.chromium.launch(executable_path='/usr/bin/chromium', headless=True,
        args=['--no-sandbox','--disable-dev-shm-usage','--disable-gpu','--single-process','--no-zygote'])
    context = browser.new_context(viewport={'width':1440,'height':1000})
    context.set_default_timeout(15000)
    def watch(page, seat):
        page.on('pageerror',lambda e: errors.append(str(e)))
        page.on('request',lambda r: commands.append({'seat':seat,'command':r.post_data_json}) if r.method=='POST' and '/commands' in r.url else None)
    def state(seat):
        c=creds[seat]
        response=context.request.get(BASE+'/api/rooms/'+c['roomId']+'/state',headers={'Authorization':'Bearer '+c['token']})
        assert response.ok
        return response.json()
    host=context.new_page();watch(host,0)
    try:
        host.goto(BASE,wait_until='networkidle')
        expect(host.get_by_role('heading',name=re.compile('世界的背面，.*等你落子。'))).to_be_visible()
        assert host.locator('[href="/legacy-debugger"]').count()==0
        checks.append({'check':'live GameApp lobby, retired debugger absent'})
        host.screenshot(path=str(ROOT/'desktop-lobby.png'))
        host.get_by_role('textbox',name='你的称呼').fill('Post-Go Smoke P0')
        with host.expect_response(lambda r:r.request.method=='POST' and r.url==BASE+'/api/rooms') as created:
            host.get_by_role('button',name='创建牌桌 →',exact=True).click()
        assert created.value.ok
        creds.append(created.value.json());result['actualNewTables']=1
        expect(host.get_by_role('heading',name='等待秘社集结')).to_be_visible()
        peer=context.new_page();watch(peer,1);peer.goto(BASE,wait_until='networkidle')
        peer.get_by_role('button',name='开始新的独立玩家会话',exact=True).click()
        peer.get_by_role('button',name='邀请码加入',exact=True).click()
        peer.get_by_role('textbox',name='你的称呼').fill('Post-Go Smoke P1')
        peer.get_by_role('textbox',name='邀请码',exact=True).fill(creds[0]['inviteCode'])
        with peer.expect_response(lambda r:r.request.method=='POST' and r.url==BASE+'/api/rooms/join') as joined:
            peer.get_by_role('button',name='加入牌桌 →',exact=True).click()
        assert joined.value.ok
        creds.append(joined.value.json());assert creds[1]['seat']==1
        peer.get_by_role('button',name='准备',exact=True).click()
        expect(peer.get_by_role('button',name='取消准备',exact=True)).to_be_visible()
        host.reload(wait_until='networkidle');host.get_by_role('button',name='准备',exact=True).click()
        expect(host.get_by_role('button',name='开始游戏',exact=True)).to_be_enabled()
        host.screenshot(path=str(ROOT/'desktop-room-lobby.png'))
        host.get_by_role('button',name='开始游戏',exact=True).click()
        expect(host.locator('[data-table-layout="overhead"]')).to_be_visible()
        host.get_by_role('button',name='保留全部手牌',exact=True).click()
        peer.reload(wait_until='networkidle');peer.get_by_role('button',name='保留全部手牌',exact=True).click()
        host.reload(wait_until='networkidle')
        expect(host.locator('.hg-region')).to_have_count(3)
        expect(host.locator('.hg-player-mat')).to_have_count(2)
        before=state(0);assert before['versions']['engine']=='rust-v0.2.50-bounded-entry-search-candidate'
        assert len(before['players'])==2 and len(before['regions'])==3
        checks.append({'check':'actual two-seat ready/start/keep-hand UI; desktop table and 3 regions', 'version':before['version'], 'engine':before['versions']['engine']})
        host.screenshot(path=str(ROOT/'desktop-table.png'))
        for card,expected in [('BQ104','dbee5b3829cf0c46b2820beccefda28708c3c8e87c589dbefce1e975faa916eb'),('XQ48','0cd31207bda681093839e86e5bb87cee624a3c0a657c70b60fa765f439089746')]:
            response=context.request.get(BASE+'/cards/'+card+'.jpg');assert response.ok
            assert hashlib.sha256(response.body()).hexdigest()==expected
            checks.append({'check':'actual hosted original asset byte hash', 'card':card,'sha256':expected})
        host.set_viewport_size({'width':390,'height':844});host.screenshot(path=str(ROOT/'mobile-table.png'))
        expect(host.locator('[data-table-layout="overhead"]')).to_be_visible()
        checks.append({'check':'mobile table renders in current CSS', 'viewport':[390,844]})
        host.set_viewport_size({'width':1440,'height':1000})
        host.get_by_role('button',name='暂停并保存此桌',exact=True).click()
        expect(host.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
        host.reload(wait_until='networkidle');expect(host.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
        after=state(0);assert after.get('pause')
        checks.append({'check':'pause/reload retains real current room', 'version':after['version']})
        assert len(commands)<=10 and not errors
        result.update({'passed':True,'roomId':creds[0]['roomId'],'terminalVersion':after['version']})
    except Exception as error:
        result.update({'passed':False,'error':str(error)})
        try:host.screenshot(path=str(ROOT/'failed.png'))
        except Exception:pass
        raise
    finally:
        browser.close()
        result.update({'uiCommands':len(commands),'checks':checks,'pageErrors':errors,'browserStopped':True})
        (ROOT/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
        (ROOT/'commands.json').write_text(json.dumps(commands,ensure_ascii=False,indent=2)+'\n')
        print(json.dumps({k:v for k,v in result.items() if k not in ['checks','pageErrors']},ensure_ascii=False),flush=True)
