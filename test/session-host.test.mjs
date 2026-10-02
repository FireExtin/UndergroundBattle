import { test } from 'node:test';
import assert from 'node:assert/strict';
import { HttpError, RoomService, normalizedAction } from '../src/service.mjs';

const authorization = 'Bearer ' + 'a'.repeat(64);
const actorView = version => ({ roomId: 'clock-room', version, you: 'p0' });
const accepted = version => JSON.stringify({ state: `opaque-${version}`, version, seat: 0,
  view: actorView(version), changed: true, outcome: 'accepted', journal: [{ Command: { seat: 0 } }] });
const expired = version => JSON.stringify({ state: `opaque-${version}`, version, seat: 0,
  view: actorView(version), changed: true, outcome: 'rejected', errorCode: 'window_expired',
  errorMessage: '原响应窗口已结束', journal: [{ Tick: { server_now_ms: 6000 } }] });
function harness(kernel, now = () => 6000) {
  const service = new RoomService({}, { supportsPacing: () => true, ...kernel }, now);
  const current = { id: 'clock-room', state: 'opaque-4', version: 4 };
  const receipts = new Map(), writes = [], ticks = [];
  service.store = {
    seat: async () => ({ seat: 0 }), room: async () => ({ ...current }),
    receipt: async (_, id) => receipts.get(id),
    command: async value => {
      writes.push(value); Object.assign(current, { state: value.state, version: value.version });
      receipts.set(value.commandId, { intent_hash: value.intentHash, response: value.response }); return true;
    },
    system: async value => {
      ticks.push(value); Object.assign(current, { state: value.state, version: value.version }); return true;
    },
  };
  return { service, current, receipts, writes, ticks };
}

test('recovering a paid response receipt precedes all later clock evaluation', async () => {
  let applies = 0, clockReads = 0;
  const h = harness({ applyRoom: () => { applies++; return accepted(5); } }, () => { clockReads++; return 1000; });
  const original = { commandId: 'paid-original', expectedVersion: 4,
    action: { kind: 'submitResponse', windowId: 'response:1', intentId: 'intent-1', action: { kind: 'reveal', cardId: 'private-own-card' } } };
  const first = await h.service.command('clock-room', authorization, original);
  Object.assign(h.current, { version: 19, state: 'opaque-19' });
  h.service.now = () => { throw new Error('a recovered receipt must not read time'); };
  assert.deepEqual(await h.service.command('clock-room', authorization, original), first);
  assert.equal(applies, 1); assert.equal(clockReads, 1); assert.equal(h.writes.length, 1);
  await assert.rejects(h.service.command('clock-room', authorization,
    { ...original, action: { ...original.action, intentId: 'changed-intent' } }), error => error.status === 409);
});

test('a matching-version GET persists an expired window before deciding to return 204', async () => {
  const h = harness({ pollRoom: (state, seat, now) => {
    assert.equal(seat, 0); assert.equal(now, '6000');
    return state === 'opaque-4' ? expired(5) : JSON.stringify({ state, version: 5, seat,
      view: actorView(5), changed: false, outcome: 'accepted', journal: [] });
  } });
  assert.deepEqual(await h.service.state('clock-room', authorization, '4'), actorView(5));
  assert.equal(await h.service.state('clock-room', authorization, '5'), null);
  assert.equal(h.ticks.length, 1); assert.equal(h.writes.length, 0); assert.equal(h.receipts.size, 0);
  assert.equal(h.ticks[0].expectedVersion, 4);
  assert.deepEqual(JSON.parse(h.ticks[0].entry), { SessionEvents: { events: [{ Tick: { server_now_ms: 6000 } }] } });
});

test('an expired response commits only the system tick and never an accepted user receipt', async () => {
  const h = harness({ applyRoom: () => expired(5) });
  await assert.rejects(h.service.command('clock-room', authorization, { commandId: 'late-begin', expectedVersion: 4,
    action: { kind: 'beginResponse', windowId: 'response:1', intentId: 'too-late' } }),
  error => error instanceof HttpError && error.status === 409 && error.code === 'window_expired' && error.view.version === 5);
  assert.equal(h.current.version, 5); assert.equal(h.ticks.length, 1); assert.equal(h.writes.length, 0);
  assert.equal(h.receipts.size, 0);
});

