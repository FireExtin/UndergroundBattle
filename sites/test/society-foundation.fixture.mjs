// Explicit local fixture-feature Worker. No public rooms, browsers or printed society admission.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';

test('fixture-only society foundation follows four-seat APIs, opaque D1 reopen, original receipts and natural reset', async t => {
  const dist = process.env.HEGEMONY_SOCIETY_WORKER_DIST;
  assert(dist, 'Supply the isolated fixture-feature backend dry-run output.');
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-society-fixture-'));
  const options = convertV4MiniflareOptions({ name: 'society-foundation-fixture', modulesRoot: resolve(dist), resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve(dist, 'index.js') },
    ...readdirSync(dist).filter(n => n.endsWith('.wasm')).map(n => ({ type: 'CompiledWasm', path: resolve(dist, n) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'society-foundation-fixture-db' } });
  let mf = new Miniflare(options); t.after(() => mf.dispose());
  let db = await mf.getD1Database('DB');
  for (const name of readdirSync('drizzle').filter(n => n.endsWith('.sql'))) for (const sql of readFileSync('drizzle/' + name, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  let calls = 0, commands = 0;
  const api = async (path, body, session) => {
    calls++;
    const r = await mf.dispatchFetch('http://localhost' + path, { method: body ? 'POST' : 'GET', headers: { ...(body ? { 'Content-Type': 'application/json' } : {}), ...(session ? { Authorization: 'Bearer ' + session.token } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    return { status: r.status, body: r.status === 204 ? null : await r.json() };
  };
  const cat = (await api('/api/catalog')).body;
  assert.equal(cat.engineVersion, 'rust-v0.2.45-jc089-poison-blood-fixture'); assert.equal(cat.cardPoolVersion, 'limited-v2.42-jc089-poison-blood-candidate'); assert.equal(cat.cards.length, 103); assert.equal(cat.societies.length, 3);
  assert(cat.societies.every(s => s.id.startsWith('FIXTURE_'))); assert(!cat.societies.some(s => s.id === 'MSJC09'));
  const draft = societyId => ({ id: 'society-internal-deck', name: '内部fixture牌组', description: '', societyId, cards: [{ cardId: 'JC125', count: 50 }], rulesVersion: cat.rulesVersion, cardPoolVersion: cat.cardPoolVersion, engineVersion: cat.engineVersion, updatedAt: '' });
  for (const d of [{ ...draft('MSJC09') }, { ...draft('FIXTURE_SOCIETY_SIX'), cards: [{ cardId: 'JC125', count: 49 }] }]) assert.equal((await api('/api/rooms', { name: '非法', mode: 'teams', deckDraft: d, requestId: crypto.randomUUID() })).status, 400);
  const originalCreate = { name: 'A', mode: 'teams', deckDraft: draft('FIXTURE_SOCIETY_SIX'), requestId: crypto.randomUUID() };
  const created = await api('/api/rooms', originalCreate); assert.equal(created.status, 200); assert.deepEqual(await api('/api/rooms', originalCreate), created);
  assert.equal((await api('/api/rooms', { ...originalCreate, deckDraft: draft(null) })).status, 409);
  const sessions = [created.body];
  for (const [name, societyId] of [['B', null], ['C', 'FIXTURE_SOCIETY_FOUR'], ['D', 'FIXTURE_SOCIETY_PENDING']]) {
    const joined = await api('/api/rooms/join', { inviteCode: created.body.inviteCode, name, deckDraft: draft(societyId), requestId: crypto.randomUUID() }); assert.equal(joined.status, 200); sessions.push(joined.body);
  }
  const url = (s, kind) => `/api/rooms/${s.roomId}/${kind}`;
  const views = async () => Promise.all(sessions.map(async s => { const r = await api(url(s, 'state'), null, s); assert.equal(r.status, 200); return r.body; }));
  const send = async (seat, action, expectedVersion) => {
    const v = expectedVersion ?? (await views())[seat].version;
    const command = { commandId: crypto.randomUUID(), expectedVersion: v, action };
    const result = await api(url(sessions[seat], 'commands'), command, sessions[seat]); assert.equal(result.status, 200, JSON.stringify(result.body)); commands++; return { result, command };
  };
  let all = await views();
  for (let s = 0; s < 4; s++) { assert.equal(all[s].societyZones.length, 4); assert(all[s].societyZones.every(z => z.card === null)); assert.equal(all[s].yourDeck.societyId, draft([ 'FIXTURE_SOCIETY_SIX', null, 'FIXTURE_SOCIETY_FOUR', 'FIXTURE_SOCIETY_PENDING' ][s]).societyId); }
  for (let s = 0; s < 4; s++) await send(s, { kind: 'game', action: { kind: 'ready' } });
  await send(0, { kind: 'game', action: { kind: 'start' } });
  all = await views(); assert.deepEqual(all[0].players.map(p => p.handCount), [6, 6, 4, 6]); assert.deepEqual(all[0].players.map(p => p.deckCount), [44, 44, 46, 44]);
  const source = all[0].societyZones[0].card.instanceId, pending = all[0].societyZones[3].card.instanceId;
  let paid, paidCommand, afterPaymentState, responseRestoredBeforeExpiry = false, reopened = false;
  for (let n = 0; n < 1000; n++) {
    all = await views();
    for (let s = 0; s < 4; s++) {
      assert(all[s].hand.every(c => c.owner === `p${s}`)); assert.equal(all[s].societyZones[1].card, null);
      assert(all[s].societyZones.filter(z => z.card).every(z => !z.card.faceDown && z.card.kind === 'society' && z.card.region === undefined));
      assert(!all[s].legalActions.some(a => a.cardId === pending));
      const others = all.filter((_, i) => i !== s).flatMap(v => v.hand.map(c => c.instanceId));
      assert(!others.some(id => all[s].pendingChoice?.options.some(o => o.card?.instanceId === id)));
    }
    if (paid && all[0].turn >= 2) break;
    const choosing = all.findIndex(v => v.pendingChoice);
    if (choosing >= 0) { const c = all[choosing].pendingChoice; await send(choosing, { kind: 'game', action: { kind: 'choose', choiceId: c.id, selected: c.allowDecline ? [] : c.options.slice(0, c.min ?? 1).map(o => o.id) } }, all[choosing].version); continue; }
    if (!paid) {
      const activation = all[0].legalActions.find(a => a.kind === 'activate' && a.cardId === source);
      if (activation) {
        assert.equal(activation.sourceZoneId, 'society:p0'); assert.equal(activation.region, undefined);
        const before = all[0].hand.length; const action = { kind: 'activate', cardId: source, abilityId: activation.abilityId };
        const sent = await send(0, { kind: 'game', action }, all[0].version); paid = sent.result; paidCommand = sent.command;
        assert(paid.body.societyZones[0].card.exhausted); assert.equal(paid.body.hand.length, before); assert(paid.body.assets.some(a => a.exhausted));
        assert(paid.body.responseWindow);
        // A normal undecided response window keeps its original five-second
        // deadlines through reopen; this deck has no legal composing response.
        afterPaymentState = (await new RoomStore(db).room(sessions[0].roomId)).state;
        await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB'); reopened = true;
        assert.equal((await new RoomStore(db).room(sessions[0].roomId)).state, afterPaymentState);
        assert.deepEqual(await api(url(sessions[0], 'commands'), paidCommand, sessions[0]), paid);
        assert.equal((await api(url(sessions[0], 'commands'), { ...paidCommand, action: { kind: 'game', action: { ...action, abilityId: 'different' } } }, sessions[0])).status, 409);
        all = await views(); responseRestoredBeforeExpiry = !!all[0].responseWindow;
        if (responseRestoredBeforeExpiry) {
          assert.equal(all[0].responseWindow.id, paid.body.responseWindow.id);
          assert.deepEqual(all[0].responseWindow.members, paid.body.responseWindow.members);
          assert.equal(all[0].hand.length, before);
        } else {
          assert.equal(all[0].hand.length, before + 1); // Legitimate elapsed deadline, one effect.
        }
        assert(all.every(v => v.societyZones[0].card.exhausted));
        continue;
      }
      const asset = all[0].legalActions.find(a => a.kind === 'asset');
      if (asset) { await send(0, { kind: 'game', action: { kind: 'asset', cardId: asset.cardId } }, all[0].version); continue; }
    }
    let passed = false;
    for (let s = 0; s < 4; s++) {
      const v = all[s], member = v.responseWindow?.members.find(m => m.playerId === `p${s}`);
      if (member?.status === 'composing') { await send(s, { kind: 'cancelAndPass', windowId: v.responseWindow.id, intentId: v.responseWindow.myIntentId }, v.version); passed = true; break; }
      if (v.legalActions.some(a => a.kind === 'pass')) { await send(s, v.responseWindow ? { kind: 'passResponse', windowId: v.responseWindow.id } : { kind: 'game', action: { kind: 'pass' } }, v.version); passed = true; break; }
    }
    assert(passed, 'normal pass must advance this bounded game');
  }
  assert(paid && reopened && all[0].turn >= 2); assert.equal(all[0].societyZones[0].card.instanceId, source); assert(!all[0].societyZones[0].card.exhausted);
  assert.deepEqual(await api(url(sessions[0], 'commands'), paidCommand, sessions[0]), paid);
  const finalOpaque = (await new RoomStore(db).room(sessions[0].roomId)).state;
  assert(!finalOpaque.includes('MSJC09')); assert((await new RoomStore(db).room(sessions[0].roomId)).initial_state.includes('FIXTURE_SOCIETY_SIX'));
  t.diagnostic(JSON.stringify({ fixtureRegistryOnly: true, realPrintedSocietiesOpened: false, publicUiAcceptance: false, httpCalls: calls, acceptedCommands: commands, fourSeats: true, reopenedAfterPayment: true, responseRestoredBeforeExpiry, naturalNextTurnReset: true }));
});
