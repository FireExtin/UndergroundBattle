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
export const vampireSlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-vampires-2026-10-24.json', import.meta.url)));
export const blackSlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-black-2026-10-24.json', import.meta.url)));
export const cultistSlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-cultist-2026-10-24.json', import.meta.url)));
export const thiefTattooSlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-thief-tattoo-2026-10-24.json', import.meta.url)));
export const industrySlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-industry-2026-10-24.json', import.meta.url)));
export const sentinelSlice = JSON.parse(readFileSync(new URL('../../docs/factions/import20-sentinel-2026-10-24.json', import.meta.url)));
export const currentImport20 = JSON.parse(readFileSync(new URL('../../docs/factions/import20-tax-ban-2026-10-24.json', import.meta.url)));
export function beforeVampireSpecifications(current) {
  const prior = beforeBlackSpecifications(current);
  assert.equal(vampireSlice.sourceCorrections.length, 6);
  for (const correction of vampireSlice.sourceCorrections) {
    assert(['WM025','WM024'].includes(correction.id));
    let target = prior.cards[correction.id];
    for (const key of correction.path.slice(0, -1)) target = target[key];
    const key = correction.path.at(-1);
    assert.deepEqual(target[key], correction.new);
    target[key] = correction.old;
  }
  return prior;
}

export function beforeBlackSpecifications(current) {
  const prior = beforeCultistSpecifications(current);
  assert.equal(blackSlice.sourceCorrections.length, 3);
  for (const correction of blackSlice.sourceCorrections) {
    assert.equal(correction.id, 'BQ081');
    let target = prior.cards[correction.id];
    for (const key of correction.path.slice(0, -1)) target = target[key];
    const key = correction.path.at(-1);
    assert.deepEqual(target[key], correction.new);
    target[key] = correction.old;
  }
  return prior;
}

export function beforeCultistSpecifications(current) {
  const prior = beforeThiefTattooSpecifications(current);
  assert.equal(cultistSlice.sourceCorrections.length, 2);
  for (const correction of cultistSlice.sourceCorrections) {
    assert.equal(correction.id, 'XG13');
    let target = prior.cards[correction.id];
    for (const key of correction.path.slice(0, -1)) target = target[key];
    const key = correction.path.at(-1);
    assert.deepEqual(target[key], correction.new);
    target[key] = correction.old;
  }
  return prior;
}

export function beforeThiefTattooSpecifications(current) {
  const prior = beforeIndustrySpecifications(current);
  assert.equal(thiefTattooSlice.sourceCorrections.length, 2);
  for (const correction of thiefTattooSlice.sourceCorrections) {
    assert.equal(correction.id, 'CQ24');
    let target = prior.cards[correction.id];
    for (const key of correction.path.slice(0, -1)) target = target[key];
    const key = correction.path.at(-1);
    assert.deepEqual(target[key], correction.new);
    target[key] = correction.old;
  }
  return prior;
}

export function beforeIndustrySpecifications(current) {
  const prior=structuredClone(current);
  assert.equal(industrySlice.sourceCorrections.length,2);
  for(const correction of industrySlice.sourceCorrections){
    assert.equal(correction.id,'BQ092');
    let target=prior.cards[correction.id];
    for(const key of correction.path.slice(0,-1))target=target[key];
    const key=correction.path.at(-1);
    assert.deepEqual(target[key],correction.new);target[key]=correction.old;
  }
  return prior;
}