test('a CAS conflict retries Begin with its original expected version and complete intent', async () => {
  const seen = [];
  const h = harness({ applyRoom: (state, seat, raw, now) => {
    seen.push({ state, seat, command: JSON.parse(raw), now });
    return accepted(state === 'opaque-4' ? 5 : 6);
  } });
  const original = { commandId: 'begin-original', expectedVersion: 4,
    action: { kind: 'beginResponse', windowId: 'response:1', intentId: 'intent-1' } };
  const commit = h.service.store.command;
  let conflicts = 0;
  h.service.store.command = async value => {
    if (conflicts++ === 0) { Object.assign(h.current, { version: 5, state: 'opaque-5' }); return false; }
    return commit(value);
  };
  assert.deepEqual(await h.service.command('clock-room', authorization, original), actorView(6));
  assert.deepEqual(seen.map(value => value.command), [original, original]);
  assert.deepEqual(seen.map(value => value.state), ['opaque-4', 'opaque-5']);
  assert.equal(h.writes[0].expectedVersion, 5); assert.equal(h.writes[0].commandId, original.commandId);
});

test('a formal response cannot be rebased by the host after another seat wins CAS', async () => {
  const seen = [];
  const h = harness({ applyRoom: (state, seat, raw) => {
    seen.push(JSON.parse(raw));
    if (state === 'opaque-4') return accepted(5);
    return JSON.stringify({ state, version: 5, seat, view: actorView(5), changed: false,
      outcome: 'rejected', errorCode: 'version_conflict', journal: [] });
  } });
  const original = { commandId: 'formal-original', expectedVersion: 4,
    action: { kind: 'submitResponse', windowId: 'response:1', intentId: 'intent-1',
      action: { kind: 'play', cardId: 'own-card', targetId: 'original-target', option: 'original-mode' } } };
  h.service.store.command = async () => { Object.assign(h.current, { version: 5, state: 'opaque-5' }); return false; };
  await assert.rejects(h.service.command('clock-room', authorization, original), error => error.status === 409);
  assert.deepEqual(seen, [original, original]); assert.equal(h.receipts.size, 0);
  assert.equal(h.writes.length, 0); assert.equal(h.ticks.length, 0);
});

test('client clock injection and nested Session commands are rejected before the reducer', () => {
  for (const value of [
    { kind: 'beginResponse', windowId: 'response:1', intentId: 'i', serverNowMs: 1000 },
    { kind: 'game', action: { kind: 'passResponse', windowId: 'response:1' } },
    { kind: 'submitResponse', windowId: 'response:1', intentId: 'i', action: { kind: 'game', action: { kind: 'pass' } } },
    { kind: 'toString', serverNowMs: 1 },
  ]) assert.throws(() => normalizedAction(value), error => error.status === 400);
  assert.deepEqual(normalizedAction({ kind: 'game', action: { kind: 'choose', choiceId: 'choice-original', selected: ['option-original'] } }),
    { kind: 'game', action: { kind: 'choose', choiceId: 'choice-original', selected: ['option-original'] } });
});

test('quoting an active composition refreshes its clock and does not commit a draft or receipt', async () => {
  let quotes = 0;
  const h = harness({ pollRoom: (state, seat, now) => JSON.stringify({ state, version: 4, seat,
    view: actorView(4), changed: false, outcome: 'accepted', journal: [] }),
  quoteRoom: (state, seat, raw) => {
    quotes++; assert.equal(state, 'opaque-4'); assert.equal(seat, 0);
    assert.deepEqual(JSON.parse(raw), { windowId: 'response:1', intentId: 'i', draft: { kind: 'play', cardId: 'own-card', targetId: 'target' } });
    return JSON.stringify({ version: 4, windowId: 'response:1', intentId: 'i', ready: true, legalActions: [] });
  } });
  const request = { windowId: 'response:1', intentId: 'i', draft: { kind: 'play', cardId: 'own-card', targetId: 'target' } };
  assert.equal((await h.service.quote('clock-room', authorization, request)).ready, true);
  assert.equal(quotes, 1); assert.equal(h.current.version, 4);
  assert.equal(h.ticks.length, 0); assert.equal(h.writes.length, 0); assert.equal(h.receipts.size, 0);
});
