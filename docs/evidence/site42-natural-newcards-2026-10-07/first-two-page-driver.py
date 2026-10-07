"""Finite local natural QA. Credentials stay only in this running process/browser.
Only seeded factory RNG and legal deck metadata are supplied; never insert a board,
hand, deck order, persisted room or choice. Setup and peers use real UI/HTTP.
"""
import json,sys,uuid,time,re,hashlib,urllib.request,urllib.error
from pathlib import Path
from playwright.sync_api import sync_playwright,expect

BASE='http://127.0.0.1:8114'
OUT=Path('/workspace/game-publication-evidence/site42-natural-newcards')
PLAN=json.loads(Path('/workspace/game-publication-evidence/site42-natural-seed-plan.json').read_text())
SEED=16
CREDS=[];PAGES=[];COMMANDS=[];CHECKS=[];ERRORS=[];CAP=400
def wire(path,body=None,seat=None):
    headers={'Content-Type':'application/json'}
    if seat is not None:headers['Authorization']='Bearer '+CREDS[seat]['token']
    req=urllib.request.Request(BASE+path,data=None if body is None else json.dumps(body).encode(),headers=headers)
    try:
        with urllib.request.urlopen(req,timeout=12) as response:return json.load(response)
    except urllib.error.HTTPError as e:
        detail=json.loads(e.read());raise RuntimeError(f'Local HTTP{e.code}: '+detail.get('message','failed'))
def view(seat):return wire('/api/rooms/'+CREDS[seat]['roomId']+'/state',seat=seat)
def act(seat,action):
    if len(COMMANDS)>=CAP:raise RuntimeError('Finite400command cap reached')
    v=view(seat);body={'commandId':str(uuid.uuid4()),'expectedVersion':v['version'],'action':action}
    result=wire('/api/rooms/'+CREDS[seat]['roomId']+'/commands',body,seat)
    COMMANDS.append({'via':'normal-local-HTTP','seat':seat,'command':body,'resultVersion':result.get('version')})
    return result
def game_action(a):
    keep={'kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected','deckDraft'}
    return {'kind':'game','action':{k:v for k,v in a.items() if k in keep and v is not None}}
def ui_wire(seat,request):
    if '/commands' not in request.url or request.method!='POST':return
    body=request.post_data_json
    COMMANDS.append({'via':'actual-browser-UI','seat':seat,'command':body})
def refresh(seat):
    page=PAGES[seat]
    page.reload(wait_until='domcontentloaded')
    expect(page.locator('.hg-app')).to_be_visible()
    v=view(seat)
    if v.get('pendingChoice') and not v.get('pause'):expect(page.get_by_role('dialog',name='待完成的选择')).to_be_visible()
    return v
def choose_ui(seat,selected,pause=False):
    page=PAGES[seat];v=refresh(seat);choice=v['pendingChoice'];ident=choice['id']
    before={'version':v['version'],'choice':choice,'handIds':[c['instanceId'] for c in v['hand']]}
    if pause:
        page.get_by_role('button',name='暂停并保存此桌',exact=True).click()
        expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
        saved=view(seat);assert saved['pause'] and saved['pendingChoice']['id']==ident
        page.screenshot(path=str(OUT/f'{len(CHECKS):02}-{choice["kind"]}-paused.png'))
        page.reload(wait_until='domcontentloaded')
        expect(page.get_by_role('button',name='恢复对局',exact=True)).to_be_enabled()
        restored=view(seat)
        assert restored['pendingChoice']==saved['pendingChoice'] and [c['instanceId'] for c in restored['hand']]==before['handIds']
        page.get_by_role('button',name='恢复对局',exact=True).click()
        expect(page.get_by_role('dialog',name='待完成的选择')).to_be_visible()
        restored=view(seat);assert not restored.get('pause') and restored['pendingChoice']==choice
        CHECKS.append({'check':'actual-UI-pause-reload-resume-preserves-choice','seat':seat,'choiceKind':choice['kind'],'choiceId':ident,'optionIds':[o['id'] for o in choice['options']],'versions':[before['version'],saved['version'],restored['version']]})
    for option in selected:page.locator(f'[data-choice-option="{option}"]').click()
    page.screenshot(path=str(OUT/f'{len(CHECKS):02}-{choice["kind"]}-before-submit.png'))
    if choice['kind']=='mulligan' and not selected:page.get_by_role('button',name='保留全部手牌',exact=True).click()
    elif choice['kind']=='jz50_death_search' and not selected:page.get_by_role('button',name='不取牌并洗牌',exact=True).click()
    else:page.get_by_role('button',name='确认选择',exact=True).click()
    expect(page.locator(f'[data-choice-id="{ident}"]')).to_have_count(0)
    after=view(seat)
    CHECKS.append({'check':'actual-UI-choice-submit','seat':seat,'choiceKind':choice['kind'],'choiceId':ident,'selected':selected,'versions':[before['version'],after['version']]})
    return after
