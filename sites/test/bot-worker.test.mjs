// Run against the ordinary, current-only Worker bundle, never a deployed site.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { restoreBotPolicy } from '../src/bot-policy.mjs';

const project = fileURLToPath(new URL('../', import.meta.url));
const worker = path.join(project, 'dist/server');
const hash = value => createHash('sha256').update(value).digest('hex');

test('bundled Worker defaults off, isolates practice mode, and restores its server-owned Bot metadata', async t => {
  const wasm = readdirSync(worker).filter(f => f.endsWith('.wasm'));
  assert.equal(wasm.length, 1);
  assert.equal(hash(readFileSync(path.join(worker, wasm[0]))), '5ad7e479413a80e27fb57b06cfab87af7ac266bbb9c1a5e76a7410d742f5d481');
  const persistence = mkdtempSync(path.join(tmpdir(), 'bot-worker-'));
  const options = enabled => convertV4MiniflareOptions({ name:'bot-worker', resourcePersistencePath:persistence,
    modulesRoot:worker, modules:[{type:'ESModule',path:path.join(worker,'index.js')},
      {type:'CompiledWasm',path:path.join(worker,wasm[0])}], compatibilityDate:'2026-10-02', cf:false,
    bindings:enabled ? {BOT_PRACTICE_ENABLED:'true'} : {}, d1Databases:{DB:'bot-worker'} });
  let mf = new Miniflare(options(false)), db = await mf.getD1Database('DB');
  t.after(() => mf.dispose());
  for (const file of readdirSync(project+'drizzle').filter(f => f.endsWith('.sql'))) {
    for (const sql of readFileSync(project+'drizzle/'+file,'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  }
  const reopen = async enabled => { await mf.dispose(); mf = new Miniflare(options(enabled)); db = await mf.getD1Database('DB'); };
  const api = async (url, body, session) => {
    const response = await mf.dispatchFetch('http://localhost'+url, {method:body?'POST':'GET', headers:{
      ...(body?{'Content-Type':'application/json'}:{}), ...(session?{Authorization:'Bearer '+session.token}:{})},
      ...(body?{body:JSON.stringify(body)}:{})});
    return {status:response.status, body:response.status===204?null:await response.json()};
  };
  const create = extra => api('/api/rooms',{requestId:crypto.randomUUID(),name:'本机练习',mode:'duel',deckId:'watchers',...extra});
  const policy = async id => restoreBotPolicy((await new RoomStore(db).botState(id)).current);
  const botCount = async id => (await db.prepare("SELECT count(*) AS n FROM commands WHERE room_id=? AND command_id LIKE 'bot:%'").bind(id).first()).n;

  assert.equal((await api('/api/catalog')).body.botPractice, undefined);
  assert.equal((await create({roomKind:'bot',difficulty:'hard'})).status,400);
  await reopen(true);
  assert.deepEqual((await api('/api/catalog')).body.botPractice.hardAdvantages,['peekHumanHands']);
  const fair = await create({}); assert.equal(fair.status,200); assert.equal(fair.body.view.botMatch,undefined);
  assert.equal(fair.body.view.players.length,1); assert.equal((await new RoomStore(db).botState(fair.body.roomId)).current,null);
  assert.equal((await create({roomKind:'human',difficulty:'hard'})).status,400);

  const created = await create({roomKind:'bot',difficulty:'hard'}); assert.equal(created.status,200);
  const session = created.body;
  assert.equal(session.view.botMatch.difficulty,'hard'); assert.equal(session.view.botMatch.enabled,true);
  assert.deepEqual(session.view.botMatch.advantages,['peekHumanHands']);
  assert.equal(session.view.players.length,2); assert.equal(session.view.you,'p0');
  let view = session.view;
  const command = async (commandId, action) => {
    const payload={commandId,expectedVersion:view.version,action};
    const response=await api(`/api/rooms/${session.roomId}/commands`,payload,session);
    assert.equal(response.status,200); view=response.body; return payload;
  };
  const ready = await command('human-ready',{kind:'game',action:{kind:'ready'}});
  await command('human-start',{kind:'game',action:{kind:'start'}});
  for (let n=0;n<24;n++) {
    const response=await api(`/api/rooms/${session.roomId}/state`,null,session); assert.equal(response.status,200); view=response.body;
    if ((await botCount(session.roomId))>0) break;
    // The human must finish their own mulligan before the Bot receives a choice.
    if (view.pendingChoice?.playerId===view.you && view.pendingChoice.kind==='mulligan') {
      await command(`human-mulligan-${n}`,{kind:'game',action:{kind:'choose',choiceId:view.pendingChoice.id,selected:[]}});
    }
  }
  assert((await botCount(session.roomId))>0);
  const before=await policy(session.roomId), decisions=await botCount(session.roomId);
  assert.equal(typeof before.seed,'string'); assert.equal(BigInt(before.counter),BigInt(decisions));
  assert.equal(JSON.stringify(view.botMatch).includes(before.seed),false);
  await reopen(false);
  const recovered=await api(`/api/rooms/${session.roomId}/state`,null,session);
  assert.equal(recovered.status,200); assert.equal(recovered.body.botMatch.enabled,false);
  assert.equal(recovered.body.botMatch.difficulty,'hard'); assert.equal(await botCount(session.roomId),decisions);
  assert.deepEqual(await policy(session.roomId),before);
  const rejectedJoin=await api('/api/rooms/join',{requestId:crypto.randomUUID(),name:'人类',inviteCode:session.inviteCode,deckId:'watchers'});
  assert.equal(rejectedJoin.status,409);
  await reopen(true);
  const retry=await api(`/api/rooms/${session.roomId}/commands`,ready,session);
  assert.equal(retry.status,200); assert.equal(retry.body.version,ready.expectedVersion+1);
  assert.equal(await botCount(session.roomId),decisions);
  assert.equal((await api(`/api/rooms/${fair.body.roomId}/state`,null,fair.body)).body.botMatch,undefined);
  t.diagnostic(JSON.stringify({scope:'author local bundled Worker/D1',wasmModules:1,durableDifficulty:'hard',botCommands:decisions,defaultOff:true,reopenedTwice:true,deployed:false}));
});
