from pathlib import Path
from playwright.sync_api import sync_playwright,expect
import json,hashlib,time
out=Path('/tmp/pause-resume-evidence/live-browser');out.mkdir(exist_ok=True)
r={'scope':'public-Site40-real-UI-and-Worker-WASM-D1-natural-duel','productionWrites':True,'checks':[],'errors':[],'passed':False}
get_view="async()=>{const s=JSON.parse(localStorage.getItem('hegemony.session.v1'));return (await fetch('/api/rooms/'+s.roomId+'/state',{headers:{Authorization:'Bearer '+s.token}})).json()}"
with sync_playwright() as p:
 b=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--disable-gpu'])
 b2=b
 a_ctx=b.new_context(ignore_https_errors=True, viewport={'width':1440,'height':1000}); c_ctx=b2.new_context(ignore_https_errors=True, viewport={'width':390,'height':844})
 a=a_ctx.new_page();c=c_ctx.new_page()
 for page in [a,c]:page.on('pageerror',lambda e:r['errors'].append(str(e)))
 try:
  a.goto('https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site',wait_until='networkidle')
  a.get_by_role('textbox',name='你的称呼').fill('Site40 QA A')
  a.get_by_role('button',name='两人对决').click()
  catalog=a.evaluate("async()=>await(await fetch('/api/catalog')).json()");assert catalog['engineVersion']=='rust-v0.2.46-pause-resume-candidate';r['catalog']={k:catalog[k] for k in ['engineVersion','cardPoolVersion','rulesVersion']};r['stage']='create';a.get_by_role('button',name='创建牌桌 →',exact=True).click()
  expect(a.get_by_role('heading',name='等待秘社集结')).to_be_visible()
  code=a.evaluate("()=>JSON.parse(localStorage.getItem('hegemony.session.v1')).inviteCode")
  c.goto('https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site/?invite='+code,wait_until='networkidle')
  c.get_by_role('textbox',name='你的称呼').fill('Site40 QA B')
  r['stage']='join';c.get_by_role('button',name='加入牌桌 →',exact=True).click()
  expect(c.get_by_role('heading',name='等待秘社集结')).to_be_visible()
  r['stage']='peer-ready';c.get_by_role('button',name='准备',exact=True).click()
  expect(a.get_by_text('1 / 2 位玩家已准备',exact=True)).to_be_visible()
  r['stage']='host-ready';a.get_by_role('button',name='准备',exact=True).click()
  expect(a.get_by_text('2 / 2 位玩家已准备',exact=True)).to_be_visible()
  r['stage']='start-game';a.get_by_role('button',name='开始游戏',exact=True).click()
  expect(a.get_by_role('dialog',name='待完成的选择')).to_be_visible()
  before=a.evaluate(get_view);choice=before['pendingChoice']['id']
  r['stage']='chooser-pause';a.get_by_role('button',name='暂停并保存此桌',exact=True).click()
  expect(a.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
  expect(a.get_by_role('dialog',name='待完成的选择')).to_have_count(0)
  saved=a.evaluate(get_view);assert saved['pause'] and saved['pendingChoice']['id']==choice
  a.screenshot(path=str(out/'desktop-paused-private-choice.png'));r['checks'].append('chooser-can-pause-and-resume-controls-are-accessible')
  reads=[]
  def observe(req):
   if '/state' in req.url:reads.append(req.url.split('?')[0])
  a.on('request',observe)
  a.wait_for_timeout(3500);assert not reads,reads
  a.get_by_role('button',name='返回大厅 / 新建牌桌',exact=True).click()
  expect(a.get_by_role('region',name='我的牌桌')).to_be_visible()
  a.reload(wait_until='networkidle')
  a.get_by_role('button',name='回到牌桌 '+code+' · 席位 1',exact=True).click()
  expect(a.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
  restored=a.evaluate(get_view);assert restored['version']==saved['version'] and restored['pendingChoice']['id']==choice
  r['checks'].append('idle-no-poll-and-leave-reload-my-tables-restores-original-seat')
  # A peer receives the pause, then resumes the same paid/choice boundary.
  c.reload(wait_until='networkidle')
  expect(c.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
  c.screenshot(path=str(out/'mobile-peer-paused.png'))
  c.get_by_role('button',name='恢复对局',exact=True).click()
  expect(c.get_by_role('button',name='暂停并保存',exact=True)).to_be_enabled()
  a.get_by_role('button',name='查看最新状态',exact=True).click()
  expect(a.get_by_role('dialog',name='待完成的选择')).to_be_visible()
  after=a.evaluate(get_view);assert 'pause' not in after and after['pendingChoice']['id']==choice
  assert [x['instanceId'] for x in after['hand']]==[x['instanceId'] for x in saved['hand']]
  a.screenshot(path=str(out/'desktop-resumed-same-choice.png'))
  r['checks'].append('peer-resume-manual-sync-preserves-choice-and-hand-instance-ids')
  r['versions']=after['versions'];r['savedRevision']=saved['version'];r['resumedRevision']=after['version'];r['samePrivateChoice']=True
  assert after['versions']['engine']=='rust-v0.2.46-pause-resume-candidate'
  r['stage']='continue-original-choice';a.get_by_role('button',name='保留全部手牌',exact=True).click()
  expect(a.get_by_role('dialog',name='待完成的选择')).to_have_count(0)
  continued=a.evaluate(get_view);assert continued['version']>after['version'];assert not continued.get('pendingChoice') or continued['pendingChoice']['id']!=choice
  r['checks'].append('original-choice-actually-submitted-after-resume')
  r['stage']='leave-new-QA-table-paused';a.get_by_role('button',name='暂停并保存',exact=True).click()
  expect(a.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
  final=a.evaluate(get_view);assert final['pause'];r['continuedRevision']=continued['version'];r['finalPausedRevision']=final['version'];r['roomId']=final['roomId'];r['testTableLeftPaused']=True
  a.screenshot(path=str(out/'desktop-continued-and-paused.png'))
  assert not r['errors'];r['passed']=True
 except Exception as e:
  r['error']=str(e)[:1800]
  for page,name in [(a,'failure-a.png'),(c,'failure-b.png')]:
   if not page.is_closed():
    try:page.screenshot(path=str(out/name))
    except Exception:pass
 finally:
  if not a.is_closed():
   try:Path('/tmp/pause-resume-evidence/site40-private-qa-seat.json').write_text(a.evaluate("()=>localStorage.getItem('hegemony.session.v1')") or 'null')
   except Exception:pass
  b.close();b2.close();r['screenshots']={f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in out.glob('*.png')};(out/'results.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(r,ensure_ascii=False))
if not r['passed']:raise SystemExit(1)
