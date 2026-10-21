// One current-catalog boundary replaces cumulative per-card assertions that
// incorrectly treated the changing registry as an immutable historical catalog.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';

const read = name => readFileSync(new URL('../../' + name, import.meta.url));
const data = JSON.parse(read('rust-game/data/cards.json'));
const specs = JSON.parse(read('docs/factions/card-specifications.json')).cards;
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const digest = value => createHash('sha256').update(JSON.stringify(canonical(value))).digest('hex');
const ids = ['JZ33', 'JZ37', 'JC043', 'BQ054', 'JC048', 'WM034', 'BQ091', 'JC068', 'BQ094', 'JZ52',
  'BQ080', 'JZ57', 'JC033', 'JC038', 'CQ16', 'JC034', 'JC040', 'JC067', 'JC065', 'JZ29'];
const correctedIcons = {
  JC029: { permanent: { investigation: 0, combat: 0, influence: 0 }, temporary: { investigation: 0, combat: 1, influence: 1 } },
  JZ49: { permanent: { investigation: 0, combat: 0, influence: 0 }, temporary: { investigation: 0, combat: 2, influence: 0 } },
  JC044: { permanent: { investigation: 1, combat: 1, influence: 1 }, temporary: { investigation: 0, combat: 0, influence: 0 } },
};

test('only the 12 explicitly reviewed source corrections differ from the d98117d specification baseline', () => {
  const reviewed = ['JC029', 'JC034', 'JC038', 'JC044', 'JC048', 'JC055', 'JC065', 'JZ49', 'JZ52', 'LC16', 'WM033', 'XG12'];
  assert.equal(Object.keys(specs).length, 683);
  const unchanged = Object.fromEntries(Object.entries(specs).filter(([id]) => !reviewed.includes(id)));
  assert.equal(digest(unchanged), '0f0931bf8474250ea6f914b0be958dd37a2d4a52af566fb76dfe895052cea055');
});

test('batch20 changes only its 20 admissions, three verified icon fields and current version identity', () => {
  assert.equal(data.cards.length, 173);
  assert.equal(new Set(data.cards.map(card => card.id)).size, 173);
  assert.deepEqual(data.cards.slice(153).map(card => card.id), ids);
  // Pin read independently from d98117d188cfc65f4e29fb757dfcf9accdd545e5.
  // All fields and ordering of its 153 cards remain protected, except these
  // three icon fields checked explicitly against the original images below.
  const prior = data.cards.slice(0, 153).map(card => {
    if (!Object.hasOwn(correctedIcons, card.id)) return card;
    const { icons, ...unchanged } = card;
    assert.deepEqual(icons, correctedIcons[card.id], card.id);
    const fields = specs[card.id].fields;
    assert.deepEqual(icons, { permanent: fields.permanentIcons, temporary: fields.temporaryIcons });
    return unchanged;
  });
  assert.equal(digest(prior), '46ae8563bddc922ff124e7ce0ea6938ec584a84921d8f33a2299a2a8b5b2adac');
  const { cards, cardPoolVersion, engineVersion, ...metadata } = data;
  assert.equal(digest(metadata), 'a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0');
  assert.equal(cardPoolVersion, 'limited-v2.82-batch20');
  assert.equal(engineVersion, 'rust-v0.2.92-society-zone-fix');
  const rust = read('rust-game/src/catalog.rs').toString();
  const constant = name => rust.match(new RegExp(`pub const ${name}: &str = "([^"]+)";`))?.[1];
  assert.equal(constant('RULES_VERSION'), data.rulesVersion);
  assert.equal(constant('POOL_VERSION'), cardPoolVersion);
  assert.equal(rust.match(/#\[cfg\(not\(feature = "society-fixtures"\)\)\]\s*pub const ENGINE_VERSION: &str = "([^"]+)";/)?.[1], engineVersion);
});

test('all 20 admitted prints retain their independently reviewed names, costs, factions, icons and keywords', () => {
  const colors = {};
  for (const id of ids) {
    const card = data.cards.find(card => card.id === id), fields = specs[id].fields;
    assert.equal(card.supported, true, id);
    assert.equal(card.name, fields.name, id);
    assert.equal(card.cost, fields.printedCost, id);
    assert.equal(card.color, fields.colorKey, id);
    assert.deepEqual(card.subtypes, fields.subtypes, id);
    assert.deepEqual(card.icons, { permanent: fields.permanentIcons, temporary: fields.temporaryIcons }, id);
    assert.deepEqual(card.keywords, fields.printedKeywords, id);
    colors[card.color] = (colors[card.color] || 0) + 1;
  }
  assert.deepEqual(colors, { 红: 5, 灰: 5, 黑: 5, 蓝: 5 });
});

test('all 173 admitted scans retain exact original bytes and both public mappings', () => {
  const result = validateAdmittedScans();
  assert.equal(result.checkedCards, 173);
  assert.equal(result.manifestEntries, 185);
  for (const id of ids) assert(result.checkedIds.includes(id), id);
});
