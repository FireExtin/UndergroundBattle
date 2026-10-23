// Keep original old evidence opaque. This test does not import an old runtime,
// rewrite versions, migrate state or read any production database.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import {harness} from './xq23-http-harness.mjs';
const wrapped=JSON.parse(gunzipSync(readFileSync(new URL('../../docs/evidence/batch20-third-2026-10-21/abi94-twenty-inspections.current.json.gz',import.meta.url))));
const original=JSON.parse(wrapped.raw.json);assert.equal(original.catalog.engineVersion,'rust-v0.2.94-batch20-third');
test('single ABI95 Worker rejects original ABI94 opaque room with410 and preserves D1',async t=>{
 const id='95000000000000000000ffff';
 // The local routing key is test metadata; the original opaque state bytes,
 // including its historical internal ID and u64, are never changed.
 const views=original.initial.views.map(v=>({...v,roomId:id}));
 const h=await harness(t,{initial:{state:original.initial.state,views}}),before=await h.snapshot();
 assert.equal(before.room.state,original.initial.state);
 for(const [operation,body]of [['state'],['catalog'],['commands',{commandId:'retired-no-execution',expectedVersion:views[0].version,action:{kind:'pauseRoom'}}]]){
  const r=await h.api(operation,0,body);assert.equal(r.status,410);assert.equal(r.body.error,'unsupported_room_version');assert.deepEqual(await h.snapshot(),before);
 }
 await h.reopen();const r=await h.api('state',0);assert.equal(r.status,410);assert.equal(r.body.error,'unsupported_room_version');assert.deepEqual(await h.snapshot(),before);
 assert.equal(before.commands.length,0);assert.equal(before.journal.length,0);
 t.diagnostic(JSON.stringify({currentOnly:true,originalAbi94:true,http410:4,reopened:true,opaqueBytesUnchanged:true,commandsJournalUnchanged:true,migration:false}));
});
