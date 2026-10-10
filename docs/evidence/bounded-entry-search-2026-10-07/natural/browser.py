"""One finite local natural four-seat UI/HTTP match. Factory RNG seed9 only.
No hand/deck order/board/choice/opaque-state injection. Credentials stay in memory.
"""
import json,re,uuid,urllib.request,urllib.error,pathlib,time
from playwright.sync_api import sync_playwright,expect
ROOT=pathlib.Path('/workspace/game-publication-evidence/bounded-search50/natural');BASE='http://127.0.0.1:8120'
CREDS=[];PAGES=[];NS=[None];COMMANDS=[];CHECKS=[];ERRORS=[];active=0

def wire(path,body=None,seat=None):
 h={'Content-Type':'application/json'}
 if seat is not None:h['Authorization']='Bearer '+CREDS[seat]['token']
 req=urllib.request.Request(BASE+path,data=None if body is None else json.dumps(body).encode(),headers=h)
 with urllib.request.urlopen(req,timeout=12) as r:return json.load(r)
def view(seat):return wire('/api/rooms/'+CREDS[seat]['roomId']+'/state',seat=seat)
def act(seat,a):
 v=view(seat);c={'commandId':str(uuid.uuid4()),'expectedVersion':v['version'],'action':a}
 r=wire('/api/rooms/'+CREDS[seat]['roomId']+'/commands',c,seat);COMMANDS.append({'via':'normal-HTTP','seat':seat,'command':c,'resultVersion':r['version']});return r
def game(a):return {'kind':'game','action':{k:v for k,v in a.items() if k in {'kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected'} and v is not None}}
def refresh(seat):
 global active,page
 active=seat;page=PAGES[seat];page.reload(wait_until='domcontentloaded');expect(page.locator('.hg-app')).to_be_visible();return view(seat)
def submit(seat,selected,pause=False):
 v=refresh(seat);p=v['pendingChoice'];ident=p['id'];expect(page.get_by_role('dialog',name='待完成的选择')).to_be_visible()
 if pause:
  page.get_by_role('button',name='暂停并保存此桌',exact=True).click();expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled();saved=view(seat);assert saved['pause'] and saved['pendingChoice']==p
  page.screenshot(path=str(ROOT/(p['kind']+'-paused.png')));page.reload(wait_until='domcontentloaded');expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled();assert view(seat)['pendingChoice']==p
  page.get_by_role('button',name='恢复对局',exact=True).click();expect(page.get_by_role('dialog',name='待完成的选择')).to_be_visible();assert view(seat)['pendingChoice']==p
  CHECKS.append({'check':'actual-UI-pause-reload-resume','choiceKind':p['kind'],'seat':seat,'sameChoiceAndOptions':True})
 for id in selected:page.locator('[data-choice-option="'+id+'"]').click()
 page.screenshot(path=str(ROOT/(p['kind']+'-selection.png')))
 page.get_by_role('button',name='保留全部手牌' if p['kind']=='mulligan' and not selected else '不取牌并洗牌' if p['kind'] in ['bq104_employee_search','xq48_passer_search'] and not selected else '确认选择',exact=True).click();expect(page.locator('[data-choice-id="'+ident+'"]')).to_have_count(0)
 after=view(seat);CHECKS.append({'check':'actual-UI-submit','choiceKind':p['kind'],'seat':seat,'selected':selected,'versions':[v['version'],after['version']]});return after

