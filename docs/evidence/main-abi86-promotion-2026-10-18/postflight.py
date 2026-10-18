from pathlib import Path
import datetime,json,subprocess
repo=Path('/workspace/UndergroundBattle');out=Path(__file__).parent
pre=json.loads((out/'preflight.json').read_text());target=pre['qualifiedFixedTarget']
def api(p):return json.loads(subprocess.check_output(['gh','api','repos/FireExtin/UndergroundBattle/'+p],cwd=repo))
remote=subprocess.check_output(['git','ls-remote','--refs','origin','refs/heads/main'],cwd=repo,text=True).split()[0];assert remote==target
runs=api('actions/runs?per_page=100');status=api('commits/'+target+'/status');checks=api('commits/'+target+'/check-runs');associated=api('actions/runs?head_sha='+target+'&per_page=100');workflows=api('actions/workflows')
ids=sorted(r['id'] for r in runs['workflow_runs']);assert ids==sorted(r['id'] for r in pre['CI']['existingRuns']) and runs['total_count']==8
assert status['total_count']==checks['total_count']==associated['total_count']==0
assert [(x['id'],x['path'],x['state']) for x in workflows['workflows']]==[(x['id'],x['path'],x['state']) for x in pre['CI']['configuredWorkflows']]
local=subprocess.run(['git','show-ref','--verify','--quiet','refs/heads/main'],cwd=repo)
assert local.returncode in [0,1]
result={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'remoteMain':remote,'normalFastForwardSucceeded':True,'candidateOnly':True,'JC041Merged':False,
 'JC041WorkingHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
 'CI':{'workflowRunIdsUnchanged':ids,'totalRepoRuns':8,'allExistingEventsDynamic':all(r['event']=='dynamic' for r in runs['workflow_runs']),
 'targetWorkflowRuns':associated['total_count'],'targetCheckRuns':checks['total_count'],'targetStatuses':status['total_count'],
 'emptyAggregateState':status['state'],'emptyPendingIsNotAPassedOrRunningCI':True,'configuredWorkflowsUnchanged':True},
 'noActionsManuallyTriggered':True,'noPermissionsOrConfigurationChanged':True,'noDeployment':True,
 'localMainExists':local.returncode==0,'localMainNotCreatedOrRewritten':True,
 'priorPostcheckBoundary':'First read-only postcheck passed remote and CI assertions, then failed only because diagnostic rev-parse assumed a localmain ref. No second push; this read-only rerun records absentlocal main explicitly.'}
with (out/'result.json').open('x') as f:json.dump(result,f,indent=2,ensure_ascii=False);f.write('\n')
print(json.dumps(result))
