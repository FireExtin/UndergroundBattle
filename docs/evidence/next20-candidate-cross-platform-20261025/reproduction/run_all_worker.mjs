import{readdirSync}from'node:fs';import{spawnSync}from'node:child_process';import path from'node:path';
const cwd=path.resolve('sites'),files=readdirSync(cwd+'/test').filter(f=>f.endsWith('.test.mjs')).sort().map(f=>'test/'+f);
const args=['--test','--test-concurrency=1',...files];console.log(JSON.stringify({sourceFiles:files.length,cwd,args}));const r=spawnSync(process.execPath,args,{cwd,env:process.env,stdio:'inherit'});if(r.error)throw r.error;process.exitCode=r.status??1;
