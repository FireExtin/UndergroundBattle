import assert from 'node:assert/strict';
import { publicPassRecord } from '../src/pass-history.mjs';

// Preserve the entire reducer event comparison while allowing only the new,
// explicitly public metadata beside it. No opaque Native oracle is rewritten.
export function assertSessionJournal(raw, events, message) {
  const entry = JSON.parse(raw);
  assert.deepEqual(Object.keys(entry), ['SessionEvents'], message);
  const session = entry.SessionEvents;
  assert.deepEqual(Object.keys(session).sort(), session.publicPasses ? ['events','publicPasses'] : ['events'], message);
  assert.deepEqual(session.events, events, message);
  if (session.publicPasses) {
    assert(session.publicPasses.length > 0 && session.publicPasses.length <= 5, message);
    for (const record of session.publicPasses) assert.deepEqual(publicPassRecord(record, record.version), record, message);
  }
}

export function assertPublicView(actual, expected, message) {
  const { passHistory, ...core } = actual;
  assert.deepEqual(core, expected, message);
  if (passHistory !== undefined) {
    assert(Array.isArray(passHistory) && passHistory.length > 0 && passHistory.length <= 50, message);
    assert.equal(new Set(passHistory.map(record => record.id)).size, passHistory.length, message);
    let previous = 0;
    for (const record of passHistory) {
      assert.deepEqual(publicPassRecord(record, record.version), record, message);
      assert(record.version >= previous && record.version <= expected.version, message);
      previous = record.version;
    }
  }
}
