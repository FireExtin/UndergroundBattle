import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { kernel as abi } from '../../rust-game-wasm/tests/current-kernel.mjs';
import { loadCurrentFixture } from '../../tools/current-native-source.mjs';
import { routeKernels } from '../src/kernel-router.mjs';
import { RoomStore } from '../src/store.mjs';
import { RoomService, digest } from '../src/service.mjs';
import { passRecord } from '../src/pass-history.mjs';

const native = loadCurrentFixture('rust-game-wasm/tests/current112/jc046.json.gz');
assert.deepEqual(JSON.parse(abi.catalog()), native.catalog);
assert.equal(native.scenarios.length, 37);
const worker = path.dirname(process.env.CURRENT_CLOCK_WORKER || fileURLToPath(new URL('../dist/current-clock/worker.js', import.meta.url)));
async function harness(t, name) {
  const persist = mkdtempSync(path.join(tmpdir(), 'public-pass-'));
  const options = convertV4MiniflareOptions({ name, resourcePersistencePath: persist, modulesRoot: worker, modules: [
    { type: 'ESModule', path: path.join(worker, 'worker.js') },
    ...readdirSync(worker).filter(name => name.endsWith('.wasm')).map(name => ({ type: 'CompiledWasm', path: path.join(worker, name) })),
  ], compatibilityDate: '2026-10-02', cf: false, d1Databases: { DB: name }, bindings: { LOCAL_TEST_CLOCK_MS: '0' } });
  let mf = new Miniflare(options), db;
  t.after(async () => { await mf.dispose(); rmSync(persist, { recursive: true, force: true }); });
  db = await mf.getD1Database('DB');
  for (const file of readdirSync(new URL('../drizzle/', import.meta.url)).filter(name => name.endsWith('.sql'))) {
    for (const sql of readFileSync(new URL('../drizzle/' + file, import.meta.url), 'utf8').split('--> statement-breakpoint').filter(sql => sql.trim())) await db.prepare(sql).run();
  }
  return { db: () => db, fetch: (...args) => mf.dispatchFetch(...args), reopen: async () => {
    await mf.dispose(); mf = new Miniflare(options); db = await mf.getD1Database('DB');
  } };
}
async function seed(h, state) {
  const view = JSON.parse(abi.view(state, 0)), store = new RoomStore(h.db());
  const tokens = view.players.map((_, seat) => String(seat + 1).repeat(64));
  await store.create({ id: view.roomId, invite: view.inviteCode, state, nonce: crypto.randomUUID(), tokenHash: await digest(tokens[0]), requestHash: crypto.randomUUID(), intentHash: 'explicit-native93-fixture', response: '{}' });
  await h.db().prepare('UPDATE rooms SET version=? WHERE id=?').bind(view.version, view.roomId).run();
  for (let seat = 1; seat < tokens.length; seat++) await h.db().prepare('INSERT INTO seats(room_id,seat,token_hash) VALUES(?,?,?)').bind(view.roomId, seat, await digest(tokens[seat])).run();
  return { id: view.roomId, tokens };
}
const auth = (room, seat) => 'Bearer ' + room.tokens[seat];
const stripMetadata = entry => {
  const parsed = JSON.parse(entry);
  if (parsed.SessionEvents) delete parsed.SessionEvents.publicPasses;
  return parsed;
};

