from pathlib import Path
import datetime,gzip,hashlib,json,re,shutil,subprocess
repo=Path('/workspace/UndergroundBattle');root=Path('/workspace/jc066-p2-evidence-20261018');out=root/'evidence-staging'
required=['native-before','wasm-http-before','native-after-first','native-after-final','wasm-after','native-wasm-after','production-after','fixed-clock-after','frontend-targeted','wasm-http-after','native-full','worker-focused-after','v67-browser10']
receipts={n:json.loads((root/(n+'.exit.json')).read_text()) for n in required}
for n,r in receipts.items():assert r['exitCode']==0,(n,r)
def sha(b):return hashlib.sha256(b).hexdigest()
def rec(p):
 b=p.read_bytes();return {'bytes':len(b),'sha256':sha(b)}
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
f=repo/'rust-game-wasm/tests/jc066Native86.fixture.json.gz';bundle=json.loads(gzip.decompress(f.read_bytes()));raw=json.loads(bundle['raw']['json'])
assert (root/'native-full/native86.json').read_bytes()==(root/'native-after-final/native86.json').read_bytes()
for name,h in raw['sourceBindings']['files'].items():assert sha(subprocess.check_output(['git','show',head+':'+name],cwd=repo))==h and rec(repo/name)['sha256']==h,name
native=(root/'native-full.log').read_text();groups=[int(n) for n in re.findall(r'^test result: ok\. (\d+) passed;',native,re.M)];assert sum(groups)==1314 and max(groups)==1247 and 'test result: FAILED' not in native
front=(root/'frontend-targeted.log').read_text();assert re.search(r'Test Files\s+3 passed \(3\)',front) and re.search(r'Tests\s+51 passed \(51\)',front)
worker=(root/'worker-focused-after.log').read_text()
def total(label):
 a=re.findall(r'^ℹ '+label+r' (\d+)$',worker,re.M);assert len(a)==1;return int(a[0])
assert (total('tests'),total('pass'),total('fail'),total('cancelled'))==(130,130,0,0)
normal=[];bad=[];case=''
for line in worker.splitlines():
 if line.startswith('✔ '):case=line
 if line.startswith('ℹ {'):
  d=json.loads(line[2:])
  if d.get('reference')=='compiler-bound Native86':normal.append(d)
  if case.startswith('✔ JC066 corrupted ') and 'invalid' in d:bad.append(d);case=''
counts={'scenarios':len(normal),'accepted':sum(d['accepted'] for d in normal),'rejected':sum(d['rejected'] for d in normal),'views':sum(d['projections'] for d in normal),'badRooms':len(bad),'http400':sum(d['http400'] for d in bad)}
assert counts=={'scenarios':19,'accepted':236,'rejected':16,'views':2104,'badRooms':87,'http400':870},counts
for d in normal:assert d['originalReceiptsAfterReopen'] and d['opaqueStatesAndJournalsEqual']
for d in bad:assert d['realD1Reopen'] and d['roomCommandsJournalUnchanged']
before=json.loads((root/'native-before/repro.json').read_text());after=json.loads((root/'native-after-final/repro.json').read_text());repros=[]
for a,b,name in zip(before['cases'],after['cases'],['target-face-up-as-hidden','target-hidden-as-face-up']):
 assert a['state']==b['state']==next(c['state'] for c in raw['invalid'] if c['name']==name)
 assert a['originalState']==b['originalState'] and a['gameAccepted'] and a['roomAccepted'] and not b['gameAccepted'] and not b['roomAccepted']
 repros.append({'name':b['name'],'revision':b['revision'],'frameId':b['frameId'],'targetId':b['targetId'],'badStateSha256':sha(b['state'].encode()),'sameExactStateBytes':True,'before':{'nativeAccepted':True,'wasmEntrypointsAccepted':6,'http200':10,'pauseCreatedCommands':1,'pauseCreatedJournal':1,'originalReceiptAfterReopen':True},'after':{'nativeGameAndRoomRejected':True,'wasmEntrypointsRejected':6,'http400':10,'zeroExtraD1Writes':True,'roomCommandsJournalUnchangedBeforeAndAfterReopen':True}})
