from pathlib import Path
import os,subprocess,signal,json,time,urllib.request,urllib.parse,sys
root=Path(__file__).absolute().parent;repo=Path.cwd();assert repo.name=='UndergroundBattle-batch3';label=sys.argv[1] if len(sys.argv)>1 else '02';assert label.isdigit();state=root/f'batch3-build/qa-abi94-start-20261021-{label}';assert not state.exists();log=root/f'batch3-abi94-local-start-{label}.log';report=root/f'batch3-abi94-local-start-{label}.json';assert not log.exists() and not report.exists()
env=dict(os.environ);env.update(PATH=str(root/'.tools/node-v24.22.1-linux-x64/bin')+':'+env['PATH'],PORT='0',HOST='127.0.0.1',HEGEMONY_QA_STATE_DIR=str(state),WRANGLER_SEND_METRICS='false')
handle=log.open('w');p=subprocess.Popen(['node','scripts/qa-preview.mjs'],cwd=repo/'sites',env=env,stdout=handle,stderr=subprocess.STDOUT,text=True)
url=None;engine=None;started=time.monotonic()
try:
 while time.monotonic()-started<40:
  for line in log.read_text().splitlines():
   if line.startswith('READY '):url=line.split(' ',1)[1]
   if line.startswith('ENGINE '):engine=line.split(' | ',1)[0].removeprefix('ENGINE ')
  if url and engine:break
  if p.poll() is not None:break
  time.sleep(0.1)
 assert url and engine=='rust-v0.2.94-batch20-third','Local startup identity not confirmed'
 assert urllib.parse.urlparse(url).hostname=='127.0.0.1'
 with urllib.request.urlopen(url,timeout=5) as r:html=r.read().decode();status=r.status
 assert status==200 and 'id="root"' in html
 result={'engineVersion':engine,'localUrl':url,'healthAndExactCatalog':'passed by existing qa-preview.mjs','rootHtmlStatus':status,'singleCurrentKernel':True,'syntheticD1':str(state),'publicAdmissions':0,'scope':'Local startup/identity/connectivity only; no public Site or browser gameplay acceptance'}
finally:
 if p.poll() is None:p.send_signal(signal.SIGINT)
 try:p.wait(timeout=10)
 except subprocess.TimeoutExpired:p.send_signal(signal.SIGTERM);p.wait(timeout=10)
 handle.close()
result['exitCode']=p.returncode;result['shutdown']='cleanExit' if p.returncode==0 else 'requestedSIGINT';report.write_text(json.dumps(result,indent=2)+'\n');assert p.returncode in (0,-signal.SIGINT,128+signal.SIGINT),'Unexpected preview process termination';print(json.dumps(result))