def snapshot():
    rows=[]
    for seat in range(2):
        v=view(seat);pending=v.get('pendingChoice')
        rows.append({'seat':seat,'version':v['version'],'turn':v.get('turn'),'phase':v['phase'],'step':v.get('step'),
          'hand':[(c['instanceId'],c.get('cardId')) for c in v['hand']],
          'assets':[(c['instanceId'],c.get('cardId'),c.get('exhausted')) for c in v['assets']],
          'board':[(r['index'],c['instanceId'],c.get('cardId'),c.get('controller'),c.get('faceDown')) for r in v['regions'] for c in r['characters']],
          'sealed':v.get('sealedCards',[]),'graveyard':[(c['instanceId'],c.get('cardId')) for c in v['graveyard']],
          'pending':pending,'waiting':v.get('waitingChoice'),'responseWindow':v.get('responseWindow'),'stack':v.get('stack'),
          'legalSample':v['legalActions'][:18], 'legalKinds':sorted(set(a['kind'] for a in v['legalActions']))})
    return {'commands':len(COMMANDS),'checks':CHECKS,'views':rows}
def advance(n,goal):
    for _ in range(min(n,30)):
        views=[view(0),view(1)]
        if any(v.get('pendingChoice') for v in views):return {'stopped':'private-choice',**snapshot()}
        moved=False
        for seat,v in enumerate(views):
            a=next((a for a in v['legalActions'] if a['kind']=='passResponse'),None)
            if a:
                act(seat,{k:val for k,val in a.items() if k in {'kind','windowId'}});moved=True;break
        if moved:continue
        candidates=[]
        for seat,v in enumerate(views):
            cards={c['instanceId']:c for c in v['hand']+v['assets']+[c for r in v['regions'] for c in r['characters']]}
            for a in v['legalActions']:
                c=cards.get(a.get('cardId'),{});id_=c.get('cardId');rank=0
                if a['kind']=='asset':
                    if seat==0:rank=100 if id_ in {'JC104','JZ59','JZ61','JZ58','XQ45'} else 45 if id_=='JC125' else 0
                    else:rank=100 if id_ in {'JZ49','JC085','JZ48'} and not any(x.get('color')=='黑' for x in v['assets']) else 98 if id_ in {'XQ12','JC029'} and not any(x.get('color')=='蓝' for x in v['assets']) else 45 if id_=='JC125' else 30 if id_ in {'JZ49','JC085','JZ48','XQ12','JC029'} else 0
                if a['kind']=='play' and a.get('region')==0:
                    if seat==0 and id_=='XQ40' and not any(x.get('cardId')=='XQ40' for r in v['regions'] for x in r['characters']):rank=120
                    if seat==1 and id_=='JC125' and not any(x['controller']=='p1' and not x.get('faceDown') for r in v['regions'] for x in r['characters']):rank=110
                    if goal=='flip' and seat==1 and id_=='XQ16':rank=130
                if a['kind']=='conceal' and seat==1 and id_=='JZ50' and goal.startswith('jz50'):rank=140
                if a['kind']=='reveal' and seat==1 and id_=='JZ50' and goal.startswith('jz50'):rank=150
                if a.get('abilityId') and seat==0 and id_=='XQ40' and goal.startswith('seal'):
                    target=cards.get(a.get('targetId'),{})
                    if target.get('controller')=='p1' and not target.get('faceDown'):rank=160
                if a['kind']=='play' and seat==0 and id_=='XQ45' and goal=='destroy':rank=170
                if a['kind']=='pass':rank=1
                if rank>0:candidates.append((rank,seat,a))
        if not candidates:return {'stopped':'no-selected-legal-action',**snapshot()}
        _,seat,a=max(candidates,key=lambda x:x[0]);act(seat,game_action(a))
    return snapshot()

