import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { frozenAbi79, stageFrozenAbi79 } from './frozen-kernels.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
test('retained ABI79 is the exact published v61 bytes and identity, separate from ABI77',()=>{
  const saved=frozenAbi79();
  assert.equal(saved.identity.providerSource,'906a1b4c54e43eff5dd722c275d55ad567b8c20c');
  assert.equal(saved.identity.catalogIdentity.engineVersion,'rust-v0.2.79-jz46-private-criminal-inspection-candidate');
  assert.equal(hash(saved.wasm),'8fdc829ab25d0eb5d68d973489fafff6b70e0da8b7837621a3ef86d52360f0fd');
  assert.equal(saved.wasm.length,3005332);
  const dest=mkdtempSync(path.join(tmpdir(),'frozen79-stage-'));stageFrozenAbi79(dest);
  assert.deepEqual(readdirSync(dest).sort(),['hegemony_wasm.js','hegemony_wasm_bg.wasm']);
  assert.deepEqual(readFileSync(path.join(dest,'hegemony_wasm_bg.wasm')),saved.wasm);
});
test('missing or tampered historical ABI79 inputs fail without migration, network or rebuild fallback',()=>{
  const saved=frozenAbi79(),empty=mkdtempSync(path.join(tmpdir(),'frozen79-missing-'));
  assert.throws(()=>frozenAbi79({directory:empty}),/Missing published ABI79/);
  writeFileSync(path.join(empty,'hegemony_wasm.js'),saved.js);
  writeFileSync(path.join(empty,'hegemony_wasm_bg.wasm.gz'),Buffer.from('different'));
  assert.throws(()=>frozenAbi79({directory:empty}),/ABI79 runtime hash mismatch/);
  assert.equal(readFileSync(path.join(empty,'hegemony_wasm_bg.wasm.gz'),'utf8'),'different');
});
