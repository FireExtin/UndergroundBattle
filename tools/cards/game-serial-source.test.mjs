import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { import20, priorSpecifications } from './import20-spec-history.mjs';

const read = name => readFileSync(new URL('../../' + name, import.meta.url));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const baseline = JSON.parse(read('docs/evidence/game-serial-20261024/source-record-baselines.json'));

test('serial merge retains every incoming card and reconstructs the exact pre-correction specification', () => {
  const records = new Map(pool.cards.map(c => [c.id, sha(JSON.stringify(canonical(c)))]));
  assert.equal(records.size, 254);
  const expectedIds = new Set();
  for (const source of baseline.sources) {
    for (const [id, digest] of Object.entries(source.records)) {
      expectedIds.add(id);
      assert.equal(records.get(id), digest, `${source.sha}:${id}`);
    }
  }
  assert.equal(expectedIds.size, 252);
  assert.deepEqual(pool.cards.slice(252).map(c => c.id), ['BQ090', 'BQ064']);
  const prior = priorSpecifications(JSON.parse(read('docs/factions/card-specifications.json')));
  assert.equal(sha(JSON.stringify(prior, null, 2) + '\n'), baseline.specificationsSha256);
  assert.equal(pool.cards.filter(c => c.supported !== false).length, 213);
});

test('serial WIP has a unique single native identity and does not claim a matching released guest', () => {
  assert.equal(pool.engineVersion, import20.engineVersion);
  assert.equal(pool.cardPoolVersion, import20.cardPoolVersion);
  const plan = JSON.parse(read('docs/factions/batch20-fourth-2026-10-22.json'));
  assert.equal(plan.scope.nativePilots, 37);
  assert.equal(plan.scope.nativeWipCards, 2);
  assert.equal(plan.cards.find(c => c.id === 'WM043').fullNativeCardComplete, false);
  assert.equal(plan.developmentIdentity.wasmBuilt, false);
  assert.equal(plan.developmentIdentity.workerRun, false);
  assert.equal(plan.serialIntegration.integratedApproval, false);
  assert.equal(plan.scope.accepted, 0);
});
