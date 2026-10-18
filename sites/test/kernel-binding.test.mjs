import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import * as abi from '../../rust-game-wasm/pkg/hegemony_wasm.js';
import {lazyKernel} from '../src/lazy-kernel.mjs';

const bytes=readFileSync(new URL('../../rust-game-wasm/pkg/hegemony_wasm_bg.wasm',import.meta.url));
abi.initSync({module:bytes});
const catalog=JSON.parse(abi.catalog());
const source=readFileSync(new URL('../src/kernel.mjs',import.meta.url),'utf8');
const identity=JSON.parse(source.match(/^const kernel0 = lazyKernel\(current, currentModule, (\{[^\n]+\})\);$/m)[1]);

test('fixed production Site identity agrees with the candidate source-built ABI86',()=>{
  assert.deepEqual(identity,{rulesVersion:'hegemony-pdf-v1',cardPoolVersion:'limited-v2.78-jc066-sealed-icons-candidate',engineVersion:'rust-v0.2.86-jc066-sealed-icons-candidate'});
  for(const key of ['rulesVersion','cardPoolVersion','engineVersion']) assert.equal(identity[key],catalog[key]);
  assert.equal(lazyKernel(abi,bytes,identity).catalog(),abi.catalog());
});
test('strict lazy initialization rejects a mismatching frozen identity',()=>{
  const wrong={...identity,engineVersion:'rust-v0.2.64-xq24-riot-police-candidate'};
  assert.throws(()=>lazyKernel(abi,bytes,wrong).catalog(),/Frozen kernel identity mismatch/);
});
