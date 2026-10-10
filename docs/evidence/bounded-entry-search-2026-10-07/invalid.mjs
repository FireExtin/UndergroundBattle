import {resolve,dirname} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
const pkg=resolve(process.env.WASM_PKG_DIR||'rust-game-wasm/pkg');
const abi=await import(pathToFileURL(resolve(pkg,'hegemony_wasm.js')).href);
import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const root=resolve(process.env.ENTRY_SEARCH_EVIDENCE_DIR||dirname(fileURLToPath(import.meta.url)))+'/';
abi.initSync({module:readFileSync(resolve(pkg,'hegemony_wasm_bg.wasm'))});
let rejected=0;for(const n of readdirSync(root+'invalid')){const s=readFileSync(root+'invalid/'+n,'utf8');assert.throws(()=>abi.view(s,0),n);assert.throws(()=>abi.applyRoom(s,0,'{"commandId":"malformed","expectedVersion":0,"action":{"kind":"pauseRoom"}}','0'),n);rejected++;}
assert.equal(rejected,192);const r={engine:'rust-v0.2.50-bounded-entry-search-candidate',invalidNativeSnapshots:rejected,viewAndCommandBothReject:true,scope:'Two fixed programs, four actors,24 malformed program/choice/library/source variants each; explicit Native test states'};writeFileSync(root+'invalid-result.json',JSON.stringify(r,null,2)+'\n');console.log(JSON.stringify(r));
