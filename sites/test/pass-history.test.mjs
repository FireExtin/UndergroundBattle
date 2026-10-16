import test from 'node:test';
import assert from 'node:assert/strict';
import { capturePasses, passRecord, publicPassRecord } from '../src/pass-history.mjs';

const view = { turn: 7, phase: 'confrontation', step: 'region:1:combat:before', serverNowMs: 1000,
  players: [{ seat: 0, id: 'p0', name: '甲' }, { seat: 1, id: 'p1', name: '乙' }],
  regions: [{ index: 1, id: 'original-region', name: '旧港' }],
  hand: [{ name: 'PRIVATE HAND' }], privateDeckTop: { cardId: 'PRIVATE TOP' },
  responseWindow: { id: 'response:2', myIntentId: 'PRIVATE INTENT', members: [
    { playerId: 'p0', status: 'undecided', deadlineMs: 1500 },
    { playerId: 'p1', status: 'composing' },
  ] } };
const command = (kind, seat = 0) => ({ Command: { seat, command: { action: kind === 'pass' ? { kind: 'game', action: { kind } } : { kind } } } });

test('accepted pass context keeps public event-time actor, step and original region, without private fields', () => {
  const record = passRecord(view, 0, 14, 0, 'response');
  assert.deepEqual(record.region, { index: 1, instanceId: 'original-region', name: '旧港' });
  assert.equal(record.playerName, '甲'); assert.equal(record.step, 'region:1:combat:before');
  const changed = structuredClone(view); changed.regions[0].id = 'replacement'; changed.regions[0].name = '新城'; changed.phase = 'end';
  assert.deepEqual(record.region, { index: 1, instanceId: 'original-region', name: '旧港' });
  assert(!JSON.stringify(record).includes('PRIVATE'));
  assert.deepEqual(publicPassRecord({ ...record, hand: ['PRIVATE'], token: 'PRIVATE' }, 14), record);
});
test('nonregional windows are global even when the stack has a region target', () => {
  for (const [phase, step] of [['start','prepare'], ['start','draw'], ['action','team:1'], ['confrontation','mobility'], ['end','end']]) {
    assert.equal(passRecord({ ...view, phase, step, stack: [{ targetRegion: 1 }] }, 1, 14, 0, 'pass').region, null);
  }
});
test('manual, response, cancel and timeout records use only accepted journal events', () => {
  for (const [kind, reason] of [['pass','pass'], ['passResponse','response'], ['cancelAndPass','cancelResponse']]) {
    const records = capturePasses({ view: () => JSON.stringify(view) }, 'opaque', { seat: 0, version: 14, journal: [command(kind)] });
    assert.equal(records.length, 1); assert.equal(records[0].reason, reason);
  }
  assert.deepEqual(capturePasses({}, 'opaque', { journal: [] }), []);
  assert.deepEqual(capturePasses({}, 'opaque', { journal: [command('beginResponse')] }), []);
  const expired = capturePasses({ view: () => JSON.stringify(view) }, 'opaque', { seat: 0, version: 14, journal: [{ Tick: { server_now_ms: 2000 } }] });
  assert.deepEqual(expired.map(record => [record.playerId, record.reason]), [['p0','timeout']]);
});
test('a command after Tick uses the post-timeout phase instead of borrowing the prior region', () => {
  const afterTick = { ...view, phase: 'end', step: 'end', responseWindow: null };
  const records = capturePasses({ view: () => JSON.stringify(view), pollRoom: state => {
    assert.equal(state, 'OPAQUE-u64-18446744073709551615');
    return JSON.stringify({ changed: true, version: 14, view: afterTick });
  } }, 'OPAQUE-u64-18446744073709551615', { seat: 0, version: 14, journal: [{ Tick: { server_now_ms: 2000 } }, command('pass', 1)] });
  assert.equal(records.length, 2); assert.equal(records[0].region.name, '旧港');
  assert.equal(records[1].phase, 'end'); assert.equal(records[1].region, null);
  assert.equal(new Set(records.map(record => record.id)).size, 2);
});
test('inconsistent public metadata fails closed or is omitted without exposing injected objects', () => {
  assert.throws(() => passRecord({ ...view, regions: [] }, 0, 14, 0, 'pass'));
  assert.throws(() => capturePasses({ view: () => JSON.stringify(view) }, 'opaque', { seat: 0, version: 14, journal: [{ Tick: { server_now_ms: 500 } }] }));
  const record = passRecord(view, 0, 14, 0, 'response');
  for (const bad of [{ ...record, version: 15 }, { ...record, playerId: 'private-object' }, { ...record, region: { name: 'PRIVATE' } }]) assert.equal(publicPassRecord(bad, 14), null);
});
