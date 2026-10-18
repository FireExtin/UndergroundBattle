from pathlib import Path
import datetime,gzip,hashlib,json,re,shutil,subprocess
repo=Path('/workspace/UndergroundBattle');root=Path('/workspace/jc066-evidence-20261018');out=root/'evidence-staging'
def sha(b):return hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
required=['native-full-second','frontend-full-second','worker-batch-first','jz46-target-first','production-first','fixed-clock-first','source-guards-first','current-entry-first','frozen85-first','native-wasm-final','browser-actor1-first','browser-reveal-first','browser-shared-first']
receipts={name:json.loads((root/(name+'.exit.json')).read_text()) for name in required}
for name,x in receipts.items():assert x['exitCode']==(1 if name=='worker-batch-first' else 0),(name,x)
native=(root/'native-full-second.log').read_text();groups=[int(n) for n in re.findall(r'^test result: ok\. (\d+) passed;',native,re.M)]
assert sum(groups)==1313 and max(groups)==1246 and 'test result: FAILED' not in native
front=(root/'frontend-full-second.log').read_text();assert re.search(r'Test Files\s+139 passed \(139\)',front);assert re.search(r'Tests\s+1342 passed \(1342\)',front)
worker=(root/'worker-batch-first.log').read_text()
def total(label):
 matches=re.findall(r'^ℹ '+label+r' (\d+)$',worker,re.M);assert len(matches)==1,(label,matches);return int(matches[0])
worker_total=total('tests');worker_passed=total('pass');worker_failed=total('fail')
assert (worker_total,worker_passed,worker_failed,total('cancelled'))==(306,294,12,0)
assert set(re.findall(r'^test at ([^:]+):',worker,re.M))=={'test/jz46-integration.test.mjs'}
assert worker.count("actual: 'rust-v0.2.86-jc066-sealed-icons-candidate'")==12
assert worker.count("expected: 'rust-v0.2.85-jc044-time-repress-candidate'")==12
initial_part=worker.split('ℹ tests ')[0]
failed_names=[re.sub(r' \([\d.]+ms\)$','',line[2:]) for line in initial_part.splitlines() if line.startswith('✖ ')]
assert len(failed_names)==12
retest=(root/'jz46-target-first.log').read_text()
def retotal(label):
 matches=re.findall(r'^ℹ '+label+r' (\d+)$',retest,re.M);assert len(matches)==1;return int(matches[0])
retest_count=retotal('tests');assert retotal('pass')==retest_count and retotal('fail')==0 and retotal('cancelled')==0
passed_names={re.sub(r' \([\d.]+ms\)$','',line[2:]) for line in retest.splitlines() if line.startswith('✔ ')}
assert set(failed_names)<=passed_names,(set(failed_names)-passed_names)
worker_boundary={'firstBatch':{'tests':worker_total,'passed':worker_passed,'failed':worker_failed,'failureFile':'sites/test/jz46-integration.test.mjs','failuresAllGlobalCatalog85Vs86':True,'failedNames':failed_names},'jz46FullFileRetest':{'tests':retest_count,'passed':retest_count,'failed':0},'full306NotRerunAfterTestOnlyFix':True,'noCrossRoundPassTotalClaim':True,'productionBytesUnchanged':True}
normal=[];bad=[];case=None
for line in worker.splitlines():
 if line.startswith('✔ '):case=line
 if line.startswith('ℹ {'):
  d=json.loads(line[2:])
  if d.get('reference')=='compiler-bound Native86':normal.append(d)
  if case and case.startswith('✔ JC066 corrupted ') and 'invalid' in d:bad.append(d);case=None
assert len(normal)==15 and len(bad)==85,(len(normal),len(bad))
counts={'scenarios':len(normal),'accepted':sum(d['accepted'] for d in normal),'rejected':sum(d['rejected'] for d in normal),'projections':sum(d['projections'] for d in normal),'corruptRooms':len(bad),'corruptHttp400':sum(d['http400'] for d in bad)}
assert counts=={'scenarios':15,'accepted':160,'rejected':16,'projections':1464,'corruptRooms':85,'corruptHttp400':850},counts
for d in normal:assert d['originalReceiptsAfterReopen'] and d['opaqueStatesAndJournalsEqual']
for d in bad:assert d['realD1Reopen'] and d['roomCommandsJournalUnchanged']
fixture=repo/'rust-game-wasm/tests/jc066Native86.fixture.json.gz';bundle=json.loads(gzip.decompress(fixture.read_bytes()));raw=json.loads(bundle['raw']['json'])
assert (root/'native-full-second/native86.json').read_bytes()==(root/'native-society-third/native86.json').read_bytes()
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip();files=raw['sourceBindings']['files']
for name,h in files.items():
 b=subprocess.check_output(['git','show',head+':'+name],cwd=repo);assert sha(b)==h and (repo/name).read_bytes()==b,name
