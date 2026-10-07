"""Normal local HTTP commands for the UI-created third table. No DB writes.
Read ONLY the two original entry ACKs to reuse our already authenticated QA seats.
Never read/copy/replace private state into the running game.
"""
import importlib.util,json,sqlite3,sys,uuid
from pathlib import Path
ROOT=Path('/workspace/game-publication-evidence/site42-natural-newcards')
spec=importlib.util.spec_from_file_location('qa_driver',ROOT/'driver.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
DB=next(Path('/tmp/site42-natural-newcards-local-d1/v3/d1/miniflare-D1DatabaseObject').glob('*.sqlite'))
ROOM='d61b44bbe38eada719bcc4d8'
db=sqlite3.connect('file:'+str(DB)+'?mode=ro',uri=True)
for row in db.execute('SELECT response FROM entry_receipts WHERE room_id=?',(ROOM,)):
    receipt=json.loads(row[0]);m.CREDS.append(receipt)
m.CREDS.sort(key=lambda r:r['seat']);assert [x['seat'] for x in m.CREDS]==[0,1]
TRACE=ROOT/'http-commands.jsonl'
def send(seat,a):
    total=db.execute('SELECT COUNT(*) FROM commands WHERE room_id=?',(ROOM,)).fetchone()[0]
    if total>=400:raise RuntimeError('Finite400 command cap reached')
    v=m.view(seat);allowed={'kind','cardId','targetId','region','option','choiceId','selected','top','bottom','allocations','abilityId','costSelected','deckDraft'}
    payload={k:x for k,x in a.items() if k in allowed and x is not None}
    window=v.get('responseWindow');member=next((x for x in (window or {}).get('members',[]) if x['playerId']==v['you']),None)
    if window and not(v.get('pendingChoice') or v.get('waitingChoice')):
        if payload['kind']=='pass' and member and member['status']=='undecided':session={'kind':'passResponse','windowId':window['id']}
        elif payload['kind']=='pass' and member and member['status']=='composing':session={'kind':'cancelAndPass','windowId':window['id'],'intentId':window['myIntentId']}
        elif member and member['status']=='composing':session={'kind':'submitResponse','windowId':window['id'],'intentId':window['myIntentId'],'action':payload}
        else:raise RuntimeError('Wait for response-window member decision')
    else:session={'kind':'game','action':payload}
    body={'commandId':str(uuid.uuid4()),'expectedVersion':v['version'],'action':session}
    after=m.wire('/api/rooms/'+ROOM+'/commands',body,seat)
    with TRACE.open('a') as f:f.write(json.dumps({'seat':seat,'command':body,'resultVersion':after['version']},ensure_ascii=False)+'\n')
    return after
def advance(n,goal):
    for _ in range(min(n,30)):
        if db.execute('SELECT COUNT(*) FROM commands WHERE room_id=?',(ROOM,)).fetchone()[0]>=390:return 'finite-stop-with-ten-commands-reserved'
        views=[m.view(0),m.view(1)]
        if any(v.get('pendingChoice') for v in views):return 'private-choice'
        if goal=='inspect' and any(v['phase']=='action' and not v.get('responseWindow') and any(a['kind']!='pass' for a in v['legalActions']) for v in views):return 'action-phase-inspection'
        candidates=[]
        for seat,v in enumerate(views):
            board=[c for r in v['regions'] for c in r['characters']]
            cards={c['instanceId']:c for c in v['hand']+v['assets']+board}
            own_assets=[c for c in v['assets'] if c['controller']==v['you']]
            own_board=[c for c in board if c['controller']==v['you']]
            colors=[c.get('color','').replace('色','') for c in own_assets]
            counts={id_:sum(c.get('cardId')==id_ for c in v['hand']) for id_ in ['XQ45','XQ41','XQ40','JZ50','XQ16']}
            for a in v['legalActions']:
                c=cards.get(a.get('cardId'),{});id_=c.get('cardId');rank=0
                if a['kind']=='asset':
                    if seat==0:
                        rank=100 if id_ in {'JC104','JZ59','JZ61','JZ58'} and colors.count('紫')<3 else 70 if id_=='JC125' else 60 if id_ in {'JC104','JZ59','JZ61','JZ58'} else 0
                        if id_=='XQ45' and counts['XQ45']>1:rank=95
                        if id_=='XQ40' and colors.count('紫')<3 and any(x.get('cardId')=='XQ40' for x in own_board):rank=100
                    else:
                        rank=100 if id_ in {'JZ49','JC085','JZ48'} and '黑' not in colors else 98 if id_ in {'XQ12','JC029'} and '蓝' not in colors else 70 if id_=='JC125' else 60 if id_ in {'JZ49','JC085','JZ48','XQ12','JC029'} else 0
                if a['kind']=='deploy' and a.get('region')==0:
                    if seat==0 and id_=='XQ40' and not any(x.get('cardId')=='XQ40' for x in own_board):rank=120
                    if goal=='natural' and seat==0 and id_=='JZ61' and v.get('turn',0)>=7 and v.get('sealedCards'):rank=120
                    if seat==1 and id_=='JC125' and not any(x.get('cardId')=='JC125' and not x.get('faceDown') for x in own_board):rank=110
                    if goal in {'flip','jz50second','natural'} and seat==1 and id_=='XQ16':rank=130
                if a['kind']=='conceal' and seat==1 and id_=='JZ50' and (goal.startswith('jz50') or goal=='natural') and sum(x.get('cardId')=='JZ50' for x in own_board)<(2 if goal in {'jz50second','natural'} else 1):rank=140
                if a['kind']=='reveal' and seat==1 and id_=='JZ50' and (goal.startswith('jz50') or goal=='natural'):rank=150
                if a.get('abilityId') and seat==0 and id_=='XQ40' and (goal.startswith('seal') or goal=='natural'):
                    target=cards.get(a.get('targetId'),{})
                    if target.get('controller')=='p1' and not target.get('faceDown'):rank=160
                if a['kind']=='cast' and seat==0 and id_=='XQ45' and goal in {'destroy','natural'} and any(s['hostId']==a.get('targetId') for s in v.get('sealedCards',[])):rank=170
                if a['kind']=='pass':rank=1
                if v.get('responseWindow') and a['kind']=='pass':
                    member=next((x for x in v['responseWindow']['members'] if x['playerId']==v['you']),None)
                    if member and member['status']=='undecided':rank=200
                    else:rank=0
                if rank:candidates.append((rank,seat,a))
        if not candidates:return 'no-selected-legal-action'
        _,seat,a=max(candidates,key=lambda t:t[0]);send(seat,a)
    return 'batch-complete'

instruction=json.loads(sys.argv[1]);op=instruction['op']
if op=='advance':stopped=advance(instruction.get('n',20),instruction.get('goal','seal'))
elif op=='act':
    v=m.view(instruction['seat']);a=next(a for a in v['legalActions'] if a['id']==instruction['id']);send(instruction['seat'],a);stopped='one-legal-action'
else:stopped='read'
result=m.snapshot();result['stopped']=stopped;result['commands']=db.execute('SELECT COUNT(*) FROM commands WHERE room_id=?',(ROOM,)).fetchone()[0]
for row in result['views']:
    current=m.view(row['seat'])
    row['relevantActions']=[a for a in current['legalActions'] if a['kind'] in {'activate','cast','reveal'}]
    row['actualAssets']=current['assets']
(ROOT/'http-progress.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'commands':result['commands'],'stopped':stopped,'views':[{k:v[k] for k in ['seat','version','turn','phase','hand','assets','board','pending','sealed','legalKinds']} for v in result['views']]},ensure_ascii=False))