test('all 37 current Native93 traces preserve opaque states, core views, journals and duplicate receipts while recording passes', async t => {
  const h = await harness(t, 'pass-native93');
  let commands = 0, records = 0;
  for (const scenario of native.scenarios) {
    // Each source trace intentionally uses the same test-only room identity.
    for (const table of ['commands','journal','entry_receipts','seats','rooms']) await h.db().prepare(`DELETE FROM ${table}`).run();
    const room = await seed(h, scenario.initial.state);
    let service = new RoomService(h.db(), routeKernels(abi), () => 0);
    let expectedPasses = [];
    for (const step of scenario.steps) {
      const before = JSON.parse(abi.view(step.state, step.seat));
      const kind = step.command.action.kind;
      const isPass = kind === 'passResponse' || kind === 'cancelAndPass' || kind === 'game' && step.command.action.action.kind === 'pass';
      if (step.expected.errorCode) {
        await assert.rejects(service.command(room.id, auth(room, step.seat), step.command), error => error.code === step.expected.errorCode);
      } else {
        const response = await service.command(room.id, auth(room, step.seat), step.command);
        assert.deepEqual(response, step.expected.view, scenario.name);
        assert.deepEqual(await service.command(room.id, auth(room, step.seat), step.command), response);
        if (isPass) {
          expectedPasses.push(passRecord(before, step.seat, step.expected.version, 0, kind === 'passResponse' ? 'response' : kind === 'cancelAndPass' ? 'cancelResponse' : 'pass'));
          records++;
        }
        const row = await h.db().prepare('SELECT entry FROM journal WHERE room_id=? AND version=?').bind(room.id, step.expected.version).first();
        assert.deepEqual(stripMetadata(row.entry), { SessionEvents: { events: step.expected.journal } }, scenario.name);
        commands++;
      }
      assert.equal((await new RoomStore(h.db()).room(room.id)).state, step.expected.state, scenario.name);
      for (let seat = 0; seat < room.tokens.length; seat++) {
        const publicView = await service.publicView(JSON.parse(abi.view(step.expected.state, seat)));
        assert.deepEqual(publicView.passHistory || [], expectedPasses.slice(-50), scenario.name);
      }
    }
    await h.reopen(); service = new RoomService(h.db(), routeKernels(abi), () => 0);
    const restored = await service.state(room.id, auth(room, 0), null);
    assert.deepEqual((await service.publicView(restored)).passHistory || [], expectedPasses.slice(-50), scenario.name);
  }
  assert(commands > 300); assert(records > 100);
  t.diagnostic(JSON.stringify({ scenarios: native.scenarios.length, acceptedCommands: commands, publicPassRecords: records, originalOpaqueStatesEqual: true, originalCoreViewsEqual: true }));
});

test('timeout, cancellation, pause and late rejection record only committed public passes and preserve their original contexts', async t => {
  const h = await harness(t, 'pass-timeout');
  const scenario = native.scenarios.find(scenario => scenario.name === 'response-source-bounce');
  const prepared = scenario.steps.find(step => step.command.action.kind === 'beginResponse');
  assert(prepared);
  const room = await seed(h, prepared.state);
  let now = 0;
  let service = new RoomService(h.db(), routeKernels(abi), () => now);
  let state = prepared.state;
  const send = async (seat, action) => {
    const view = JSON.parse(abi.view(state, seat));
    const command = { commandId: crypto.randomUUID(), expectedVersion: view.version, action };
    const response = await service.command(room.id, auth(room, seat), command);
    state = (await new RoomStore(h.db()).room(room.id)).state;
    return { command, response };
  };
  const before = JSON.parse(abi.view(state, prepared.seat));
  await send(prepared.seat, prepared.command.action);
  let view = JSON.parse(abi.view(state, prepared.seat));
  const intent = view.responseWindow.myIntentId;
  const cancel = await send(prepared.seat, { kind: 'cancelAndPass', windowId: view.responseWindow.id, intentId: intent });
  const cancelHistory = (await service.publicView(cancel.response)).passHistory;
  assert.equal(cancelHistory.length, 1); assert.equal(cancelHistory[0].reason, 'cancelResponse');
  assert.equal(cancelHistory[0].phase, before.phase); assert.equal(cancelHistory[0].step, before.step);
  view = JSON.parse(abi.view(state, 0));
  await send(0, { kind: 'pauseRoom' });
  now = 100000;
  assert.equal((await service.publicView(await service.state(room.id, auth(room, 1), null))).passHistory.length, 1);
  await send(1, { kind: 'resumeRoom' });
  view = JSON.parse(abi.view(state, 0));
  const undecided = view.responseWindow.members.filter(member => member.status === 'undecided');
  assert(undecided.length);
  now = Math.max(...undecided.map(member => member.deadlineMs));
  const late = { commandId: crypto.randomUUID(), expectedVersion: view.version, action: { kind: 'passResponse', windowId: view.responseWindow.id } };
  const actor = view.players.find(player => player.id === undecided[0].playerId).seat;
  await assert.rejects(service.command(room.id, auth(room, actor), late), error => error.code === 'window_expired');
  assert.equal(await new RoomStore(h.db()).receipt(room.id, late.commandId), null);
  state = (await new RoomStore(h.db()).room(room.id)).state;
  const history = (await service.publicView(JSON.parse(abi.view(state, 0)))).passHistory;
  assert.equal(history.length, 1 + undecided.length);
  assert(history.slice(1).every(record => record.reason === 'timeout' && record.phase === view.phase && record.step === view.step));
  assert.equal(new Set(history.map(record => record.id)).size, history.length);
  const oldReceipt = await service.publicView(await service.command(room.id, auth(room, prepared.seat), cancel.command));
  assert.deepEqual(oldReceipt.passHistory, cancelHistory); // No future events attached to an old ACK.
  await h.reopen(); service = new RoomService(h.db(), routeKernels(abi), () => now);
  for (let seat = 0; seat < 4; seat++) assert.deepEqual((await service.publicView(await service.state(room.id, auth(room, seat), null))).passHistory, history);
});