def main():
    with sync_playwright() as p:
        browser=p.chromium.launch(executable_path='/usr/bin/chromium',headless=True,args=['--no-sandbox','--disable-dev-shm-usage','--disable-gpu','--single-process','--no-zygote'])
        context=browser.new_context(viewport={'width':1440,'height':1000},extra_http_headers={'X-Local-QA-Factory-Seed':str(SEED)})
        context.add_init_script('localStorage.setItem("hegemony.deckLibrary.v1",'+json.dumps(json.dumps({'version':1,'drafts':PLAN['decks']}))+');')
        try:
            a=context.new_page();PAGES.append(a);a.on('pageerror',lambda e:ERRORS.append(str(e)));a.on('request',lambda r:ui_wire(0,r))
            a.goto(BASE,wait_until='networkidle');a.locator('.hg-library-saved').get_by_role('button').filter(has_text='Local seal QA').first.click()
            a.get_by_role('button',name='保存并选择此牌组',exact=True).click();a.get_by_role('textbox',name='你的称呼').fill('Local UI A')
            with a.expect_response(lambda r:r.request.method=='POST' and r.url==BASE+'/api/rooms') as response:a.get_by_role('button',name='创建牌桌 →',exact=True).click()
            receipt=response.value.json();assert response.value.headers.get('x-local-qa-factory-seed-applied')==str(SEED);CREDS.append(receipt)
            expect(a.get_by_role('heading',name='等待秘社集结')).to_be_visible()
            b=context.new_page();PAGES.append(b);b.on('pageerror',lambda e:ERRORS.append(str(e)));b.on('request',lambda r:ui_wire(1,r))
            b.goto(BASE,wait_until='networkidle');b.get_by_role('button',name='开始新的独立玩家会话',exact=True).click()
            b.locator('.hg-library-saved').get_by_role('button').filter(has_text='Local reveal QA').first.click();b.get_by_role('button',name='保存并选择此牌组',exact=True).click()
            b.get_by_role('button',name='邀请码加入',exact=True).click();b.get_by_role('textbox',name='你的称呼').fill('Local UI B');b.get_by_role('textbox',name='邀请码',exact=True).fill(receipt['inviteCode'])
            with b.expect_response(lambda r:r.request.method=='POST' and r.url==BASE+'/api/rooms/join') as joined:b.get_by_role('button',name=re.compile('加入牌桌'),exact=False).click()
            CREDS.append(joined.value.json());b.get_by_role('button',name='准备',exact=True).click();a.get_by_role('button',name='准备',exact=True).click()
            expect(a.get_by_role('button',name='开始游戏',exact=True)).to_be_enabled();a.get_by_role('button',name='开始游戏',exact=True).click()
            CHECKS.append({'check':'real-UI-custom-decks-create-independent-peer-join-ready-start','seed':SEED,'roomId':receipt['roomId'],'no_board_or_hand_injection':True})
            print(json.dumps({'ready':True,**snapshot()},ensure_ascii=False),flush=True)
            for line in sys.stdin:
                try:
                    instruction=json.loads(line);op=instruction['op']
                    if op=='read':result=snapshot()
                    elif op=='advance':result=advance(instruction.get('n',20),instruction.get('goal','seal'))
                    elif op=='choose':choose_ui(instruction['seat'],instruction.get('selected',[]),instruction.get('pause',False));result=snapshot()
                    elif op=='action':
                        v=view(instruction['seat']);action=next(a for a in v['legalActions'] if a['id']==instruction['id']);act(instruction['seat'],game_action(action));result=snapshot()
                    elif op=='shot':PAGES[instruction.get('seat',0)].screenshot(path=str(OUT/instruction['name']));result={'screenshot':instruction['name']}
                    elif op=='end':break
                    else:raise ValueError('Unknown local QA instruction')
                    print(json.dumps(result,ensure_ascii=False),flush=True)
                    (OUT/'progress.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
                except Exception as e:print(json.dumps({'error':str(e)},ensure_ascii=False),flush=True)
        finally:
            browser.close()
            (OUT/'commands.json').write_text(json.dumps(COMMANDS,ensure_ascii=False,indent=2)+'\n')
            (OUT/'checks.json').write_text(json.dumps({'scope':'local actual published Worker/WASM/D1/browser; factory-seed-only shim and legal deck metadata; normal UI/HTTP commands, no board injection','seed':SEED,'commandCap':CAP,'commands':len(COMMANDS),'checks':CHECKS,'uncaughtJsErrors':ERRORS,'publicWrites':False},ensure_ascii=False,indent=2)+'\n')

if __name__=='__main__':main()