source_binding={'sourceCommit':head,'compilerInputs':len(files),'everyInputMatchesCommittedBlobAndWorkingTree':True,'nativeRawSha256':bundle['raw']['sha256'],'wasmSha256':bundle['wasmSha256'],'files':files}
assert len(files)==135
with (out/'source-binding-git.json').open('x') as f:json.dump(source_binding,f,indent=2);f.write('\n')
browsers=[]
for name,commit in [('browser-actor1-first','26972b286740d47f4cc42bdd94a377b788656561'),('browser-reveal-first',head),('browser-shared-first',head)]:
 d=json.loads((root/name/'report.json').read_text());assert d['passed'] and not d['pageErrors'] and not d['onlineGameplay']
 driver_sha=sha(subprocess.check_output(['git','show',commit+':tools/frontend/jc066-http-four-clients-qa.mjs'],cwd=repo))
 if d.get('driverSha256'):assert d['driverSha256']==driver_sha
 browsers.append({'directory':name,'scenario':d['scenario'],'browserContexts':4,'uiCommands':sum(c['via']=='production-ui' for c in d['commands']),'expectedRejectedBrowserFetches':sum(c['via']=='browser-fetch-expected-rejection' for c in d['commands']),'reopened':d['reopened'],'originalReceiptsAfterReopen':d['originalReceiptsAfterReopen'],'sealedReadSeats':[c['seat'] for c in d['sealedReads']],'hostRead':d.get('hostRead'),'driverSourceCommit':commit,'driverSha256':driver_sha,'wasmSha256':d['wasmSha256'],'clientIndexSha256':d['clientIndexSha256'],'workerSha256':d['workerSha256'],'pageErrors':d['pageErrors'],'onlineGameplay':False})
site=json.loads((root/'official-v67-09.json').read_text());browser=json.loads((root/'v67-browser09/normal-browser.json').read_text())
assert site['site']['latest_version_number']==67 and site['deployment']['status']=='succeeded' and browser['certificateBlocked'] and browser['siteWrites']==0 and browser['onlineGames']==0
summary={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'candidateSourceCommit':head,'independentReview':'pending-parent-serial-review','published':False,'mainMerged':False,'nativeWorkspace':{'passed':sum(groups),'library':1246,'integration':67,'sameSingleCommand':True,'receipt':receipts['native-full-second']},'frontend':{'files':139,'passed':1342,'receipt':receipts['frontend-full-second']},'worker':{'boundary':worker_boundary,'receipt':receipts['worker-batch-first'],'retestReceipt':receipts['jz46-target-first'],'jc066':counts,'normal':normal,'badRooms':bad},'nativeWasm':{'counts':bundle['counts'],'compilerInputs':len(files),'rawSha256':bundle['raw']['sha256'],'fixture':record(fixture),'wasmSha256':bundle['wasmSha256'],'fullRunExportEqualsSpecializedExport':True},'browsers':browsers,'siteThisHour':{'official':site,'normalBrowser':{'checkedUtc':browser['checkedUtc'],'certificateBlocked':True,'navigationRequests':len(browser['requests']),'siteWrites':0,'onlineGames':0,'bypass':False}},'latestRequiredRunReceipts':receipts,'historicalRebuildIdentityIssue':'not claimed resolved; retained ABI77–85 use exact original frozen bytes'}
with (out/'validation-summary.json').open('x') as f:json.dump(summary,f,indent=2,ensure_ascii=False);f.write('\n')
# Only completed logs, natural exit receipts and selected synthetic browser records.
for p in root.glob('*.exit.json'):
 name=p.name[:-10];shutil.copy2(p,out/'runs'/p.name);(out/'runs'/(name+'.log.gz')).write_bytes(gzip.compress((root/(name+'.log')).read_bytes(),compresslevel=9,mtime=0))
print(json.dumps({'sourceCommit':head,'nativePassed':sum(groups),'frontendPassed':1342,'workerBoundary':worker_boundary,'jc066':counts,'browserCases':len(browsers),'fixture':record(fixture),'wasmSha256':bundle['wasmSha256']}))
