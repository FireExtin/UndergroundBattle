// Local candidate only: explicit module path, never load/overwrite release pkg.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {resolve,dirname,join} from 'node:path';
const [abiPath,nativePath,reviewPath,outPath]=process.argv.slice(2);
assert(abiPath&&nativePath&&reviewPath&&outPath);
const kernel=await import(pathToFileURL(resolve(abiPath)).href);
const bytes=readFileSync(join(dirname(resolve(abiPath)),'hegemony_wasm_bg.wasm'));
kernel.initSync({module:bytes});
const catalog=JSON.parse(kernel.catalog());
assert(catalog.societies.some(s=>s.id==='MSJC14'));
assert.equal(catalog.cards.find(c=>c.id==='WM044').supported,false);
let commands=0,views=0,polls=0,duplicates=0;
for(const scenario of JSON.parse(readFileSync(nativePath,'utf8'))){
 for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(scenario.initial,seat)),scenario.initialViews[seat]);views++;}
 for(const row of scenario.steps){
  const actual=JSON.parse(kernel.applyRoom(row.before,row.seat,JSON.stringify(row.command),row.now));
  assert.deepEqual(actual,row.expected,scenario.name);commands++;
  for(let seat=0;seat<4;seat++){
   assert.deepEqual(JSON.parse(kernel.view(actual.state,seat)),row.views[seat]);views++;
   const poll=JSON.parse(kernel.pollRoom(actual.state,seat,row.now));
   assert.equal(poll.state,actual.state);assert.deepEqual(poll.view,row.views[seat]);polls++;
  }
  const duplicate=JSON.parse(kernel.applyRoom(actual.state,row.seat,JSON.stringify(row.command),row.now));
  assert.deepEqual(duplicate,row.duplicate);assert.equal(duplicate.state,actual.state);duplicates++;
 }
}
// Reviewer exported JSON objects, so compare their state integers in Python.
// JS receives only opaque strings and never parses or rewrites private state.
const review=JSON.parse(readFileSync(reviewPath,'utf8'));let state=review.initial;const reviewStates=[];
for(const row of review.steps){
 const actual=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.now));
 assert.equal(actual.errorCode,undefined);
 reviewStates.push({actual:actual.state,expected:row.expectedState});state=actual.state;
 const duplicate=JSON.parse(kernel.applyRoom(state,row.seat,JSON.stringify(row.command),row.now));
 assert.equal(duplicate.outcome,'rejected');assert.equal(duplicate.state,state);duplicates++;
}
writeFileSync(outPath,JSON.stringify({scope:'Local candidate author WASM checks; not released ABI95/96, Worker, browser or full-match acceptance',wasmSha256:createHash('sha256').update(bytes).digest('hex'),wasmBytes:bytes.length,nativeExactCommands:commands,nativeExactViews:views,nativeExactPolls:polls,duplicates,reviewCommands:reviewStates.length,reviewStates},null,2),{flag:'wx'});
console.log(JSON.stringify({commands,views,polls,duplicates,reviewCommands:reviewStates.length}));
