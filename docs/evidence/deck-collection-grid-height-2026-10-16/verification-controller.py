import datetime,hashlib,json,os,pathlib,re,signal,socket,subprocess,sys,time,urllib.request
root=pathlib.Path('/workspace/UndergroundBattle');base=pathlib.Path('/workspace/collection-layout-20261016');fixed=sys.argv[1];out=base/('verified-'+fixed[:7]+'-clean-preview');out.mkdir(exist_ok=False);results=[]
def guard():
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==fixed
 assert not subprocess.check_output(['git','status','--porcelain=v1'],cwd=root,text=True).strip()
def save():
 (out/'results.json').write_text(json.dumps({'source':fixed,'serialSinglePass':True,'checks':results,'mainPush':False,'deployment':False},indent=2)+'\n')
def run(name,cmd):
 guard();print(json.dumps({'started':name,'source':fixed,'atUtc':datetime.datetime.now(datetime.timezone.utc).isoformat()}),flush=True);start=time.monotonic();log=out/(name+'.log')
 with log.open('x') as stream:r=subprocess.run(cmd,cwd=root,stdout=stream,stderr=subprocess.STDOUT)
 raw=log.read_bytes();row={'name':name,'command':cmd,'exitCode':r.returncode,'seconds':round(time.monotonic()-start,3),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()};results.append(row);save();print(json.dumps(row),flush=True)
 if r.returncode:raise RuntimeError(name+' failed; no later validation started')
 guard()
preview=None;preview_log=None
try:
 guard()
 previous=base/('verified-'+fixed[:7]);prior=json.loads((previous/'results.json').read_text());assert prior['source']==fixed
 build=prior['checks'][0];assert build['name']=='production-build' and build['exitCode']==0
 raw=(previous/'production-build.log').read_bytes();assert hashlib.sha256(raw).hexdigest()==build['sha256']
 (out/'production-build.log').write_bytes(raw);results.append({**build,'reusedFrom':str(previous),'sourceUnchanged':True});save()
 print(json.dumps({'reusedPassedProductionBuild':fixed,'ownedBaselinePreviewNowStopped':True}),flush=True)
 probe=socket.socket();in_use=probe.connect_ex(('127.0.0.1',8090))==0;probe.close();assert not in_use,'Port 8090 unexpectedly occupied; do not duplicate or stop another process'
 preview_log=(out/'local-preview.log').open('x');env=dict(os.environ,WRANGLER_SEND_METRICS='false',XDG_CONFIG_HOME=str(base/'wrangler-config'))
 cmd=['node','node_modules/wrangler/bin/wrangler.js','dev','--local','--ip','127.0.0.1','--port','8090','--persist-to',str(base/'local-state')]
 preview=subprocess.Popen(cmd,cwd=root/'sites',env=env,stdout=preview_log,stderr=subprocess.STDOUT,start_new_session=True)
 print(json.dumps({'started':'owned-local-preview-after-build','pid':preview.pid}),flush=True)
 deadline=time.monotonic()+35;last=None;opener=urllib.request.build_opener(urllib.request.ProxyHandler({}));ready=None
 try:
  while time.monotonic()<deadline:
   if preview.poll() is not None:raise RuntimeError('Owned preview exited during startup')
   try:
    with opener.open('http://127.0.0.1:8090/',timeout=2) as response:html=response.read();assert response.status==200
    assets=re.findall(rb'(?:src|href)="(/assets/[^"]+)"',html);assert assets
    rows=[]
    for asset in assets:
     path=asset.decode()
     with opener.open('http://127.0.0.1:8090'+path,timeout=2) as response:raw=response.read();assert response.status==200
     assert hashlib.sha256(raw).hexdigest()==hashlib.sha256((root/'sites/public'/path.lstrip('/')).read_bytes()).hexdigest()
     rows.append({'path':path,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
    with opener.open('http://127.0.0.1:8090/api/catalog',timeout=2) as response:catalog=json.loads(response.read());assert response.status==200
    ready={'source':fixed,'rootStatus':200,'catalogStatus':200,'engineVersion':catalog['engineVersion'],'catalogCards':len(catalog['cards']),'assetsExactlyMatchLocalBuild':rows,'localOnly':True};break
   except Exception as error:last=type(error).__name__+': '+str(error);time.sleep(.5)
  assert ready,'Preview failed readiness: '+str(last)
  (out/'preview-readiness.json').write_text(json.dumps(ready,indent=2)+'\n');print(json.dumps({'ready':'local-preview','matchedAssets':len(ready['assetsExactlyMatchLocalBuild'])}),flush=True)
  run('production-layout-browser',['python3','tools/cloud-playtest/deck_collection_layout.py','--source',fixed,'--output',str(out/'layout-browser')])
  run('production-browser',['python3','tools/cloud-playtest/deck_collection_page.py','--source',fixed,'--output',str(out/'browser')])
 finally:
  if preview and preview.poll() is None:
   os.killpg(preview.pid,signal.SIGINT)
   try:preview.wait(timeout=12)
   except subprocess.TimeoutExpired:os.killpg(preview.pid,signal.SIGTERM);preview.wait(timeout=8)
  if preview_log:preview_log.close()
  print(json.dumps({'ownedPreviewStopped':True,'browserCompleteBeforeStop':True}),flush=True)
 run('web-full',['node','tools/build-storage.mjs','--','npm','--prefix','web','test','--','--maxWorkers=1','--reporter=default','--reporter=json','--outputFile='+str(out/'web-full-vitest.json')])
 print('All local production/UI/Web validations passed; no main push or deployment.',flush=True)
except Exception as error:
 (out/'failure.json').write_text(json.dumps({'source':fixed,'error':type(error).__name__+': '+str(error),'earlierResultsPreserved':True},indent=2)+'\n');print(str(error),flush=True);sys.exit(1)
