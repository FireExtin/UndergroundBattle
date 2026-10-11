// Exact original native inputs: private authoritative state remains an opaque string.
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
const [bindings, modulePath, zip, manifest, output] = process.argv.slice(2);
const abi = await import(pathToFileURL(bindings));
const wasm = readFileSync(modulePath); abi.initSync({ module: wasm });
const py = spawn('python3', ['-c', `
import zipfile,json,sys,hashlib
m=json.load(open(sys.argv[2]));z=zipfile.ZipFile(sys.argv[1])
assert sorted(z.namelist())==sorted(x['path'] for x in m)
for x in m:
 b=z.read(x['path']);assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256']
 print(json.dumps({'name':x['path'],'record':json.loads(b)},separators=(',',':')))
`, zip, manifest], { stdio: ['ignore', 'pipe', 'inherit'] });
let checkpoints=0, commands=0, rejections=0, invalidStates=0, views=0;
for await (const line of createInterface({input:py.stdout,crlfDelay:Infinity})) {
  const {name,record:d}=JSON.parse(line);
  if (name.startsWith('invalid/')) {
    assert.throws(()=>abi.view(d.state,0),name);
    assert.throws(()=>abi.applyRoom(d.state,0,JSON.stringify({commandId:name,expectedVersion:0,action:{kind:'game',action:{kind:'pass'}}}),'0'),name);
    invalidStates++; continue;
  }
  let state=d.state;
  if (d.command) {
    const result=JSON.parse(abi.applyRoom(state,d.seat,JSON.stringify(d.command),'0'));
    assert.deepEqual(result,d.expected,name); state=result.state;
    if (d.expected.errorCode) rejections++; else commands++;
  } else checkpoints++;
  for (let seat=0;seat<d.views.length;seat++) { assert.deepEqual(JSON.parse(abi.view(state,seat)),d.views[seat],`${name}, seat ${seat}`);views++; }
}
const exit=py.exitCode??await new Promise(resolve=>py.on('exit',resolve));assert.equal(exit,0);
const result={passed:true,engine:JSON.parse(abi.catalog()).engineVersion,checkpoints,commands,rejections,invalidStates,views,
  originalInputsByteVerified:true,completeTransitionsEqual:true,opaqueStateBytesEqual:true,
  wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),
  scope:'Explicit native layouts and actual room commands; no natural browser play claim'};
writeFileSync(output,JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
