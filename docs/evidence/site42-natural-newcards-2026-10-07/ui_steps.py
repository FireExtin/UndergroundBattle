"""Restore genuine original local QA seat ACK into a new browser after tool reconnect.
Only authentication storage is restored. No runtime game state or fake API replies.
All choices/actions use actual production UI against local published Worker/WASM/D1.
"""
import importlib.util,json,sqlite3,sys,time
from pathlib import Path
from playwright.sync_api import sync_playwright,expect

ROOT=Path('/workspace/game-publication-evidence/site42-natural-newcards')
spec=importlib.util.spec_from_file_location('qa_driver',ROOT/'driver.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
DB=next(Path('/tmp/site42-natural-newcards-local-d1/v3/d1/miniflare-D1DatabaseObject').glob('*.sqlite'))
ROOM='d61b44bbe38eada719bcc4d8'
db=sqlite3.connect('file:'+str(DB)+'?mode=ro',uri=True)
for row in db.execute('SELECT response FROM entry_receipts WHERE room_id=?',(ROOM,)):m.CREDS.append(json.loads(row[0]))
m.CREDS.sort(key=lambda r:r['seat']);assert [x['seat'] for x in m.CREDS]==[0,1]
instruction=json.loads(sys.argv[1]);seat=instruction.get('seat',0)
saved={k:m.CREDS[seat][k] for k in ['roomId','inviteCode','token','seat']}
with sync_playwright() as p:
    browser=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--disable-gpu','--single-process','--no-zygote','--blink-settings=imagesEnabled=false'])
    try:
        context=browser.new_context(viewport={'width':1440,'height':1000})
        context.add_init_script('if(!localStorage.getItem("hegemony.session.v1")){localStorage.setItem("hegemony.session.v1",'+json.dumps(json.dumps(saved))+');localStorage.setItem("hegemony.screen.v1","table");}')
        page=context.new_page();m.PAGES.extend([page,page]);m.NAMESPACES.extend([None,None])
        page.on('pageerror',lambda e:m.ERRORS.append(str(e)))
        page.on('request',lambda req:m.ui_wire(seat,req))
        page.goto(m.BASE,wait_until='networkidle');expect(page.locator('.hg-app')).to_be_visible()
        m.OUT=ROOT/f"ui-v{m.view(seat)['version']}";m.OUT.mkdir(exist_ok=True)
        if instruction['op']=='choose':
            m.choose_ui(seat,instruction.get('selected',[]),instruction.get('pause',False))
            if m.COMMANDS:
                sessionAction=m.COMMANDS[-1]['command']['action']
                payload=sessionAction.get('action',sessionAction)
                m.CHECKS[-1]['actualUiPayload']=payload
        elif instruction['op']=='shot':page.screenshot(path=str(ROOT/instruction['name']))
        elif instruction['op']=='inspect-sealed':
            v=m.view(seat);card=v['sealedCards'][0]
            assert card['kind']=='sealed' and all(k not in card for k in ['cost','icons','defense','magic','damage','wounds','shield'])
            page.locator('.hg-sealed-cards-menu > summary').click()
            tile=page.locator(f'[data-sealed-instance="{card["instanceId"]}"]')
            expect(tile).to_have_attribute('data-sealed-host',card['hostId'])
            hostText=tile.inner_text();assert '拥有' in hostText and '载体' in hostText and '操控' in hostText
            tile.click();reader=page.get_by_role('dialog',name='放大阅读'+card['name'])
            expect(reader).to_be_visible();content=reader.locator('.hg-reading-card')
            expect(content).to_contain_text('已封印 · 场外空白牌')
            expect(content.locator('.hg-icons, .hg-card-cost, .hg-card-stats')).to_have_count(0)
            page.screenshot(path=str(m.OUT/f'sealed-reader-seat{seat}.png'))
            m.CHECKS.append({'check':'actual-UI-public-sealed-reader-has-host-owner-and-blank-state','seat':seat,'version':v['version'],'sealedId':card['instanceId'],'cardId':card['cardId'],'hostId':card['hostId'],'hostLabel':hostText,'costIconsDefenseAbsent':True,'artworkQa':False})
        elif instruction['op']=='inspect-returned':
            v=m.view(seat);card=next(c for c in v['hand'] if c.get('cardId')=='XQ41')
            assert not v.get('sealedCards',[]) and card['kind']=='character' and card['cost']==3 and card['owner']==card['controller']=='p0'
            page.locator(f'[data-card-instance="{card["instanceId"]}"]').click()
            page.get_by_role('button',name='放大文字与图标 ↗',exact=True).click()
            reader=page.get_by_role('dialog',name='放大阅读'+card['name']);expect(reader).to_be_visible()
            expect(reader.locator('.hg-reading-card')).to_contain_text('被封印在一个角色')
            expect(reader.locator('.hg-reading-card .hg-icons')).to_have_count(2)
            expect(reader.locator('.hg-reading-card')).not_to_contain_text('场外空白牌')
            page.screenshot(path=str(m.OUT/'returned-XQ41-reader.png'))
            m.CHECKS.append({'check':'actual-UI-returned-XQ41-regains-character-print-in-owner-hand','seat':seat,'version':v['version'],'returnedId':card['instanceId'],'cost':card['cost'],'owner':card['owner'],'iconsVisible':True,'sealedMarkerAbsent':True,'artworkQa':False})
        elif instruction['op']=='pause-final':
            before=m.view(seat)
            page.get_by_role('button',name='暂停并保存',exact=True).click()
            expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
            page.reload(wait_until='domcontentloaded');expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
            after=m.view(seat);assert after.get('pause') and before.get('pendingChoice')==after.get('pendingChoice') and before['hand']==after['hand']
            page.screenshot(path=str(m.OUT/'finite-end-paused.png'))
            m.CHECKS.append({'check':'finite-local-QA-table-left-paused-through-real-UI','versions':[before['version'],after['version']],'handPreserved':True,'commandsAtStop':db.execute('SELECT COUNT(*) FROM commands WHERE room_id=?',(ROOM,)).fetchone()[0]})
        else:raise ValueError('Unsupported UI operation')
        result=m.snapshot();result['restoredOriginalSeatAuthenticationOnly']=True;result['uncaughtJsErrors']=m.ERRORS
        previous=json.loads((ROOT/'resumed-ui-checks.json').read_text()) if (ROOT/'resumed-ui-checks.json').exists() else {'checks':[],'commands':[]}
        previous['checks'].extend(m.CHECKS);previous['commands'].extend(m.COMMANDS)
        (ROOT/'resumed-ui-checks.json').write_text(json.dumps(previous,ensure_ascii=False,indent=2)+'\n')
        (ROOT/'resumed-ui-progress.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
        print(json.dumps({'checks':m.CHECKS,'uncaughtJsErrors':m.ERRORS,'views':[{k:v[k] for k in ['seat','version','turn','phase','hand','board','pending','sealed','stack']} for v in result['views']]},ensure_ascii=False))
    finally:browser.close()
