import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { frozenAbi80, stageFrozenAbi80 } from './frozen-kernels.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
test('retained ABI80 is the exact published v62 with its original 125 Native inputs',()=>{
  const saved=frozenAbi80();
  assert.equal(saved.identity.providerSource,'1f1b785df84738e6f85c1462482236b3ddabb8be');
  assert.equal(saved.identity.catalogIdentity.engineVersion,'rust-v0.2.80-wm062-conceal-adjacent-move-candidate');
  assert.equal(hash(saved.wasm),'0229f75b7af79035f2a4025fc11c6de6f3f0b895127cc8b537f829268d030472');
  assert.equal(saved.wasm.length,3047680);
  const binding=JSON.parse(readFileSync(new URL('../rust-game-wasm/frozen/abi80/native-source-binding.json',import.meta.url),'utf8'));
  assert.equal(Object.keys(binding.files).length,125);
  const dest=mkdtempSync(path.join(tmpdir(),'frozen80-stage-'));stageFrozenAbi80(dest);
  assert.deepEqual(readdirSync(dest).sort(),['hegemony_wasm.js','hegemony_wasm_bg.wasm']);
  assert.deepEqual(readFileSync(path.join(dest,'hegemony_wasm_bg.wasm')),saved.wasm);
});
test('missing or tampered ABI80 fails without migration, network or rebuild fallback',()=>{
  const saved=frozenAbi80(),empty=mkdtempSync(path.join(tmpdir(),'frozen80-missing-'));
  assert.throws(()=>frozenAbi80({directory:empty}),/Missing published ABI80/);
  writeFileSync(path.join(empty,'hegemony_wasm.js'),saved.js);
  writeFileSync(path.join(empty,'hegemony_wasm_bg.wasm.gz'),Buffer.from('different'));
  assert.throws(()=>frozenAbi80({directory:empty}),/ABI80 runtime hash mismatch/);
  assert.equal(readFileSync(path.join(empty,'hegemony_wasm_bg.wasm.gz'),'utf8'),'different');
});