catalog=wire('/api/catalog');assert catalog['engineVersion']=='rust-v0.2.50-bounded-entry-search-candidate';assert len(catalog['cards'])==109
DRAFT={'id':'bounded-entry-natural','name':'Local entry search QA','description':'One finite local legal50 match','societyId':None,'cards':[{'cardId':'BQ104','count':3},{'cardId':'XQ48','count':3},{'cardId':'JC125','count':44}],'rulesVersion':catalog['rulesVersion'],'cardPoolVersion':catalog['cardPoolVersion'],'engineVersion':catalog['engineVersion'],'updatedAt':'2026-10-07'}
result={'scope':'candidate50 actual local compiled Worker + D1 + production client; UI entry/choices, HTTP phase progress; no public acceptance','finiteLimit':600,'newTables':3,'factorySeed':9,'noStateInjection':True}
with sync_playwright() as p:
 browser=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--disable-gpu','--blink-settings=imagesEnabled=false'])
 context=browser.new_context(viewport={'width':1440,'height':1000},extra_http_headers={'X-Local-QA-Factory-Seed':'9'})
 context.add_init_script('localStorage.setItem("hegemony.deckLibrary.v1",'+json.dumps(json.dumps({'version':1,'drafts':[DRAFT]}))+');');page=context.new_page();PAGES.append(page)
 page.on('pageerror',lambda e:ERRORS.append(str(e)))
 page.on('request',lambda r:COMMANDS.append({'via':'actual-browser-UI','seat':0,'command':r.post_data_json}) if r.method=='POST' and '/commands' in r.url else None)
 try:
  page.goto(BASE,wait_until='networkidle');page.locator('.hg-library-saved').get_by_role('button').filter(has_text='Local entry search QA').first.click();page.get_by_role('button',name='保存并选择此牌组',exact=True).click();page.get_by_role('button',name=re.compile('四人协作')).click();page.get_by_role('textbox',name='你的称呼').fill('Entry UI P0')
  with page.expect_response(lambda r:r.request.method=='POST' and r.url==BASE+'/api/rooms') as received:page.get_by_role('button',name='创建牌桌 →',exact=True).click()
  assert received.value.headers.get('x-local-qa-factory-seed-applied')=='9';CREDS.append(received.value.json());expect(page.get_by_role('heading',name='等待秘社集结')).to_be_visible()
  for seat in [1,2,3]:
   active=seat;peer_context=browser.new_context(viewport={'width':1440,'height':1000});peer_context.add_init_script('localStorage.setItem("hegemony.deckLibrary.v1",'+json.dumps(json.dumps({'version':1,'drafts':[DRAFT]}))+');');page=peer_context.new_page();PAGES.append(page);page.on('pageerror',lambda e:ERRORS.append(str(e)));page.on('request',lambda r,s=seat:COMMANDS.append({'via':'actual-browser-UI','seat':s,'command':r.post_data_json}) if r.method=='POST' and '/commands' in r.url else None);page.goto(BASE,wait_until='networkidle');page.locator('.hg-library-saved').get_by_role('button').filter(has_text='Local entry search QA').first.click();page.get_by_role('button',name='保存并选择此牌组',exact=True).click();page.get_by_role('button',name='邀请码加入',exact=True).click();page.get_by_role('textbox',name='你的称呼').fill('Entry UI P'+str(seat));page.get_by_role('textbox',name='邀请码',exact=True).fill(CREDS[0]['inviteCode'])
   with page.expect_response(lambda r:r.request.method=='POST' and r.url==BASE+'/api/rooms/join') as joined:page.get_by_role('button',name=re.compile('加入牌桌')).click()
   CREDS.append(joined.value.json());page.get_by_role('button',name='准备',exact=True).click();expect(page.get_by_role('button',name='取消准备',exact=True)).to_be_visible()
  refresh(0);page.get_by_role('button',name='准备',exact=True).click();expect(page.get_by_role('button',name='开始游戏',exact=True)).to_be_enabled();page.get_by_role('button',name='开始游戏',exact=True).click();CHECKS.append({'check':'four-actual-independent-UI-entries-custom50-ready-start','seats':4})
  for seat in range(4):submit(seat,[])
  completed=set()
  for turn in range(600):
   if len(COMMANDS)>=600:raise RuntimeError('finite600command cap')
   if len(completed)==2:break
   views=[view(s) for s in range(4)];pending=next(((s,v['pendingChoice']) for s,v in enumerate(views) if v.get('pendingChoice')),None)
   if pending:
    seat,c=pending
    if c['kind'] in ['bq104_employee_search','xq48_passer_search']:
     selected=[o['id'] for o in c['options'][:c['min'] if c['kind']=='bq104_employee_search' else c['max']]]
     for s in range(4):
      if s!=seat:assert views[s].get('pendingChoice') is None
     before=views[seat];after=submit(seat,selected,True);completed.add(c['kind'])
     CHECKS.append({'check':'natural-entry-result','kind':c['kind'],'seat':seat,'selectedCount':len(selected),'deckCounts':[before['players'][seat]['deckCount'],after['players'][seat]['deckCount']]})
     if c['kind']=='bq104_employee_search':
      hidden=next(x for r in after['regions'] for x in r['characters'] if x.get('cardId')=='BQ104' and x.get('faceDown'))
      for other in range(4):
       if other!=seat:
        ov=view(other);oc=next(x for r in ov['regions'] for x in r['characters'] if x['instanceId']==hidden['instanceId']);assert oc.get('cardId') is None
      CHECKS.append({'check':'natural-hidden-public-employee-only-controller-can-read','instanceId':hidden['instanceId'],'controllerSeat':seat,'otherSeats':3})
    else:
     selected=['accept'] if c['kind']=='trigger' else [o['id'] for o in c['options'][:c.get('min',0)]];act(seat,{'kind':'game','action':{'kind':'choose','choiceId':c['id'],'selected':selected}})
   else:
    responsive=next(((s,a) for s,v in enumerate(views) for a in v['legalActions'] if a['kind']=='passResponse'),None)
    if responsive:seat,a=responsive;act(seat,{'kind':'passResponse','windowId':a['windowId']})
    else:
     choices=[]
     for s,v in enumerate(views):
      hand={x['instanceId']:x for x in v['hand']}
      for a in v['legalActions']:
       d=hand.get(a.get('cardId'),{}).get('cardId');rank=100 if a['kind']=='asset' and d=='JC125' else 90 if a['kind']=='play' and a.get('region')==2 and ((d=='BQ104' and 'bq104_employee_search' not in completed) or (d=='XQ48' and 'xq48_passer_search' not in completed)) else 1 if a['kind']=='pass' else 0
       if rank:choices.append((rank,s,a))
     if not choices:raise RuntimeError('No selected legal action')
     _,seat,a=max(choices,key=lambda row:row[0]);act(seat,game(a))
   (ROOT/'progress.json').write_text(json.dumps({'commands':len(COMMANDS),'completed':sorted(completed),'checks':len(CHECKS)},ensure_ascii=False)+'\n')
  assert len(completed)==2
  refresh(0);page.get_by_role('button',name='暂停并保存此桌',exact=True).click();expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled();page.screenshot(path=str(ROOT/'final-paused.png'))
  result.update({'roomId':CREDS[0]['roomId'],'completed':sorted(completed),'passed':True,'terminalVersion':view(0)['version'],'terminalTurn':view(0)['turn']})
 except Exception as e:
  result.update({'passed':False,'error':str(e)});
  try:page.screenshot(path=str(ROOT/'failed.png'))
  except Exception:pass
  raise
 finally:
  browser.close();result.update({'commands':len(COMMANDS),'uiCommands':sum(x['via']=='actual-browser-UI' for x in COMMANDS),'httpCommands':sum(x['via']=='normal-HTTP' for x in COMMANDS),'checks':CHECKS,'pageErrors':ERRORS});(ROOT/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');(ROOT/'commands.json').write_text(json.dumps(COMMANDS,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='checks'},ensure_ascii=False),flush=True)
