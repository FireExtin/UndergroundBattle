import {spawnSync} from 'node:child_process';
import path from 'node:path';
const files=['integration.test.mjs','jc035-entry.test.mjs','jc041-entry.test.mjs','jc044-entry.test.mjs','jc057-entry.test.mjs','jc066-entry.test.mjs','kernel-binding.test.mjs','msjc0405-entry.test.mjs'].map(f=>'test/'+f);
const command=[process.execPath,'--test','--test-reporter=tap','--test-concurrency=1',...files];
console.log(JSON.stringify({cwd:path.resolve('sites'),command,CURRENT_CLOCK_WORKER:process.env.CURRENT_CLOCK_WORKER}));
const result=spawnSync(command[0],command.slice(1),{cwd:path.resolve('sites'),env:process.env,stdio:'inherit'});if(result.error)throw result.error;process.exitCode=result.status??1;
