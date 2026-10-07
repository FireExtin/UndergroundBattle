from pathlib import Path
from playwright.sync_api import sync_playwright, expect
import json, hashlib
out=Path('/workspace/game-publication-evidence/site41-local-single-ui');out.mkdir(exist_ok=True)
r={'scope':'local-only-production-Site41-build-single-seat-real-UI-peer-normal-HTTP-commands','productionWrites':False,'localD1Writes':True,'checks':[],'passed':False,'errors':[]}
state="async()=>{const s=JSON.parse(localStorage.getItem('hegemony.session.v1'));return(await fetch('/api/rooms/'+s.roomId+'/state',{headers:{Authorization:'Bearer '+s.token}})).json()}"
with sync_playwright() as p:
    b=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--disable-gpu','--single-process','--no-zygote'])
    ctx=b.new_context(viewport={'width':1440,'height':1000});a=ctx.new_page();a.on('pageerror',lambda e:r['errors'].append(str(e)))
    try:
        a.goto('http://127.0.0.1:8113',wait_until='networkidle')
        cat=a.evaluate("async()=>await(await fetch('/api/catalog')).json()")
        assert cat['engineVersion']=='rust-v0.2.49-sealed-restart-candidate' and len(cat['cards'])==107 and len(cat['societies'])==8
        r['catalog']={k:cat[k] for k in ['engineVersion','cardPoolVersion','rulesVersion']}
        r['newFourCards']={i:next(c['name'] for c in cat['cards'] if c['id']==i) for i in ['XQ40','XQ41','XQ45','JZ50']}
        a.get_by_role('textbox',name='你的称呼').fill('Local Site41 UI A');a.get_by_role('button',name='两人对决').click();a.get_by_role('button',name='创建牌桌 →',exact=True).click()
        expect(a.get_by_role('heading',name='等待秘社集结')).to_be_visible()
        peer=a.evaluate("async()=>{const s=JSON.parse(localStorage.getItem('hegemony.session.v1'));const response=await fetch('/api/rooms/join',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({inviteCode:s.inviteCode,name:'Local Site41 API B',deckId:'watchers',requestId:crypto.randomUUID()})});if(!response.ok)throw Error('join status '+response.status);return response.json()}")
        def peer_command(action):
            return a.evaluate("async args=>{const headers={Authorization:'Bearer '+args.peer.token,'Content-Type':'application/json'};const path='/api/rooms/'+args.peer.roomId;const view=await(await fetch(path+'/state',{headers})).json();const response=await fetch(path+'/commands',{method:'POST',headers,body:JSON.stringify({commandId:crypto.randomUUID(),expectedVersion:view.version,action:args.action})});if(!response.ok)throw Error('peer command status '+response.status);return response.json()}",{'peer':peer,'action':action})
        peer_command({'kind':'game','action':{'kind':'ready'}})
        expect(a.get_by_text('1 / 2 位玩家已准备',exact=True)).to_be_visible()
        a.get_by_role('button',name='准备',exact=True).click();expect(a.get_by_text('2 / 2 位玩家已准备',exact=True)).to_be_visible()
        a.get_by_role('button',name='开始游戏',exact=True).click();expect(a.get_by_role('dialog',name='待完成的选择')).to_be_visible()
        before=a.evaluate(state);choice=before['pendingChoice']['id'];code=before['inviteCode']
        a.get_by_role('button',name='暂停并保存此桌',exact=True).click();expect(a.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled();saved=a.evaluate(state)
        assert saved['pause'] and saved['pendingChoice']['id']==choice;a.screenshot(path=str(out/'paused.png'));r['checks'].append('real-UI-create-ready-start-private-choice-pause')
        reads=[];a.on('request',lambda req:reads.append(req.url) if '/state' in req.url else None);a.wait_for_timeout(3500);assert not reads
        a.get_by_role('button',name='返回大厅 / 新建牌桌',exact=True).click();a.reload(wait_until='networkidle');a.get_by_role('button',name='回到牌桌 '+code+' · 席位 1',exact=True).click()
        expect(a.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled();restored=a.evaluate(state);assert restored['version']==saved['version'] and restored['pendingChoice']['id']==choice;r['checks'].append('pause-stops-poll-leave-reload-restores-original-seat-and-choice')
        peer_command({'kind':'resumeRoom'});a.get_by_role('button',name='查看最新状态',exact=True).click();expect(a.get_by_role('dialog',name='待完成的选择')).to_be_visible();resumed=a.evaluate(state)
        assert not resumed.get('pause') and resumed['pendingChoice']['id']==choice;assert [c['instanceId'] for c in resumed['hand']]==[c['instanceId'] for c in saved['hand']]
        a.screenshot(path=str(out/'resumed.png'));r['checks'].append('real-peer-API-resume-and-UI-manual-sync-preserve-private-choice')
        a.get_by_role('button',name='保留全部手牌',exact=True).click();expect(a.get_by_role('dialog',name='待完成的选择')).to_have_count(0);continued=a.evaluate(state);assert continued['version']>resumed['version']
        a.get_by_role('button',name='暂停并保存',exact=True).click();expect(a.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled();final=a.evaluate(state);assert final['pause'];a.screenshot(path=str(out/'continued-paused.png'))
        r['roomId']=final['roomId'];r['revisions']=[saved['version'],resumed['version'],continued['version'],final['version']];r['checks'].append('UI-actually-submits-original-choice-after-restore');assert not r['errors'];r['passed']=True
    except Exception as e:
        r['error']=str(e)[:1800]
    finally:
        b.close();r['screenshots']={f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in out.glob('*.png')};(out/'results.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(r,ensure_ascii=False))
if not r['passed']:raise SystemExit(1)
