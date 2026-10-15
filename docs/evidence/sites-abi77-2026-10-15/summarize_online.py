"""Finalize evidence only after this owned public PC match actually finishes."""
from pathlib import Path
from collections import Counter
import json,shutil

base=Path('/workspace/abi77-publication-20261015')
run=base/'online-pc-v54'
target=Path('/workspace/UndergroundBattle/docs/evidence/sites-abi77-2026-10-15')
load=lambda p:json.loads(p.read_text())
final=load(run/'full-game-summary.json')
assert final['passed'] and final['completeGame'] and final['final']['status']=='finished'
assert final['roomId']=='fd08636bd17bf0e956e194d1'
assert final['final']['versions']['engine']=='rust-v0.2.77-jc046-mandatory-sacrifice-candidate'
assert final['final']['versions']['cardPool']=='limited-v2.70-jc046-mandatory-sacrifice-candidate'
identity=load(run/'online-identity.json')
assert identity['all158Match'] and len(identity['frontendFiles'])==158
rows=[json.loads(line) for line in (run/'duel-actions.jsonl').read_text().splitlines()]
rejected=[row for row in rows if row['httpStatus']!=200]
for row in rows:
 assert row['before']['versions']['engine']==final['final']['versions']['engine']
 assert row.get('after') and row['after']['versions']['engine']==final['final']['versions']['engine']
for row in rejected:
 assert row['httpStatus']==409 and row['kind']=='passResponse' and row['error']['error']=='window_expired'
 assert row['after']['version']>row['before']['version']
attempt_dirs=sorted(run.glob('attempt[1-4]-*'))
attempts=[load(p/'full-game-summary.json') for p in attempt_dirs]+[final]
assert len(attempts)==5 and all(s['roomId']==final['roomId'] for s in attempts)
assert all(not s['passed'] and not s['completeGame'] for s in attempts[:-1])
assert [s['uiPostCount'] for s in attempts]==sorted(s['uiPostCount'] for s in attempts)
network=[post for attempt in attempts for post in attempt['postResults']]
assert len(network)==final['uiPostCount']
assert len(rows)+2==len(network) # one creation + one join, all actual UI POSTs
assert Counter(post['status'] for post in network)==Counter({200:len(network)-len(rejected),409:len(rejected)})
assert len({row['commandId'] for row in rows})==len(rows)
assert not any(e['kind']=='pageerror' for attempt in attempts for e in attempt['browserErrors'])
assert not final['browserErrors']
console_409='Failed to load resource: the server responded with a status of 409 (Conflict)'
prior_console_errors=[e for attempt in attempts[:-1] for e in attempt['browserErrors']]
for e in prior_console_errors:
 assert e['kind']=='console' and e['message']==console_409
 assert any(row['seat']==e['seat'] for row in rejected)
transport=load(run/'online-transport.json')
expected_console_count=len(prior_console_errors)+len(transport.get('expectedBrowserErrors',[]))
assert expected_console_count==len(rejected)
checks=load(run/'online-pc-checks.json')
names={c['name'] for c in checks['checks'] if c['passed']}
required={'public-engine-pool-and-all158-frontend-hashes','real-choice-select-unselect-reselect',
 'reload-retains-own-seat-choice-and-selected-draft','online-pause-reload-resume-keeps-choice-and-draft',
 'rapid-choice-clicks-one-command','paused-peer-page-refreshed-after-other-seat-resume'}
assert required<=names
result=dict(passed=True,completeOnlineGame=True,mode='duel',viewport=[1180,760],
 candidateSha='4918cfb6372fb9ec900f037e8ed518385d6183e6',version=54,
 deploymentId='appgdep_6ad09dcf575c819182ed0f3340eccfb1',roomId=final['roomId'],
 publicAnonymous=True,hostingOwnerBypass=False,stateInjection=False,apiMoves=False,
 sameOwnedRoomAcrossAttempts=True,scriptAttempts=5,
 uiPostCount=len(network),uiCommandRecords=len(rows),acceptedUiCommands=len(rows)-len(rejected),
 expectedExpiredResponse409s=len(rejected),unexpectedPageErrors=0,
 expectedBrowser409ConsoleEvents=expected_console_count,unexpectedBrowserErrors=0,
 status=final['final']['status'],turn=final['final']['turn'],winnerTeam=final['final']['winnerTeam'],
 scores=[p['score'] for p in final['final']['players']],versions=final['final']['versions'],
 requiredPcChecksPassed=sorted(required),
 actionKinds=dict(Counter(row['kind'] for row in rows if row['httpStatus']==200)),
 priorInterruptedAttempts=[dict(name=p.name,passed=s['passed'],failure=s['failure'],uiPostCount=s['uiPostCount']) for p,s in zip(attempt_dirs,attempts[:-1])],
 rejectedCommands=[{k:row[k] for k in ['seat','kind','httpStatus','commandId','expectedVersion','action','error']} for row in rejected],
 limits=['PC only; no mobile acceptance','One natural duel, not full-card mechanism acceptance',
         'Paused peers refresh after another seat resumes; no claim of automatic paused polling',
         'Native save content_hash is recorded separately from the local tar.gz/tar whole-byte hashes'])
for name in ['full-game-summary.json','online-pc-checks.json','online-transport.json','duel-actions.jsonl']:
 shutil.copy2(run/name,target/name)
shutil.copy2(base/'online_pc_abi77.py',target/'online_pc_abi77.py')
for folder in attempt_dirs:
 dest=target/'interrupted-attempts'/folder.name;dest.mkdir(parents=True,exist_ok=True)
 for name in ['full-game-summary.json','online-transport.json']:shutil.copy2(folder/name,dest/name)
for dest in [base/'online-complete-summary.json',target/'online-complete-summary.json']:
 dest.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['passed','completeOnlineGame','roomId','uiPostCount','acceptedUiCommands','expectedExpiredResponse409s','turn','winnerTeam','scores']},ensure_ascii=False))
