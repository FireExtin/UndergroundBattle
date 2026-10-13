import json,os,subprocess
from pathlib import Path
root=Path(os.environ['JZ35_REVIEW_ROOT']);source=Path(os.environ['JZ35_SOURCE_ROOT']);e=root/'evidence/ui-refresh';e.mkdir(parents=True,exist_ok=True)
records=[]
def run(name,args,extra=None):
    env=os.environ.copy();env.update(extra or {})
    with (e/(name+'.log')).open('w') as log:
        p=subprocess.run(args,cwd=source,env=env,stdout=log,stderr=subprocess.STDOUT)
    records.append({'name':name,'exit_code':p.returncode,'log':str(e/(name+'.log'))})
    (e/'progress.json').write_text(json.dumps(records,indent=2)+'\n')
    print(name,p.returncode,flush=True)
    if p.returncode:raise SystemExit(p.returncode)
def tests(name,pattern,extra):run(name,['cargo','test','--locked','-p','hegemony-server','--lib',pattern,'--','--test-threads=1'],extra)
for card in ['jc061','jc087','jc094','wm056','wm023','xq25','wm038']:
    base=e/card
    tests(card,card,{'GREEN_EVIDENCE_DIR':str(base/'trace'),card.upper()+'_EVIDENCE_DIR':str(base/'native'),card.upper()+'_INVALID_DIR':str(base/'invalid')})
    run(card+'-fixture',['python3','tools/fixture-tools/'+card+'-current.py',str(base),'72'])
tests('lifecycle-blue','blue_jc032',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-blue'),'BLUE_FRONTEND_DIR':str(e/'lifecycle/blue')})
tests('lifecycle-black','black_expansion',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-black'),'BLACK_EXPANSION_EVIDENCE_DIR':str(e/'lifecycle/black')})
tests('lifecycle-red','red_time',{'GREEN_EVIDENCE_DIR':str(e/'lifecycle/trace-red'),'RED_TIME_EVIDENCE_DIR':str(e/'lifecycle/red')})
run('lifecycle-fixture',['python3','tools/fixture-tools/current-four-faction-choices.py',str(e/'lifecycle'),'72','3e2a0cfba34c80b844482ba838776c079acb58c3'])
tests('death-frontends','death_observer',{'DEATH_OBSERVER_FRONTEND_DIR':str(e/'death')})
run('current-worker-web-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','current_ui_worker_fixtures','--',str(source),str(e/'death'),'72','--web-only'])
run('seven-card-fixtures',['cargo','run','--locked','-p','hegemony-wasm','--example','seven_card_ui_fixtures','--',str(source/'web/src/game/sevenCardUIV072.fixture.json')])
tests('win-flow','win_flow',{'WIN_FLOW_EVIDENCE_DIR':str(e/'win')})
run('fixed-slots-fixture',['python3','tools/fixture-tools/current-win-flow.py',str(e/'win'),'72','--web-only'])
run('bq030',['cargo','test','--locked','-p','hegemony-server','--test','bq030_attachment_regression','--','--test-threads=1'],{'BQ030_TRACE_DIR':str(e/'bq030/room')})
run('bq030-fixture',['node','tools/fixture-tools/current-bq030.mjs',str(e/'bq030'),str(source/'rust-game-wasm/pkg/hegemony_wasm.js'),'--web-only'])
base=root/'evidence'
tests('jz35','jz35',{'GREEN_EVIDENCE_DIR':str(base/'trace'),'JZ35_EVIDENCE_DIR':str(base/'native'),'JZ35_INVALID_DIR':str(base/'invalid')})
run('jz35-fixture',['python3','tools/fixture-tools/jz35-current.py',str(base)])
tests('xq24','xq24_continuous_duel_teams_room_pause_restore_journal_and_receipt_inputs',{'XQ24_TRACE_DIR':str(e/'xq24')})
rows=[json.loads(p.read_text()) for p in sorted((e/'xq24').glob('*.json'))]
assert len(rows)==12,len(rows)
(source/'web/src/game/xq24Native72.fixture.json').write_text(json.dumps({'catalog':rows[0]['catalog'],'scenarios':rows},ensure_ascii=False,separators=(',',':'))+'\n')
print('native72-input-generation-complete',flush=True)
