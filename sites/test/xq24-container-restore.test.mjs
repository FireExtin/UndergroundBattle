import { loadCurrentFixture } from '../../tools/current-native-source.mjs';
// Explicit Native invalid saves; actual built Worker HTTP/WASM and D1 SQLite.
// Synthetic credentials and layouts only; this is no production room migration.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { kernel as abi, moduleBytes as currentBytes } from '../../rust-game-wasm/tests/current-kernel.mjs';

test('XQ24 invalid effects saves reject at real Worker restore before HTTP pass or clock polling', async t => {
  abi.initSync({ module:currentBytes });
  const fixture=loadCurrentFixture('rust-game-wasm/tests/current112/xq24-invalid.json.gz');
  const persist=mkdtempSync(join(tmpdir(), 'xq24-container-d1-'));
  const options=convertV4MiniflareOptions({ name:'xq24-container-restore', resourcePersistencePath:persist,
    modules:[{ type:'ESModule', path:resolve('dist/server/index.js') },
      ...readdirSync('dist/server').filter(n=>n.endsWith('.wasm')).map(n=>({ type:'CompiledWasm', path:resolve('dist/server',n) }))],
    compatibilityDate:'2026-10-02', cf:false, d1Databases:{ DB:'xq24-container-db' } });
  let mf=new Miniflare(options), db=await mf.getD1Database('DB');
  t.after(async()=>{ await mf.dispose(); rmSync(persist,{ recursive:true, force:true }); });
  for (const file of readdirSync('drizzle').filter(n=>n.endsWith('.sql'))) {
    for (const sql of readFileSync('drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s=>s.trim())) await db.prepare(sql).run();
  }
  const token='a'.repeat(64); let rejectedRequests=0;
  for (const [index,c] of fixture.cases.entries()) {
    const original=JSON.parse(abi.view(c.original,0)), id=original.roomId;
    const store=new RoomStore(db);
    // Distinct synthetic DB invitations; this test never joins through them.
    if (!await store.room(id)) await store.create({ id, invite:original.inviteCode+'-'+c.mode, state:c.state,
      nonce:crypto.randomUUID(), tokenHash:await digest(token), requestHash:crypto.randomUUID(),
      intentHash:'explicit-native-invalid-container', response:'{}' });
    await db.prepare('UPDATE rooms SET state=?,version=? WHERE id=?').bind(c.state,original.version,id).run();
    await mf.dispose(); mf=new Miniflare(options); db=await mf.getD1Database('DB');
    const requests=[['state',undefined], ['commands',{ commandId:'invalid-container-'+index,
      expectedVersion:original.version, action:{ kind:'game', action:{ kind:'pass' } } }]];
    for (const [operation,body] of requests) {
      const result=await mf.dispatchFetch(`http://localhost/api/rooms/${id}/${operation}`, {
        method:body?'POST':'GET', headers:{ Authorization:'Bearer '+token,
          ...(body?{ 'Content-Type':'application/json' }:{}) },
        ...(body?{ body:JSON.stringify(body) }:{}) });
      assert.equal(result.status,400);
      assert.match((await result.json()).message,/XQ24进场响应帧不能保存于直接效果队列/);
      const saved=await new RoomStore(db).room(id);
      assert.equal(saved.state,c.state); assert.equal(saved.version,original.version);
      for (const table of ['commands','journal']) {
        const count=await db.prepare(`SELECT COUNT(*) AS n FROM ${table} WHERE room_id=?`).bind(id).first();
        assert.equal(count.n,0);
      }
      rejectedRequests++;
    }
  }
  assert.equal(rejectedRequests,8);
  t.diagnostic(JSON.stringify({ invalidNativeSaves:4, workerdD1Reopens:4,
    httpRestoreRejections:rejectedRequests, savedStateUnchanged:true, noCommandsOrJournal:true }));
});
