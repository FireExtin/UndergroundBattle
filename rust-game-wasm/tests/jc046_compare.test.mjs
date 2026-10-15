// Exercise the CLI contract: omissions must fail before a partial comparison can report success.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const script=fileURLToPath(new URL('./jc046_compare.mjs',import.meta.url));
const fixture=fileURLToPath(new URL('../../web/src/game/jc046Native77.fixture.json',import.meta.url));
const original=fileURLToPath(new URL('../../docs/evidence/jc046-2026-10-15/invalid-room/',import.meta.url));
const manifest=fileURLToPath(new URL('../../docs/evidence/jc046-2026-10-15/native-evidence-manifest.json',import.meta.url));
const names=readdirSync(original).sort();
assert.equal(names.length,55);
const compare=args=>spawnSync(process.execPath,[script,fixture,...args],{encoding:'utf8',timeout:60_000,maxBuffer:1024*1024});
function reject(args,message) {
  const result=compare(args);
  assert.ifError(result.error);
  assert.notEqual(result.status,0,result.stdout);
  assert.match(result.stderr,message);
}
function copies(t) {
  const root=mkdtempSync(join(tmpdir(),'jc046-compare-contract-'));
  t.after(()=>rmSync(root,{recursive:true,force:true})); // Only this test's new copies.
  for(const name of names)copyFileSync(join(original,name),join(root,name));
  return root;
}

test('JC046 refuses a missing invalid-state directory argument',()=>reject([],/complete-55-invalid-directory/));
test('JC046 refuses an empty invalid-state directory',t=>{
  const root=mkdtempSync(join(tmpdir(),'jc046-compare-empty-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
  reject([root],/exactly the 55 frozen files/);
});
test('JC046 refuses a 54-file subset',t=>{
  const root=copies(t);rmSync(join(root,names[0]));reject([root],/exactly the 55 frozen files/);
});
test('JC046 refuses an extra unreviewed file',t=>{
  const root=copies(t);copyFileSync(join(root,names[0]),join(root,'unreviewed.json'));
  reject([root],/exactly the 55 frozen files/);
});
test('JC046 refuses 55 files with a replaced name',t=>{
  const root=copies(t);renameSync(join(root,names[0]),join(root,'replacement.json'));
  reject([root],/exactly the 55 frozen files/);
});
test('JC046 refuses a same-length byte change even when that state is still invalid',t=>{
  const root=copies(t),path=join(root,names[0]);
  const bytes=readFileSync(path),changed=Buffer.from(bytes);changed[changed.length-1]=changed.at(-1)===10?32:10;
  writeFileSync(path,changed);reject([root],/invalid-state SHA256 differs/);
});
test('JC046 refuses a changed manifest identity',t=>{
  const root=copies(t),copy=join(root,'modified-manifest.txt');
  writeFileSync(copy,readFileSync(manifest,'utf8')+'\n');
  reject([root,copy],/manifest must match the frozen reviewed identity/);
});
test('JC046 requires the complete reviewed 55 files and executes both rejection entrypoints',()=>{
  const result=compare([original,manifest]);assert.ifError(result.error);assert.equal(result.status,0,result.stderr);
  const report=JSON.parse(result.stdout);
  assert.equal(report.invalidStates,55);assert.equal(report.invalidStateEntrypoints,110);
  assert.equal(report.scenarios,37);assert.equal(report.steps,334);assert.equal(report.seatViews,1468);
  assert.equal(report.invalidManifestSha256,'bedea2a79fc7cc98a6c1d0ed98a5d166396d638dba07060a91b6436ea5698661');
  assert.equal(report.completeNativeStatesEqual,true);assert.equal(report.previous76Rejected,true);
});
