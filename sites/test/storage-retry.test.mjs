import { test } from 'node:test';
import assert from 'node:assert/strict';
import { RoomService, digest } from '../src/service.mjs';
import { storageFailure, retryOriginalCommand } from '../src/storage-errors.mjs';

test('an ambiguous D1 commit retries the identical command and returns its receipt without applying twice', async () => {
  const response = { roomId: 'room', version: 2, you: 'p0' };
  const token = 'a'.repeat(64), command = { commandId: 'original-id', expectedVersion: 1, action: { kind: 'pass' } };
  let receipt = null, applies = 0, writes = 0;
  const service = new RoomService({}, { apply: () => {
    applies++; return JSON.stringify({ state: 'new-opaque-state', version: 2, seat: 0, view: response });
  } });
  service.store = {
    seat: async (_, hash) => { assert.equal(hash, await digest(token)); return { seat: 0 }; },
    receipt: async (_, id) => { assert.equal(id, command.commandId); return receipt; },
    room: async () => ({ version: 1, state: 'old-opaque-state' }),
    command: async value => {
      writes++; receipt = { intent_hash: value.intentHash, response: value.response };
      throw new Error('D1_ERROR: Network connection lost. private SQL and token must never be logged');
    },
  };
  assert.deepEqual(await service.command('room', 'Bearer ' + token, command), response);
  assert.equal(applies, 1); assert.equal(writes, 1);
  assert.deepEqual(await service.command('room', 'Bearer ' + token, command), response);
  assert.equal(applies, 1); assert.equal(writes, 1);
  await assert.rejects(service.command('room', 'Bearer ' + token, { ...command, action: { kind: 'ready' } }), error => error.status === 409);
});
test('storage diagnostics expose fixed labels and do not retry quota, constraint or unknown failures', () => {
  assert.deepEqual(storageFailure(new Error('UNIQUE constraint failed: private data')), { code: 'unique_constraint', retryable: false });
  assert.deepEqual(storageFailure({ cause: new Error('D1_ERROR: Internal error in D1 DB storage caused object to be reset.') }), { code: 'storage_reset', retryable: true });
  assert.deepEqual(storageFailure(new Error('Your account has exceeded D1\'s free tier daily row read limit. secret')), { code: 'row_read_quota', retryable: false });
  assert.deepEqual(storageFailure(new Error('unknown exception with a bearer value')), { code: 'unclassified', retryable: false });
});
test('a failure before commit retries the original identity, while an exhausted failure never returns success', async () => {
  const token = 'c'.repeat(64), command = { commandId: 'unchanged-id', expectedVersion: 1, action: { kind: 'pass' } };
  const view = { version: 2, you: 'p0' }; let receipt = null, calls = 0, commits = 0;
  const service = new RoomService({}, { apply: () => JSON.stringify({ state: 'new', version: 2, seat: 0, view }) });
  service.store = {
    seat: async () => ({ seat: 0 }), receipt: async () => receipt,
    room: async () => ({ version: 1, state: 'old' }),
    command: async value => {
      assert.equal(value.commandId, command.commandId); assert.equal(value.expectedVersion, 1); assert.equal(value.seat, 0);
      if (++calls === 1) throw new Error('Network connection lost.');
      commits++; receipt = { intent_hash: value.intentHash, response: value.response }; return true;
    },
  };
  assert.deepEqual(await service.command('room', 'Bearer ' + token, command), view);
  assert.equal(calls, 2); assert.equal(commits, 1);
  receipt = null; calls = 0;
  service.store.command = async () => { calls++; throw new Error('Network connection lost.'); };
  await assert.rejects(service.command('room', 'Bearer ' + token, command), /Network connection lost/);
  assert.equal(calls, 3); assert.equal(commits, 1);
});
test('transient failures have a finite budget; HTTP rejections and unclassified failures are returned immediately', async () => {
  const failure = new Error('Network connection lost. private SQL');
  let attempts = 0; const delays = [], logs = [];
  await assert.rejects(retryOriginalCommand(async () => { attempts++; throw failure; }, {
    wait: async ms => delays.push(ms), log: (...args) => logs.push(args),
  }), error => error === failure);
  assert.equal(attempts, 3); assert.equal(delays.length, 2);
  assert(delays[0] >= 150 && delays[0] < 250); assert(delays[1] >= 300 && delays[1] < 400);
  assert(!JSON.stringify(logs).includes('private SQL'));
  for (const error of [Object.assign(new Error('Network connection lost.'), { status: 409 }), new Error('unclassified')]) {
    attempts = 0;
    await assert.rejects(retryOriginalCommand(async () => { attempts++; throw error; }), value => value === error);
    assert.equal(attempts, 1);
  }
});