test('real HTTP manual passes traverse a complete duel turn; both clients and a reopened Worker see identical event-time history', async t => {
  const h = await harness(t, 'pass-http-duel');
  const api = async (route, body, session) => {
    const response = await h.fetch('http://localhost/api/' + route, { method: body ? 'POST' : 'GET', headers: {
      ...(body ? { 'Content-Type': 'application/json' } : {}), ...(session ? { Authorization: 'Bearer ' + session.token } : {}), 'X-Test-Server-Now': '0',
    }, ...(body ? { body: JSON.stringify(body) } : {}) });
    return { status: response.status, body: response.status === 204 ? null : await response.json() };
  };
  const versions = JSON.parse(abi.catalog());
  const deckDraft = { id: 'pass-only', name: '让过验证牌组', description: '', societyId: null, cards: [{ cardId: 'JC125', count: 50 }], rulesVersion: versions.rulesVersion, cardPoolVersion: versions.cardPoolVersion, engineVersion: versions.engineVersion, updatedAt: '' };
  const host = (await api('rooms', { name: '甲端', mode: 'duel', deckDraft, requestId: crypto.randomUUID() })).body;
  const guest = (await api('rooms/join', { name: '乙端', inviteCode: host.inviteCode, deckDraft, requestId: crypto.randomUUID() })).body;
  const sessions = [host, guest].sort((a,b) => a.seat - b.seat);
  const id = host.roomId;
  const pathFor = operation => `rooms/${id}/${operation}`;
  let view = guest.view;
  const send = async (seat, action) => {
    const command = { commandId: crypto.randomUUID(), expectedVersion: view.version, action };
    const result = await api(pathFor('commands'), command, sessions[seat]);
    assert.equal(result.status, 200, JSON.stringify(result.body)); view = result.body;
    return command;
  };
  for (const session of sessions) await send(session.seat, { kind: 'game', action: { kind: 'ready' } });
  await send(0, { kind: 'game', action: { kind: 'start' } });
  const seen = new Set(), expected = [];
  let firstPass, firstReceipt;
  for (let guard = 0; guard < 250 && !(view.turn > 1); guard++) {
    if (view.pendingChoice || view.waitingChoice) {
      const playerId = (view.pendingChoice || view.waitingChoice).playerId;
      const seat = view.players.find(player => player.id === playerId).seat;
      const own = (await api(pathFor('state'), null, sessions[seat])).body;
      const choice = own.pendingChoice;
      assert(choice); view = own;
      const action = choice.kind === 'forecast' ? { kind: 'choose', choiceId: choice.id, top: choice.options.map(option => option.id), bottom: [] }
        : { kind: 'choose', choiceId: choice.id, selected: choice.options.slice(0, choice.min).map(option => option.id) };
      await send(seat, { kind: 'game', action });
      continue;
    }
    let seat;
    for (const session of sessions) {
      const candidate = (await api(pathFor('state'), null, session)).body;
      if (candidate.legalActions.some(action => action.kind === 'pass')) { view = candidate; seat = session.seat; break; }
    }
    assert.notEqual(seat, undefined);
    const before = view;
    seen.add(before.phase + '/' + before.step);
    const command = await send(seat, { kind: 'game', action: { kind: 'pass' } });
    expected.push(passRecord(before, seat, view.version, 0, 'pass'));
    assert.deepEqual(view.passHistory, expected.slice(-50));
    if (!firstPass) { firstPass = { seat, command }; firstReceipt = structuredClone(view); }
    for (const session of sessions) assert.deepEqual((await api(pathFor('state'), null, session)).body.passHistory, expected.slice(-50));
  }
  assert(view.turn > 1, 'complete first turn');
  for (const prefix of ['start/prepare','start/draw','action/team:','confrontation/mobility','confrontation/region:','end/end']) assert([...seen].some(step => step.startsWith(prefix)), prefix);
  assert(expected.some(record => record.region)); assert(expected.some(record => record.region === null));
  const raw = await h.db().prepare('SELECT state FROM rooms WHERE id=?').bind(id).first();
  assert(!raw.state.includes('passHistory')); assert(!raw.state.includes('publicPasses'));
  const original = await api(pathFor('commands'), firstPass.command, sessions[firstPass.seat]);
  assert.deepEqual(original.body, firstReceipt);
  await h.reopen();
  for (const session of sessions) {
    const restored = await api(pathFor('state'), null, session);
    assert.equal(restored.status, 200); assert.deepEqual(restored.body.passHistory, expected.slice(-50));
  }
  assert.equal((await api(pathFor('state'))).status, 401);
  assert.equal((await api(pathFor('state') + '?afterVersion=' + view.version, null, host)).status, 204);
  t.diagnostic(JSON.stringify({ publicPasses: expected.length, windows: [...seen], fullTurn: true, bothClients: true, reopened: true }));
});

