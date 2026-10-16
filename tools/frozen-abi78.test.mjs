import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { frozenAbi78, stageFrozenAbi78 } from './frozen-kernels.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
test('retained ABI78 is the exact published v60 bytes and identity, separate from ABI77',()=>{
  const saved=frozenAbi78();
  assert.equal(saved.identity.providerSource,'6d1a1fed34b5abad208488f09055554269e3df10');
  assert.equal(saved.identity.catalogIdentity.engineVersion,'rust-v0.2.78-bq035-vampire-mobility-candidate');
  assert.equal(hash(saved.wasm),'9511a72c960cf3e0fa249d62669724875d475ab245de4f9616615f38df43fd5f');
  assert.equal(saved.wasm.length,2976205);
  const dest=mkdtempSync(path.join(tmpdir(),'frozen78-stage-'));stageFrozenAbi78(dest);
  assert.deepEqual(readdirSync(dest).sort(),['hegemony_wasm.js','hegemony_wasm_bg.wasm']);
  assert.deepEqual(readFileSync(path.join(dest,'hegemony_wasm_bg.wasm')),saved.wasm);
});
test('missing or tampered historical ABI78 inputs fail without migration, network or rebuild fallback',()=>{
  const saved=frozenAbi78(),empty=mkdtempSync(path.join(tmpdir(),'frozen78-missing-'));
  assert.throws(()=>frozenAbi78({directory:empty}),/Missing published ABI78/);
  writeFileSync(path.join(empty,'hegemony_wasm.js'),saved.js);
  writeFileSync(path.join(empty,'hegemony_wasm_bg.wasm.gz'),Buffer.from('different'));
  assert.throws(()=>frozenAbi78({directory:empty}),/ABI78 runtime hash mismatch/);
  assert.equal(readFileSync(path.join(empty,'hegemony_wasm_bg.wasm.gz'),'utf8'),'different');
});
