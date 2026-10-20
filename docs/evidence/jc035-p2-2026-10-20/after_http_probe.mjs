import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {harness} from '/workspace/UndergroundBattle/sites/test/xq23-http-harness.mjs';
import * as abi from '/workspace/UndergroundBattle/rust-game-wasm/pkg/hegemony_wasm.js';

const root = '/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019';
const sha = value => createHash('sha256').update(value).digest('hex');
const bytes = readFileSync('/workspace/UndergroundBattle/rust-game-wasm/pkg/hegemony_wasm_bg.wasm');
const wasmSha256 = sha(bytes);
assert.notEqual(wasmSha256, '02e36dc9cd760213978dad67ebbcd95392793a64a5e9a5e108ec59eb4fcac640');
abi.initSync({module: bytes});
const native = JSON.parse(readFileSync(root + '/after/native-two-loaders.json', 'utf8'));
assert.equal(native.phase, 'after');
const worker = process.env.HEGEMONY_CURRENT_WORKER_DIR;
assert(worker && worker.includes('jc035-p2-worker'));
const workerSha256 = sha(readFileSync(worker + '/worker.js'));
assert.notEqual(workerSha256, '6dc7cf9e217f6be438b8b422e66a91ff725f837e45beacddab8a500a0108e567');

for (const [name, originalFile, invalidFile] of [
  ['missing-pending', 'original-room.json', 'missing-pending-room.json'],
  ['accepted-missing-frame', 'accepted-original-room.json', 'accepted-missing-frame-room.json'],
]) test(`JC035 P2 after-fix exact counterexample: ${name} rejects without D1 writes`, async t => {
  const original = readFileSync(root + '/before/' + originalFile, 'utf8');
  const state = readFileSync(root + '/before/' + invalidFile, 'utf8');
  const expectedError = native.stdout.split('\n').find(line => line.startsWith('ROOM_ERROR ' + name + ' '))?.slice(('ROOM_ERROR ' + name + ' ').length);
  assert(expectedError);
  assert.equal(native.observed[name].GameAccepted, false);
  assert.equal(native.observed[name].RoomAccepted, false);
  const views = Array.from({length: 4}, (_, seat) => JSON.parse(abi.view(original, seat)));
  const isNativeError = error => { assert.equal(String(error), expectedError); return true; };
  for (let seat = 0; seat < 4; seat++) assert.throws(() => abi.view(state, seat), isNativeError);
  const base = {expectedVersion: views[0].version, action: {kind: 'pauseRoom'}};
  assert.throws(() => abi.applyRoom(state, 0, JSON.stringify({...base, commandId: 'p2-after-wasm-' + name}), '0'), isNativeError);
  assert.throws(() => abi.pollRoom(state, 0, '0'), isNativeError);
  const h = await harness(t, {name, initial: {state, views}}), before = await h.snapshot();
  const responses = [];
  for (let reopen = 0; reopen < 2; reopen++) {
    if (reopen) await h.reopen();
    for (let seat = 0; seat < 4; seat++) {
      const result = await h.api('state', seat);
      assert.equal(result.status, 400);
      assert.equal(result.body.message, expectedError);
      responses.push({reopen, operation: 'state', seat, status: result.status});
    }
    const result = await h.api('commands', 0, {...base, commandId: 'p2-after-http-' + name + '-' + reopen});
    assert.equal(result.status, 400);
    assert.equal(result.body.message, expectedError);
    responses.push({reopen, operation: 'commands', seat: 0, status: result.status});
    assert.deepEqual(await h.snapshot(), before);
  }
  assert.equal(before.commands.length, 0);
  assert.equal(before.journal.length, 0);
  const summary = {
    case: name, phase: 'after-fix', invalidStateSha256: sha(state),
    exactSameInputAsBefore: true, nativeGameAndRoomRejected: true,
    wasmViewApplyAndPollRejected: true, wasmSha256, workerSha256, expectedError,
    httpResponses: responses, http400Count: responses.length,
    databaseSnapshotSha256: sha(JSON.stringify(before)),
    roomCommandsAndJournalUnchangedAcrossD1Reopen: true,
    committedCommands: 0, committedJournalRows: 0,
  };
  writeFileSync(root + '/after/' + name + '.http-wasm-rejection.json', JSON.stringify(summary, null, 2) + '\n', {flag: 'wx'});
  t.diagnostic(JSON.stringify(summary));
});
