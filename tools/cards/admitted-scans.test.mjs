import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';

test('every cards.json card has the pinned original, provenance and client URL', () => {
  const result = validateAdmittedScans();
  assert.ok(result.checkedCards > 0);
  assert.equal(new Set(result.checkedIds).size, result.checkedCards);
});

test('a pool source pointing to a different original is rejected', () => {
  const readWithWrongSource = filename => {
    const bytes = readFileSync(filename);
    if (!filename.endsWith('/rust-game/data/cards.json')) return bytes;
    const pool = JSON.parse(bytes);
    pool.cards.find(card => card.id === 'JC125').source.image = 'resource/ymsj-fun.github.io/cards/LC19 外科医生.jpg';
    return Buffer.from(JSON.stringify(pool));
  };
  assert.throws(() => validateAdmittedScans(undefined, readWithWrongSource), /JC125: pool original path differs/);
});
