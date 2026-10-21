import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { frozenAbi77, stageFrozenAbi77, frozenAbi89 } from './frozen-kernels.mjs';
const digest = b => createHash('sha256').update(b).digest('hex');
test('retained ABI77 is the exact published v58 input rather than the evolving candidate',()=>{
  const r=frozenAbi77();assert.equal(r.identity.providerSource,'eec9ffb81b66b149bdcc4f045c1be834dae21748');
  assert.equal(r.identity.catalogIdentity.engineVersion,'rust-v0.2.77-jc046-mandatory-sacrifice-candidate');
  assert.equal(digest(r.wasm),'b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c');
});
test('missing historical inputs fail closed without rebuilding or substituting another kernel',t=>{
  const dir=mkdtempSync(join(tmpdir(),'bq035-missing77-'));t.after(()=>rmSync(dir,{recursive:true,force:true}));
  assert.throws(()=>frozenAbi77({directory:dir}),/Missing published ABI77 inputs/);
});
for(const file of ['hegemony_wasm.js','hegemony_wasm_bg.wasm.gz']) test(`tampered ${file} fails before use`,t=>{
  const r=frozenAbi77(),dir=mkdtempSync(join(tmpdir(),'bq035-invalid77-'));t.after(()=>rmSync(dir,{recursive:true,force:true}));
  writeFileSync(join(dir,'hegemony_wasm.js'),r.js);writeFileSync(join(dir,'hegemony_wasm_bg.wasm.gz'),readFileSync(join(r.directory,'hegemony_wasm_bg.wasm.gz')));
  const b=readFileSync(join(dir,file));b[0]^=1;writeFileSync(join(dir,file),b);
  assert.throws(()=>frozenAbi77({directory:dir}),/Published ABI77 runtime hash mismatch/);
});
test('build staging preserves both published runtime files exactly',t=>{
  const r=frozenAbi77(),dir=mkdtempSync(join(tmpdir(),'bq035-stage77-'));t.after(()=>rmSync(dir,{recursive:true,force:true}));
  const target=join(dir,'generated','abi77');stageFrozenAbi77(target);
  assert.deepEqual(readFileSync(join(target,'hegemony_wasm.js')),r.js);
  assert.deepEqual(readFileSync(join(target,'hegemony_wasm_bg.wasm')),r.wasm);
});

test('retained v71 ABI89 uses the recovered post-P2 bytes and rejects absent inputs', t => {
  const saved = frozenAbi89();
  assert.equal(saved.identity.providerSource, '100161b628e56b9153041d952ac0f5d83428ca88');
  assert.equal(saved.wasm.length, 3471247);
  assert.equal(digest(saved.wasm), 'df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de');
  const empty = mkdtempSync(join(tmpdir(), 'audit-missing89-'));
  t.after(() => rmSync(empty, { recursive: true, force: true }));
  assert.throws(() => frozenAbi89({ directory: empty }), /Missing published ABI89 inputs/);
});
