import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
const m = await import(pathToFileURL(path.resolve('tools/current-native-source.mjs')).href);
console.log(JSON.stringify({sourceManifest:m.currentSourceManifestSha256,wasm:m.currentWasmSha256}));
try {m.loadCurrentFixture('rust-game-wasm/tests/current112/batch20-review-normal.json.gz');console.log(JSON.stringify({loaded:true}));} catch(error) {console.log(JSON.stringify({loaded:false,error:String(error),actual:error.actual,expected:error.expected}));assert.equal(error.code,'ERR_ASSERTION');}
