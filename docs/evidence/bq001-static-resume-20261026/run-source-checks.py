from pathlib import Path
import subprocess,os,json,datetime,hashlib,time,sys,resource,signal
root=Path(__file__).resolve().parent;repo=Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial');run=root/sys.argv[1];run.mkdir(exist_ok=False);files=sys.argv[2:];node=repo.parent/'.tools/node-v24.22.1-linux-x64/bin/node';cmd=[str(node),'--max-old-space-size=64','--test','--test-isolation=none','--test-concurrency=1','--test-reporter=tap',*files];sha=lambda b:hashlib.sha256(b).hexdigest();sources=files+(['tools/cards/bq001-source-contract.mjs'] if (repo/'tools/cards/bq001-source-contract.mjs').exists() else [])+['rust-game/data/cards.json','rust-game/src/catalog.rs','tools/cards/import20-spec-history.mjs','docs/factions/card-specifications.json'];sources=list(dict.fromkeys(sources));snap={}
for p in sources:
 data=(repo/p).read_bytes();snap[p]=sha(data);out=run/'source-files'/p;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
allowed=os.sched_getaffinity(0);cpu=30 if 30 in allowed else min(allowed);os.sched_setaffinity(0,[cpu]);os.nice(19);r={'state':'running','atUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'HEAD':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'worktreeDiffSha256':sha(subprocess.check_output(['git','diff','HEAD'],cwd=repo)),'command':cmd,'scope':'STATIC SOURCE CHECKS ONLY; no Cargo, WASM, Worker, UI, browser, bot, server or GPU','singleCpu':cpu,'nice':os.getpriority(os.PRIO_PROCESS,0),'v8HeapLimitMiB':64,'testIsolation':'none; single-process source assertions','timeoutSeconds':60,'sourceSha256':snap};(run/'receipt.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n');start=time.monotonic()
with (run/'output.log').open('xb') as f:
 p=subprocess.Popen(cmd,cwd=repo,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
 try:code=p.wait(timeout=60)
 except subprocess.TimeoutExpired:
  os.killpg(p.pid,signal.SIGTERM)
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  code=124
r.update(state='complete',exitCode=code,seconds=round(time.monotonic()-start,3),maxChildRssKiB=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,logSha256=sha((run/'output.log').read_bytes()));(run/'receipt.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:r[k] for k in ['HEAD','exitCode','seconds','maxChildRssKiB','scope']},ensure_ascii=False));print((run/'output.log').read_text()[-11000:]);raise SystemExit(code)
