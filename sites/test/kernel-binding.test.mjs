import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import * as abi from '../../rust-game-wasm/pkg/hegemony_wasm.js';
const bytes=readFileSync(new URL('../../rust-game-wasm/pkg/hegemony_wasm_bg.wasm',import.meta.url));
abi.initSync({module:bytes});

test('candidate package contains exactly the current ABI112 bytes and catalog identity',()=>{
  const catalog=JSON.parse(abi.catalog());
  assert.deepEqual([catalog.rulesVersion,catalog.cardPoolVersion,catalog.engineVersion],['hegemony-pdf-v1','limited-v2.103-next20-three-admitted','rust-v0.2.112-shared-keyword-rewards']);
  assert.equal(catalog.cards.length,277);
  for(const id of ['BQ033','BQ009'])assert.equal(catalog.cards.find(card=>card.id===id).supported,false);
  assert.equal(catalog.cards.filter(c=>c.supported).length,236);
  assert.equal(catalog.cards.filter(c=>!c.supported).length,44);
  assert.equal(catalog.cards.find(c=>c.id==='JZ56').supported,false);
  assert(catalog.cards.some(c=>c.id==='CQ07'&&c.supported));
  assert(!catalog.cards.some(c=>c.id==='CQ11'||c.id==='TK008'));
  assert.deepEqual(readdirSync('generated').sort(),['hegemony_wasm.js','hegemony_wasm_bg.wasm']);
  assert.deepEqual(readFileSync('generated/hegemony_wasm_bg.wasm'),bytes);
  const modules=readdirSync('dist/server').filter(name=>name.endsWith('.wasm'));
  assert.equal(modules.length,1);
  assert.deepEqual(readFileSync('dist/server/'+modules[0]),bytes);
});
