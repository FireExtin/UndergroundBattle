import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { frozenAbi77, stageFrozenAbi77, frozenAbi89, frozenAbi91, stageFrozenAbi91, frozenAbi92, stageFrozenAbi92 } from './frozen-kernels.mjs';
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

test('v73 ABI91 retains the reward-fixed bytes and catalog for staging', async t => {
  const saved = frozenAbi91();
  assert.equal(saved.identity.siteSource, 'e1969c3ded2ffa44ce429be87c0005d928d5ef38');
  assert.equal(saved.identity.catalogIdentity.engineVersion, 'rust-v0.2.91-batch20');
  assert.equal(saved.wasm.length, 3796219);
  assert.equal(digest(saved.wasm), 'b6782cdf002238f88ce4452a0864ed485abac05ea0f9ec791ee583e1b5818376');
  assert.notEqual(digest(saved.wasm), 'b3ece039dbe2a9cb16ef8dd137c7c478234bd5898cc48214c7e5450b79734da4');
  const abi = await import('data:text/javascript;base64,' + saved.js.toString('base64'));
  abi.initSync({ module: saved.wasm });
  const catalog = JSON.parse(abi.catalog());
  assert.deepEqual({ rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion,
    engineVersion: catalog.engineVersion }, saved.identity.catalogIdentity);
  assert.equal(catalog.cards.length, 173);
  assert.equal(catalog.societies.length, 10);
  const dir = mkdtempSync(join(tmpdir(), 'frozen91-stage-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  stageFrozenAbi91(dir);
  assert.deepEqual(readFileSync(join(dir, 'hegemony_wasm.js')), saved.js);
  assert.deepEqual(readFileSync(join(dir, 'hegemony_wasm_bg.wasm')), saved.wasm);
});

test('absent or corrupt ABI91 fails without modifying inputs or using the current core', t => {
  const saved = frozenAbi91(), dir = mkdtempSync(join(tmpdir(), 'frozen91-invalid-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  assert.throws(() => frozenAbi91({ directory: dir }), /Missing published ABI91 inputs/);
  writeFileSync(join(dir, 'hegemony_wasm.js'), saved.js);
  writeFileSync(join(dir, 'hegemony_wasm_bg.wasm.gz'), 'not the original v73 runtime');
  assert.throws(() => frozenAbi91({ directory: dir }), /Published ABI91 runtime hash mismatch/);
  assert.equal(readFileSync(join(dir, 'hegemony_wasm_bg.wasm.gz'), 'utf8'), 'not the original v73 runtime');
});

test('saved v74 ABI92 stages the exact originals with explicit pending publication scope', async t => {
  const saved = frozenAbi92();
  assert.equal(saved.identity.siteSource, 'fe182e656f35153be9225c0b17fbce708831b1ba');
  assert.equal(saved.identity.siteVersion, 74);
  assert.equal(saved.identity.deploymentStatus, 'pending');
  assert.equal(saved.identity.publicationConfirmed, false);
  assert.match(saved.identity.scope, /saved v74.*deployment is pending.*not confirmed published/);
  assert.equal(saved.identity.catalogIdentity.engineVersion, 'rust-v0.2.92-society-zone-fix');
  assert.equal(saved.wasm.length, 3796247);
  assert.equal(saved.js.length, 18203);
  assert.equal(digest(saved.wasm), '03c222b1da350af75ef3ba333771b57aeeb0c77a52c721490331f78ad6d9706f');
  assert.equal(digest(saved.js), 'fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb');
  assert.equal(digest(readFileSync(join(saved.directory, 'hegemony_wasm_bg.wasm.gz'))), '7823451d0a38d1037a54ef6df7c91ef2f3e6c60e6638358c512048e7aab58e96');
  const abi = await import('data:text/javascript;base64,' + saved.js.toString('base64') + '#abi92');
  abi.initSync({ module: saved.wasm });
  const catalog = JSON.parse(abi.catalog());
  assert.deepEqual({ rulesVersion: catalog.rulesVersion, cardPoolVersion: catalog.cardPoolVersion,
    engineVersion: catalog.engineVersion }, saved.identity.catalogIdentity);
  assert.equal(catalog.cards.length, 173);
  assert.equal(catalog.societies.length, 10);
  const binding = JSON.parse(readFileSync(new URL('../rust-game-wasm/frozen/abi92/native-source-binding.json', import.meta.url), 'utf8'));
  assert.equal(binding.sourceCommit, 'eb64382604b4a6e1291247d31c6d185d0f3fb56c');
  assert.equal(binding.runtimeSourceCommit, '5d89673b64b76fd2dfe359b8467f7587d0d8d512');
  assert.equal(Object.keys(binding.files).length, 151);
  assert.match(binding.scope, /source inventory, not a new compiler trace/);
  const dir = mkdtempSync(join(tmpdir(), 'frozen92-stage-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  stageFrozenAbi92(dir);
  assert.deepEqual(readFileSync(join(dir, 'hegemony_wasm.js')), saved.js);
  assert.deepEqual(readFileSync(join(dir, 'hegemony_wasm_bg.wasm')), saved.wasm);
});

test('absent ABI92 fails without substituting the current or ABI91 core', t => {
  const dir = mkdtempSync(join(tmpdir(), 'frozen92-missing-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  assert.throws(() => frozenAbi92({ directory: dir }), /Missing retained ABI92 inputs/);
});

for (const file of ['hegemony_wasm.js', 'hegemony_wasm_bg.wasm.gz']) test(`corrupt saved ABI92 ${file} fails and leaves inputs intact`, t => {
  const saved = frozenAbi92(), dir = mkdtempSync(join(tmpdir(), 'frozen92-invalid-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  writeFileSync(join(dir, 'hegemony_wasm.js'), saved.js);
  writeFileSync(join(dir, 'hegemony_wasm_bg.wasm.gz'), readFileSync(join(saved.directory, 'hegemony_wasm_bg.wasm.gz')));
  const corrupt = readFileSync(join(dir, file));
  corrupt[0] ^= 1;
  writeFileSync(join(dir, file), corrupt);
  assert.throws(() => frozenAbi92({ directory: dir }), /Retained ABI92 runtime hash mismatch/);
  assert.deepEqual(readFileSync(join(dir, file)), corrupt);
});
