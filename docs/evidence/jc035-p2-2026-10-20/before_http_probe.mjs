// Reproduction assertions deliberately confirm the pre-fix vulnerability.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {harness} from '/workspace/UndergroundBattle/sites/test/xq23-http-harness.mjs';
import * as abi from './before/abi89-pkg/hegemony_wasm.js';

const root = '/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019';
const read = name => readFileSync(root + '/before/' + name, 'utf8');
const sha = value => createHash('sha256').update(value).digest('hex');
abi.initSync({module: readFileSync(root + '/before/abi89-pkg/hegemony_wasm_bg.wasm')});
assert.equal(sha(readFileSync(root + '/before/abi89-pkg/hegemony_wasm_bg.wasm')),
  '02e36dc9cd760213978dad67ebbcd95392793a64a5e9a5e108ec59eb4fcac640');

for (const [name, originalFile, invalidFile] of [
  ['missing-pending', 'original-room.json', 'missing-pending-room.json'],
  ['accepted-missing-frame', 'accepted-original-room.json', 'accepted-missing-frame-room.json'],
]) test(`P2 before-fix reproduction: ${name} is incorrectly accepted`, async t => {
  const original = read(originalFile), state = read(invalidFile);
  const views = Array.from({length: 4}, (_, seat) => JSON.parse(abi.view(original, seat)));
  const badViews = Array.from({length: 4}, (_, seat) => JSON.parse(abi.view(state, seat)));
  assert(badViews.every(view => !view.pendingChoice && !view.waitingChoice && !view.responseWindow));
  const command = {commandId: 'p2-before-' + name, expectedVersion: views[0].version, action: {kind: 'pauseRoom'}};
  const wasmCommand = JSON.parse(abi.applyRoom(state, 0, JSON.stringify(command), '0'));
  assert.equal(wasmCommand.outcome, 'accepted');
  assert.equal(wasmCommand.changed, true);
  const h = await harness(t, {name, initial: {state, views}});
  const before = await h.snapshot(), reads = [];
  for (let reopen = 0; reopen < 2; reopen++) {
    if (reopen) await h.reopen();
    for (let seat = 0; seat < 4; seat++) {
      const result = await h.api('state', seat);
      assert.equal(result.status, 200);
      assert(!result.body.pendingChoice && !result.body.waitingChoice && !result.body.responseWindow);
      reads.push({reopen, seat, status: result.status});
    }
    assert.deepEqual(await h.snapshot(), before);
  }
  const response = await h.api('commands', 0, command);
  assert.equal(response.status, 200, JSON.stringify(response.body));
  const after = await h.snapshot();
  assert.equal(after.commands.length, 1);
  assert.equal(after.journal.length, 1);
  assert.equal(after.room.version, before.room.version + 1);
  const summary = {
    case: name, phase: 'before-fix, vulnerability expected and observed',
    originalStateSha256: sha(original), invalidStateSha256: sha(state),
    wasmViewIncorrectlyAcceptedForSeats: 4, wasmPauseIncorrectlyAccepted: true,
    httpReads: reads, httpPauseStatus: response.status,
    readOnlySnapshotUnchangedAcrossReopen: true,
    committedCommands: after.commands.length, committedJournalRows: after.journal.length,
    beforeSnapshotSha256: sha(JSON.stringify(before)), afterSnapshotSha256: sha(JSON.stringify(after)),
    noPrivateAuthHeadersOrTokensRecorded: true,
  };
  writeFileSync(root + '/before/' + name + '.http-wasm-reproduction.json', JSON.stringify(summary, null, 2) + '\n', {flag: 'wx'});
  t.diagnostic(JSON.stringify(summary));
});
