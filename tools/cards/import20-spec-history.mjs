// Test-only reconstruction: retain the historical whole-document hash while
// permitting only the exact, image-backed BQ064 and WM025/WM024 corrections.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
export const import20 = JSON.parse(readFileSync(new URL('../../docs/factions/import20-first-slice-2026-10-24.json', import.meta.url)));
export function priorSpecifications(current) {
  const prior = beforeVampireSpecifications(current);
  assert.equal(import20.sourceCorrections.length, 4);
  for (const correction of import20.sourceCorrections) {
    assert.equal(correction.id, 'BQ064');
    let target = prior.cards[correction.id];
    for (const key of correction.path.slice(0, -1)) target = target[key];
    const key = correction.path.at(-1);
    assert.deepEqual(target[key], correction.new);
    target[key] = correction.old;
  }
  return prior;
}

export const observerSlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-observers-2026-10-24.json', import.meta.url)));
export const currentImport20 = JSON.parse(readFileSync(new URL('../../docs/factions/import20-vampires-2026-10-24.json', import.meta.url)));
export function beforeVampireSpecifications(current) {
  const prior = structuredClone(current);
  assert.equal(currentImport20.sourceCorrections.length, 6);
  for (const correction of currentImport20.sourceCorrections) {
    assert(['WM025','WM024'].includes(correction.id));
    let target = prior.cards[correction.id];
    for (const key of correction.path.slice(0, -1)) target = target[key];
    const key = correction.path.at(-1);
    assert.deepEqual(target[key], correction.new);
    target[key] = correction.old;
  }
  return prior;
}
