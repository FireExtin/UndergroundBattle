import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';
import { digest } from '../src/service.mjs';
import { initSync, newGame } from '../generated/hegemony_wasm.js';
import * as legacy from '../generated/legacy-v0.2.1/hegemony_wasm.js';
import * as intermediate from '../generated/legacy-v0.2.2/hegemony_wasm.js';
import * as last from '../generated/legacy-v0.2.3/hegemony_wasm.js';
import * as stable from '../generated/legacy-v0.2.4/hegemony_wasm.js';
import * as paced from '../generated/legacy-v0.2.5/hegemony_wasm.js';
import * as attached from '../generated/legacy-v0.2.6/hegemony_wasm.js';
import * as grave from '../generated/legacy-v0.2.7/hegemony_wasm.js';
import * as playable from '../generated/legacy-v0.2.8/hegemony_wasm.js';
import * as society from '../generated/legacy-v0.2.9/hegemony_wasm.js';
import * as forceMage from '../generated/legacy-v0.2.10/hegemony_wasm.js';

test('real workerd/WASM with D1 SQLite: concurrency, receipts, rollback and reopen', async t => {
  const persist = mkdtempSync(join(tmpdir(), 'hegemony-worker-d1-'));
  const options = convertV4MiniflareOptions({ name: 'hegemony-integration', resourcePersistencePath: persist, modules: [
    { type: 'ESModule', path: resolve('dist/server/index.js') },
    ...readdirSync('dist/server').filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: resolve('dist/server', name) })),
  ], compatibilityDate: '2026-10-02', d1Databases: { DB: 'hegemony-integration-db' } });
  let mf = new Miniflare(options);
  t.after(() => mf.dispose());
  let db = await mf.getD1Database('DB');
  const store = new RoomStore(db);
  const migration = readdirSync('drizzle').find(name => name.endsWith('.sql'));
  for (const sql of readFileSync('drizzle/' + migration, 'utf8').split('--> statement-breakpoint').filter(s => s.trim())) await db.prepare(sql).run();
  const pacedRooms = new Set();
  const pacedVersions = new Set(['rust-v0.2.5', 'rust-v0.2.6', 'rust-v0.2.7', 'rust-v0.2.8', 'rust-v0.2.9', 'rust-v0.2.10', 'rust-v0.2.11', 'rust-v0.2.36-jz55-unique-destroy-candidate']);
  const durableView = value => pacedVersions.has(value?.versions?.engine) ? { ...value, serverNowMs: undefined } : value;
  const api = async (path, body, session) => {
    if (body?.action && pacedRooms.has(session?.roomId) && !['game','beginResponse','passResponse','cancelAndPass','submitResponse'].includes(body.action.kind)) {
      body = { ...body, action: { kind: 'game', action: body.action } };
    }
    const response = await mf.dispatchFetch('http://localhost' + path, {
      method: body ? 'POST' : 'GET', headers: { ...(body ? { 'Content-Type': 'application/json' } : {}),
        ...(session ? { Authorization: 'Bearer ' + session.token } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
    const result = { status: response.status, body: response.status === 204 ? null : await response.json() };
    if (pacedVersions.has(result.body?.view?.versions?.engine)) pacedRooms.add(result.body.roomId);
    return result;
  };
  const counts = async id => {
    const rows = await db.batch(['seats', 'commands', 'journal'].map(table => db.prepare(`SELECT count(*) AS n FROM ${table} WHERE room_id=?`).bind(id)));
    return rows.map(r => r.results[0].n);
  };
  const key = () => crypto.randomUUID();
  const path = (session, action) => `/api/rooms/${session.roomId}/${action}`;
  try {
    const health = await api('/api/health'); assert.equal(health.body.transport, 'polling');
    const catalog = await api('/api/catalog'); assert.equal(catalog.body.cards.length, 100); for (const id of ['JC075','JC104','JZ31','JZ55']) assert(catalog.body.cards.some(c => c.id === id)); assert.equal(catalog.body.entryIdempotency, true);
    assert(catalog.body.cards.some(card => card.id === 'JC047'));
    assert(catalog.body.cards.some(card => card.id === 'JC007'));
    assert.equal(catalog.body.engineVersion, 'rust-v0.2.36-jz55-unique-destroy-candidate');
    assert.equal(catalog.body.cardPoolVersion, 'limited-v2.33-jz55-unique-destroy-candidate');
    assert.equal(catalog.body.deckBuildRules.minimumCards, 50);
    assert.equal(catalog.body.cards.find(card => card.id === 'JC125').deckCopyLimit, null);
    const create = { name: '甲', mode: 'teams', deckId: 'responders', requestId: key() };
    const hosts = await Promise.all([api('/api/rooms', create), api('/api/rooms', create)]);
    assert.equal(hosts[0].status, 200); assert.equal(hosts[1].status, 200); assert.deepEqual(hosts[0].body, hosts[1].body);
    const host = hosts[0].body;
    assert.equal((await api(path(host, 'catalog'))).status, 401);
    assert.deepEqual((await api(path(host, 'catalog'), null, host)).body, catalog.body);
    const currentKeepers = catalog.body.decks.find(deck => deck.id === 'keepers');
    assert.equal(currentKeepers.cards.find(entry => entry.cardId === 'JC058').count, 3);
    assert.equal(currentKeepers.cards.find(entry => entry.cardId === 'JC125').count, 14);
    assert.equal((await api('/api/rooms', { ...create, name: '异意图' })).status, 409);
    const joins = [1, 2, 3].map(i => ({ inviteCode: host.inviteCode, name: String(i), deckId: 'responders', requestId: key() }));
    const joined = await Promise.all(joins.map(body => api('/api/rooms/join', body)));
    assert(joined.every(r => r.status === 200));
    const players = [host, ...joined.map(r => r.body)];
    assert.equal(new Set(players.map(s => s.seat)).size, 4);
    assert.deepEqual(await counts(host.roomId), [4, 0, 3]);
    const retryJoin = await api('/api/rooms/join', joins[0]); assert.deepEqual(retryJoin.body, joined[0].body);
    assert.equal((await api('/api/rooms/join', { ...joins[0], name: '不同' })).status, 409);
    assert.deepEqual(await counts(host.roomId), [4, 0, 3]);
    const unauthenticated = await api(path(host, 'state'));
    assert.equal(unauthenticated.status, 401); assert.equal(unauthenticated.body.error, 'invalid_seat_token');
    assert.equal((await api(path(host, 'state') + '?afterVersion=3', null, host)).status, 204);
    const original = { commandId: key(), expectedVersion: 3, action: { kind: 'ready' } };
    const identical = await Promise.all([api(path(host, 'commands'), original, host), api(path(host, 'commands'), original, host)]);
    assert(identical.every(r => r.status === 200)); assert.deepEqual(identical[0].body, identical[1].body);
    assert.deepEqual(await counts(host.roomId), [4, 1, 4]);
    assert.equal((await api(path(host, 'commands'), original)).status, 401);
    assert.equal((await api(path(host, 'commands'), { ...original, action: { kind: 'pass' } }, host)).status, 409);
    assert.equal((await api(path(host, 'commands'), original, players[1])).status, 409);
    const concurrent = await Promise.all(players.slice(1, 3).map(s => api(path(s, 'commands'),
      { commandId: key(), expectedVersion: 4, action: { kind: 'ready' } }, s)));
    assert.deepEqual(concurrent.map(r => r.status).sort(), [200, 409]);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    const duelBody = { name: '末席测试', mode: 'duel', deckId: 'watchers', requestId: key() };
    const duel = (await api('/api/rooms', duelBody)).body;
    const lastSeatBody = { inviteCode: duel.inviteCode, name: '同一末席', deckId: 'responders', requestId: key() };
    const lastSeat = await Promise.all([api('/api/rooms/join', lastSeatBody), api('/api/rooms/join', lastSeatBody)]);
    assert(lastSeat.every(r => r.status === 200)); assert.deepEqual(lastSeat[0].body, lastSeat[1].body);
    assert.deepEqual(await counts(duel.roomId), [2, 0, 1]);
    // Test-only fresh lobby fixture; never mutate a live game's intermediate state.
    // The identical Rust WASM creates the maximum-u64 seed, kept opaque through D1 and Worker.
    initSync({ module: readFileSync((existsSync('rust-game-wasm') ? '' : '../') + 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm') });
    const preciseId = 'f'.repeat(24), preciseToken = 'a'.repeat(64);
    const precise = JSON.parse(newGame(preciseId, 'U64MAXTEST', 'duel', '精度测试', 'watchers', '18446744073709551615'));
    assert(precise.state.includes('"seed":18446744073709551615')); assert(precise.state.includes('"random":18446744073709551615'));
    await store.create({ id: preciseId, invite: 'U64MAXTEST', state: precise.state, nonce: key(), tokenHash: await digest(preciseToken),
      requestHash: await digest(key()), intentHash: 'fixture', response: '{}' });
    const preciseSession = { roomId: preciseId, token: preciseToken }; pacedRooms.add(preciseId);
    const preciseReady = await api(path(preciseSession, 'commands'), { commandId: key(), expectedVersion: 0, action: { kind: 'ready' } }, preciseSession);
    assert.equal(preciseReady.status, 200);
    const opaqueAfter = (await store.room(preciseId)).state;
    assert(opaqueAfter.includes('"seed":18446744073709551615')); assert(opaqueAfter.includes('"random":18446744073709551615'));
    // A genuinely old WASM fresh lobby is persisted locally, then advanced only by normal API actions.
    // Its pending choice and receipts must survive a Worker reopen without upgrading its rules.
    const oldRooms = [];
    for (const [oldKernel, engineVersion, idChar, tokenChar, invite] of [
      [legacy, 'rust-v0.2.1', 'e', 'b', 'LEGACYTEST21'],
      [intermediate, 'rust-v0.2.2', 'd', 'c', 'LEGACYTEST22'],
      [last, 'rust-v0.2.3', 'c', 'd', 'LEGACYTEST23'],
      [stable, 'rust-v0.2.4', 'b', 'e', 'LEGACYTEST24'],
      [paced, 'rust-v0.2.5', 'a', 'f', 'LEGACYTEST25'],
      [attached, 'rust-v0.2.6', '9', '9', 'LEGACYTEST26'],
      [grave, 'rust-v0.2.7', '8', '8', 'LEGACYTEST27'],
      [playable, 'rust-v0.2.8', '7', '7', 'LEGACYTEST28'],
      [society, 'rust-v0.2.9', '6', '6', 'LEGACYTEST29'],
      [forceMage, 'rust-v0.2.10', '5', '5', 'LEGACYTEST210'],
    ]) {
      oldKernel.initSync({ module: readFileSync((existsSync('rust-game-wasm') ? '' : '../') + `rust-game-wasm/legacy-v0.2.${engineVersion.slice("rust-v0.2.".length)}/hegemony_wasm_bg.wasm`) });
      const id = idChar.repeat(24), token = tokenChar.repeat(64);
      const old = JSON.parse(oldKernel.newGame(id, invite, 'duel', '旧核', 'watchers', '18446744073709551615'));
      assert.equal(old.view.versions.engine, engineVersion);
      await store.create({ id, invite, state: old.state, nonce: key(), tokenHash: await digest(token),
        requestHash: await digest(key()), intentHash: 'legacy-fixture', response: '{}' });
      const oldHost = { roomId: id, token };
      const roomCatalog = await api(path(oldHost, 'catalog'), null, oldHost);
      assert.equal(roomCatalog.status, 200);
      assert.equal(roomCatalog.body.engineVersion, engineVersion);
      assert.equal(roomCatalog.body.cards.some(c => c.id === 'JC004'), engineVersion === 'rust-v0.2.10');
      assert(!roomCatalog.body.cards.some(c => c.id === 'JC005'));
      if (['rust-v0.2.9', 'rust-v0.2.10'].includes(engineVersion)) {
        const before = (await store.room(id)).state;
        const unsupportedCards = engineVersion === 'rust-v0.2.9' ? ['JC004', 'JC005'] : ['JC005'];
        for (const cardId of unsupportedCards) for (const versions of [catalog.body, roomCatalog.body]) {
          const rejected = await api('/api/rooms/join', { inviteCode: invite, name: '新版构筑拒绝', requestId: key(), deckDraft: { id: 'new-deck', name: '新卡', societyId: null, cards: [{ cardId, count: 3 }, { cardId: 'JC125', count: 47 }], rulesVersion: versions.rulesVersion, cardPoolVersion: versions.cardPoolVersion, engineVersion: versions.engineVersion, description: '', updatedAt: '' } });
          assert.equal(rejected.status, 400); assert.equal((await store.room(id)).state, before);
          assert.deepEqual(await counts(id), [1, 0, 0]);
        }
      }
      const hasExtraGrayCharacter = ['rust-v0.2.3', 'rust-v0.2.4', 'rust-v0.2.5', 'rust-v0.2.6', 'rust-v0.2.7', 'rust-v0.2.8', 'rust-v0.2.9', 'rust-v0.2.10'].includes(engineVersion);
      const frozenCounts = { 'rust-v0.2.1': 29, 'rust-v0.2.2': 29, 'rust-v0.2.3': 30, 'rust-v0.2.4': 30, 'rust-v0.2.5': 36, 'rust-v0.2.6': 37, 'rust-v0.2.7': 39, 'rust-v0.2.8': 48, 'rust-v0.2.9': 48, 'rust-v0.2.10': 49 };
      assert.equal(roomCatalog.body.cards.length, frozenCounts[engineVersion]);
      const oldKeepers = roomCatalog.body.decks.find(deck => deck.id === 'keepers');
      assert.equal(oldKeepers.cards.find(entry => entry.cardId === 'JC125').count, hasExtraGrayCharacter ? 14 : 17);
      assert.equal(oldKeepers.cards.some(entry => entry.cardId === 'JC058'), hasExtraGrayCharacter);
      const oldJoin = await api('/api/rooms/join', { inviteCode: invite, name: '旧核对手', deckId: 'hunters', requestId: key() });
      assert.equal(oldJoin.status, 200); assert.equal(oldJoin.body.view.versions.engine, engineVersion);
      const oldGuest = oldJoin.body;
      for (const [actor, version, action] of [[oldHost, 1, 'ready'], [oldGuest, 2, 'ready'], [oldHost, 3, 'start']]) {
        const result = await api(path(actor, 'commands'), { commandId: key(), expectedVersion: version, action: { kind: action } }, actor);
        assert.equal(result.status, 200); assert.equal(result.body.versions.engine, engineVersion);
      }
      const views = await Promise.all([oldHost, oldGuest].map(async actor => (await api(path(actor, 'state'), null, actor)).body));
      const choiceSeat = views.findIndex(v => v.pendingChoice?.kind === 'mulligan');
      assert(choiceSeat >= 0); assert(views.every(v => v.versions.engine === engineVersion));
      const opaque = (await store.room(id)).state;
      assert(opaque.includes('"seed":18446744073709551615'));
      oldRooms.push({ id, engineVersion, actors: [oldHost, oldGuest], views, choiceSeat, opaque, catalog: roomCatalog.body });
    }
    const draft = { id: 'cloud-draft-a', name: '云端自组', description: '复制预组后编辑', societyId: null,
      cards: [{ cardId: 'JC005', count: 3 }, { cardId: 'JC125', count: 47 }],
      rulesVersion: catalog.body.rulesVersion, cardPoolVersion: catalog.body.cardPoolVersion,
      engineVersion: catalog.body.engineVersion, updatedAt: '2026-10-02T16:00:00Z' };
    const customBody = { name: '构筑甲', mode: 'duel', deckDraft: draft, requestId: key() };
    const customCreates = await Promise.all([api('/api/rooms', customBody), api('/api/rooms', customBody)]);
    assert(customCreates.every(result => result.status === 200));
    assert.deepEqual(customCreates[0], customCreates[1]);
    const customHost = customCreates[0].body;
    const entries = cards => [...cards].sort((left, right) => left.cardId.localeCompare(right.cardId));
    assert.deepEqual(entries(customHost.view.yourDeck.cards), entries(draft.cards));
    assert.equal(customHost.view.players[0].deckName, draft.name);
    assert.equal((await api('/api/rooms', { ...customBody, deckDraft: { ...draft, name: '改变同请求意图' } })).status, 409);
    for (const invalidDraft of [
      { ...draft, cards: [{ cardId: 'JC125', count: 49 }] },
      { ...draft, cards: [{ cardId: 'unknown-source', count: 50 }] },
      { ...draft, cards: [{ cardId: 'JC058', count: 50 }] },
      { ...draft, societyId: 'unknown-society-source' },
      { ...draft, engineVersion: 'rust-v0.2.3' },
    ]) assert.equal((await api('/api/rooms', { ...customBody, deckDraft: invalidDraft, requestId: key() })).status, 400);
    const guestDraft = { ...draft, id: 'cloud-draft-b', name: '构筑乙卡组', cards: [{ cardId: 'JC125', count: 50 }] };
    const customJoinBody = { name: '构筑乙', inviteCode: customHost.inviteCode, deckDraft: guestDraft, requestId: key() };
    const customJoins = await Promise.all([api('/api/rooms/join', customJoinBody), api('/api/rooms/join', customJoinBody)]);
    assert(customJoins.every(result => result.status === 200)); assert.deepEqual(customJoins[0], customJoins[1]);
    const customGuest = customJoins[0].body;
    assert.deepEqual(customGuest.view.yourDeck.cards, guestDraft.cards);
    assert(!JSON.stringify(customGuest.view.players).includes('cards'));
    const frozenHostView = (await api(path(customHost, 'state'), null, customHost)).body;
    draft.cards[0].count += 1; // Editing the original browser draft cannot mutate the persisted copy.
    assert.deepEqual(durableView((await api(path(customHost, 'state'), null, customHost)).body), durableView(frozenHostView));
    for (const old of oldRooms) {
      if (['rust-v0.2.4', 'rust-v0.2.5', 'rust-v0.2.6', 'rust-v0.2.7', 'rust-v0.2.8', 'rust-v0.2.9', 'rust-v0.2.10'].includes(old.engineVersion)) continue; // Frozen saved-deck kernels accept only their own-version drafts.
      const rejected = await api('/api/rooms/join', { ...customJoinBody, inviteCode: old.actors[0].inviteCode || (await store.room(old.id)).invite, requestId: key() });
      assert.equal(rejected.status, 400); assert.match(rejected.body.message, /旧牌桌/);
      assert.equal((await store.room(old.id)).state, old.opaque);
    }
    const stale = await api(path(host, 'commands'), { commandId: key(), expectedVersion: 4, action: { kind: 'ready' } }, host);
    assert.equal(stale.status, 409); assert.equal(stale.body.view.version, 5);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    const before = await db.prepare('SELECT * FROM rooms WHERE id=?').bind(host.roomId).first();
    const zero = await store.command({ id: host.roomId, seat: 0, commandId: key(), intentHash: 'fixture', expectedVersion: 4,
      state: before.state, version: 5, nonce: key(), response: '{}', entry: '{}' });
    assert.equal(zero, false); assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    assert.deepEqual(await db.prepare('SELECT * FROM rooms WHERE id=?').bind(host.roomId).first(), before);
    // A journal unique violation after a successful CAS must roll back that CAS.
    await db.prepare('INSERT INTO journal(room_id,version,entry) VALUES(?,6,?)').bind(host.roomId, '{}').run();
    await assert.rejects(store.command({ id: host.roomId, seat: 0, commandId: key(), intentHash: 'fixture', expectedVersion: 5,
      state: before.state, version: 6, nonce: key(), response: '{}', entry: '{}' }));
    assert.deepEqual(await db.prepare('SELECT * FROM rooms WHERE id=?').bind(host.roomId).first(), before);
    await db.prepare('DELETE FROM journal WHERE room_id=? AND version=6').bind(host.roomId).run();
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    const snapshot = (await api(path(host, 'state'), null, host)).body;
    assert.equal(snapshot.version, 5); assert(!('state' in snapshot)); assert(!('seed' in snapshot));
    await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
    assert.deepEqual(durableView((await api(path(customHost, 'state'), null, customHost)).body), durableView(frozenHostView));
    assert.deepEqual((await api('/api/rooms/join', customJoinBody)).body, customGuest);
    for (const old of oldRooms) {
      for (const [i, actor] of old.actors.entries()) {
        assert.deepEqual(durableView((await api(path(actor, 'state'), null, actor)).body), durableView(old.views[i]));
        assert.deepEqual((await api(path(actor, 'catalog'), null, actor)).body, old.catalog);
      }
      assert.equal((await new RoomStore(db).room(old.id)).state, old.opaque);
      const actor = old.actors[old.choiceSeat];
      const command = { commandId: key(), expectedVersion: 4, action: { kind: 'choose', choiceId: old.views[old.choiceSeat].pendingChoice.id, selected: [] } };
      const result = await api(path(actor, 'commands'), command, actor);
      assert.equal(result.status, 200); assert.equal(result.body.versions.engine, old.engineVersion);
      const after = (await new RoomStore(db).room(old.id)).state;
      assert.deepEqual(await api(path(actor, 'commands'), command, actor), result);
      assert.equal((await new RoomStore(db).room(old.id)).state, after);
      assert.deepEqual(await counts(old.id), [2, 4, 5]);
    }
    assert.deepEqual(durableView((await api(path(host, 'state'), null, host)).body), durableView(snapshot));
    assert.deepEqual((await api(path(host, 'commands'), original, host)).body, identical[0].body);
    assert.deepEqual(await counts(host.roomId), [4, 2, 5]);
    t.diagnostic(JSON.stringify({ currentVersion: catalog.body.engineVersion, reopenedFrozenVersions: oldRooms.map(r => r.engineVersion), v010PendingChoiceAndReceiptsPreserved: oldRooms.some(r => r.engineVersion === 'rust-v0.2.10'), newJC005DraftRejectedByV009AndV010WithoutWrites: true, localWorkerdD1Only: true }));
    t.diagnostic('Provider emulation only; remote Cloudflare D1 acceptance is still required. Persisted at ' + persist);
  } finally { await mf.dispose(); }
});
