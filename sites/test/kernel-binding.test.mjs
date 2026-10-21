import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import * as abi from '../../rust-game-wasm/pkg/hegemony_wasm.js';
const bytes=readFileSync(new URL('../../rust-game-wasm/pkg/hegemony_wasm_bg.wasm',import.meta.url));
abi.initSync({module:bytes});

test('candidate package contains exactly the current ABI94 bytes and catalog identity',()=>{
  const catalog=JSON.parse(abi.catalog());
  assert.deepEqual([catalog.rulesVersion,catalog.cardPoolVersion,catalog.engineVersion],['hegemony-pdf-v1','limited-v2.84-batch20-third','rust-v0.2.94-batch20-third']);
  assert.equal(catalog.cards.length,212);
  assert.equal(catalog.cards.filter(c=>c.supported).length,193);
  assert.equal(catalog.cards.filter(c=>!c.supported).length,19);
  assert.deepEqual(readdirSync('generated').sort(),['hegemony_wasm.js','hegemony_wasm_bg.wasm']);
  assert.deepEqual(readFileSync('generated/hegemony_wasm_bg.wasm'),bytes);
  const modules=readdirSync('dist/server').filter(name=>name.endsWith('.wasm'));
  assert.equal(modules.length,1);
  assert.deepEqual(readFileSync('dist/server/'+modules[0]),bytes);
});
