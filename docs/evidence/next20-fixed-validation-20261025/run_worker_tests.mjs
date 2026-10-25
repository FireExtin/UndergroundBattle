import {readdirSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
const cwd='/home/ddd/Documents/Codex/2026-10-24/task-7/validation-fixed-1bd0776-20261025/source/sites';
const tests=readdirSync(cwd+'/test').filter(f=>f.endsWith('.test.mjs')).sort().map(f=>'test/'+f);
const result=spawnSync(process.execPath,['--test','--test-concurrency=1',...tests],{cwd,env:process.env,stdio:'inherit'});
if(result.error)throw result.error;process.exitCode=result.status??1;