site=json.loads((root/'official-v67-10.json').read_text());browser=json.loads((root/'v67-browser10/normal-browser.json').read_text());assert site['site']['latest_version_number']==67 and site['deployment']['status']=='succeeded' and browser['certificateBlocked'] and len(browser['requests'])==1 and browser['siteWrites']==browser['onlineGames']==0
ruling=json.loads((root/'user-project-ruling.json').read_text());assert ruling['choice']=='B' and not ruling['officialFaqClaim']
summary={'checkedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'candidateSourceCommit':head,'baseCommit':'8607a94298ed323b0375106fac7db01ffbc77ccd','productDiff':'one additional live same-UID face_down/public.kind consistency condition in jc066_target','p2ExactBeforeAfter':repros,'nativeWasm':{'compilerInputs':135,'counts':bundle['counts'],'rawSha256':bundle['raw']['sha256'],'fixture':rec(f),'wasm':rec(repo/'rust-game-wasm/pkg/hegemony_wasm_bg.wasm'),'fullNativeAndFocusedExportByteIdentical':True},'nativeWorkspace':{'tests':sum(groups),'library':1247,'integration':67,'singleNaturalRun':True},'frontend':{'files':3,'tests':51,'full1342NotRerun':True,'productionTypeScriptAndVitePassed':True,'all167ClientFilesUnchanged':True,'noNewComponentBrowserQaRequiredForGuardOnlyChange':True},'worker':{'focusedSingleRun':130,'jc066HttpTests':106,'kernelRoutingTests':18,'sourceGuards':6,'jc066':counts,'normal':normal,'badRooms':bad,'dedicatedP2Before':4,'dedicatedP2After':4,'prior306And55FileSuiteNotRerun':True,'prior306Boundary':'Prior306 first294pass/12JZ46testreferencefail, correctedJZ46fullfile15pass; coverage16files plus5files6checks; no all55files-green claim'},'selfSeal':ruling,'siteThisHour':{'official':site,'ordinaryBrowser':{'checkedUtc':browser['checkedUtc'],'certificateBlocked':True,'navigations':1,'writes':0,'onlineGames':0,'bypass':False}},'published':False,'independentReview':'pending-parent-review','runs':receipts}
out.mkdir();(out/'runs').mkdir();(out/'sources').mkdir()
for n,r in receipts.items():shutil.copy2(root/(n+'.exit.json'),out/'runs'/(n+'.exit.json'));(out/'runs'/(n+'.log.gz')).write_bytes(gzip.compress((root/(n+'.log')).read_bytes(),mtime=0))
for name in ['baseline-source-proof.json','official-v67-10.json','user-project-ruling.json']:
 shutil.copy2(root/name,out/name)
for stem in ['native-before','native-after-final']:
 (out/'sources'/(stem+'-repro.json.gz')).write_bytes(gzip.compress((root/stem/'repro.json').read_bytes(),mtime=0))
shutil.copy2(root/'native-before/jc066_tests.rs.gz',out/'sources/before-regression-jc066_tests.rs.gz')
(out/'sources/exact-original-inputs.json.gz').write_bytes(gzip.compress(json.dumps({n:(root/(n+'.room.json')).read_text() for n in ['entry-deploy-accept','actor-1']}).encode(),mtime=0))
shutil.copy2(root/'v67-browser10/normal-browser.json',out/'normal-browser10.json')
shutil.copy2(root/'audit-artifacts.py',out/'audit-artifacts.py');shutil.copy2(root/'assemble-evidence.py',out/'assemble-evidence.py')
(out/'validation-summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'source':head,'nativeTests':sum(groups),'frontendTargeted':51,'focusedWorkerTests':130,'jc066':counts,'beforeAndAfterExactStates':len(repros),'wasm':summary['nativeWasm']['wasm'],'selfSealChoice':'user project ruling B','published':False}))