test('pass metadata shares the command CAS: rollback and concurrent retries leave exactly one committed event', async t => {
  const h = await harness(t, 'pass-atomic');
  const scenario = native.scenarios.find(scenario => scenario.name === 'combat-kill-two');
  const step = scenario.steps.find(step => step.command.action.kind === 'game' && step.command.action.action.kind === 'pass');
  const room = await seed(h, step.state);
  const store = new RoomStore(h.db()), service = new RoomService(h.db(), routeKernels(abi), () => 0);
  const before = await store.room(room.id);
  await h.db().prepare('INSERT INTO journal(room_id,version,entry) VALUES(?,?,?)').bind(room.id, step.expected.version, '{}').run();
  await assert.rejects(service.command(room.id, auth(room, step.seat), step.command));
  assert.deepEqual(await store.room(room.id), before);
  assert.equal(await store.receipt(room.id, step.command.commandId), null);
  assert.deepEqual(await store.publicPasses(room.id, step.expected.version), []);
  await h.db().prepare('DELETE FROM journal WHERE room_id=? AND version=?').bind(room.id, step.expected.version).run();
  const responses = await Promise.all([0,1].map(() => service.command(room.id, auth(room, step.seat), step.command)));
  assert.deepEqual(responses[0], responses[1]);
  assert.equal((await store.room(room.id)).state, step.expected.state);
  assert.equal((await store.publicPasses(room.id, step.expected.version)).length, 1);
  assert.equal((await h.db().prepare('SELECT count(*) AS n FROM commands WHERE room_id=?').bind(room.id).first()).n, 1);
  const changed = { ...step.command, action: { kind: 'pauseRoom' } };
  await assert.rejects(service.command(room.id, auth(room, step.seat), changed), error => error.status === 409);
  assert.equal((await store.publicPasses(room.id, step.expected.version)).length, 1);
});
