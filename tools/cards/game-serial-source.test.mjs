import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';

const read = name => readFileSync(new URL('../../' + name, import.meta.url));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const baseline = JSON.parse(read('docs/evidence/game-serial-20261024/source-record-baselines.json'));

test('serial merge retains every exact incoming card record and the unchanged specification', () => {
  const records = new Map(pool.cards.map(c => [c.id, sha(JSON.stringify(canonical(c)))]));
  assert.equal(records.size, 252);
  const expectedIds = new Set();
  for (const source of baseline.sources) {
    for (const [id, digest] of Object.entries(source.records)) {
      expectedIds.add(id);
      assert.equal(records.get(id), digest, `${source.sha}:${id}`);
    }
  }
  assert.equal(expectedIds.size, records.size);
  assert.equal(sha(read('docs/factions/card-specifications.json')), baseline.specificationsSha256);
  assert.equal(pool.cards.filter(c => c.supported !== false).length, 213);
});

test('serial WIP has a unique single native identity and does not claim a matching released guest', () => {
  assert.equal(pool.engineVersion, 'rust-v0.2.99-game-serial');
  assert.equal(pool.cardPoolVersion, 'limited-v2.90-game-serial');
  const plan = JSON.parse(read('docs/factions/batch20-fourth-2026-10-22.json'));
  assert.equal(plan.scope.nativePilots, 37);
  assert.equal(plan.scope.nativeWipCards, 2);
  assert.equal(plan.cards.find(c => c.id === 'WM043').fullNativeCardComplete, false);
  assert.equal(plan.developmentIdentity.wasmBuilt, false);
  assert.equal(plan.developmentIdentity.workerRun, false);
  assert.equal(plan.serialIntegration.integratedApproval, false);
  assert.equal(plan.scope.accepted, 0);
});
