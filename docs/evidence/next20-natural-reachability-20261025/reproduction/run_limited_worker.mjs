import {spawnSync} from 'node:child_process';
import path from 'node:path';
const args=['--test','--test-concurrency=1','test/next20-natural.test.mjs'];
console.log(JSON.stringify({scope:'Only the three new natural candidate paths; no old full suite',cwd:path.resolve('sites'),args}));
const r=spawnSync(process.execPath,args,{cwd:path.resolve('sites'),env:process.env,stdio:'inherit'});
if(r.error)throw r.error;process.exitCode=r.status??1;
