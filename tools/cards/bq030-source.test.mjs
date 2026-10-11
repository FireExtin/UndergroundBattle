// Read-only admission checks against the approved printed-source specification.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';

const repository = fileURLToPath(new URL('../../', import.meta.url));
// Pins were read once from approved 18f2a788455af7a1ac24316aa982462f895ded83:
// cards.json's 128-card array, non-version/non-card metadata, and the BQ030
// card-specifications.json {fields, sourceVerification, edition} projection.
// Encoding: UTF-8 JSON.stringify, recursively sorted object keys, array order retained.
// Historical Git objects are deliberately not required at test execution time.
const approvedPins = {
  sourceProjection: '5621da7059a4834bc0006bc001c68c7b2f6df3473302bbeaed185f24d68a584f',
  previousCards: '7645264ab0a059390a82d350f27a7277834bf85b8614c282b5fbb3c3f5905547',
  otherPoolMetadata: '5a331fc2c9cacb7f40248ca854ccc7245819162d675d3424130c060b22e48f58',
};
// Individual card-scans.json repair entries approved at
// e067b3b167367fc98562095abbd02db337fd9c5a, using the same canonical encoding.
const repairedEntryPins = {
  XQ44: 'd84de1dcebcb4e6605b274d38fa7ae2fb0d173af690a34d0a15f96a07fc32de5',
  JZ02: 'be020f99227ffb10ac692e56cb2d5220f32ebae11bb5f62e6dd84bc927a0c50f',
  JZ22: '884a378c1353db60c306f380682bf358d634e2a34d69b32c5a60c2dca147b603',
};
const originalImageSha256 = '85aaa7444b65060ab09f33f2ea9057647f88c961ff103fcca0c0763ce2564b86';
const specificationPath = 'docs/factions/card-specifications.json';
const read = relative => readFileSync(path.join(repository, relative));
const readJson = relative => JSON.parse(read(relative).toString('utf8'));
const canonical = value => {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
  }
  return value;
};
const digest = value => createHash('sha256').update(JSON.stringify(canonical(value))).digest('hex');

test('BQ030 registry exactly normalizes every approved printed gameplay field and original source', () => {
  const specification = readJson(specificationPath).cards.BQ030;
  // The approved document calls the gameplay projection "fields".
  const sourceGameplay = specification.fields;
  assert.equal(digest({ fields: sourceGameplay, sourceVerification: specification.sourceVerification,
    edition: specification.edition }), approvedPins.sourceProjection,
  'Printed gameplay fields, pinned source and edition identity must remain approved');
  assert.equal(specification.sourceVerification.status, 'completeGameplayFieldsForPinnedImage');
  assert.deepEqual(specification.sourceVerification.blockingFields, []);
  assert.equal(specification.sourceVerification.imageSha256, originalImageSha256);
  assert.equal(specification.cardId, 'BQ030');

  const {
    name, subtitle, nameInk, colorKey, basicType, subtypes, printedCollectorCode, seriesSymbol,
    printedCost, loyalty, domains, permanentIcons, temporaryIcons, defense, startingHand,
    influenceThreshold, points, printedKeywords, printedRuleTextLines, additionalPrintedSymbols,
    ...unhandledFields
  } = sourceGameplay;
  assert.deepEqual(unhandledFields, {}, 'Every printed field needs an explicit admission mapping');
  assert.equal(basicType, '角色');
  assert.equal(nameInk, 'white');
  assert.equal(subtitle, null);
  assert.equal(startingHand, null);
  assert.equal(influenceThreshold, null);
  assert.equal(points, null);
  assert.deepEqual(additionalPrintedSymbols, []);
  assert.equal(printedCollectorCode, specification.edition.printedCollectorCode);
  assert.equal(seriesSymbol, specification.edition.seriesSymbolObserved);
  assert.deepEqual(loyalty, [{ kind: 'color', value: '蓝', count: 1 }]);
  assert.deepEqual(domains, [{ id: 'blood', count: 1 }]);

  const cards = readJson('rust-game/data/cards.json').cards.filter(card => card.id === 'BQ030');
  assert.equal(cards.length, 1, 'BQ030 must have exactly one playable registry record');
  assert.deepEqual(cards[0], {
    id: specification.cardId,
    name,
    kind: 'character',
    subtypes,
    cost: printedCost,
    loyalty: ['蓝色'],
    loyaltyText: '蓝色',
    color: colorKey,
    magic: '鲜血',
    icons: { permanent: permanentIcons, temporary: temporaryIcons },
    defense,
    text: printedRuleTextLines.join('\n'),
    keywords: printedKeywords,
    unique: false,
    source: {
      file: specificationPath,
      record: specification.cardId,
      image: specification.sourceVerification.imagePath,
      sha256: originalImageSha256,
    },
  }, 'The complete runtime record must match the approved source normalization');
});

test('only BQ030 is admitted beyond the 128-card baseline and candidate version metadata agrees', () => {
  const pool = readJson('rust-game/data/cards.json');
  const { cards, rulesVersion, cardPoolVersion, engineVersion, ...otherData } = pool;
  assert.equal(cards.length, 129);
  assert.equal(new Set(cards.map(card => card.id)).size, 129, 'Registry IDs must be unique');
  assert.equal(cards.filter(card => card.id === 'BQ030').length, 1);
  const previousCards = cards.filter(card => card.id !== 'BQ030');
  assert.equal(previousCards.length, 128);
  assert.equal(digest(previousCards), approvedPins.previousCards,
    'Every previously admitted record and its ordering must match the approved 128-card baseline');
  assert.equal(digest(otherData), approvedPins.otherPoolMetadata,
    'Decks, world and source metadata must be preserved; candidate versions are checked separately');
  assert.equal(rulesVersion, 'hegemony-pdf-v1');
  assert.equal(cardPoolVersion, 'limited-v2.56-bq030-attachment-candidate');
  assert.equal(engineVersion, 'rust-v0.2.61-bq030-attachment-candidate');

  const catalog = read('rust-game/src/catalog.rs').toString('utf8');
  const constant = name => {
    const match = catalog.match(new RegExp(`pub const ${name}: &str = "([^"]+)";`));
    assert.ok(match, `${name} is missing from catalog.rs`);
    return match[1];
  };
  const productionEngine = catalog.match(
    /#\[cfg\(not\(feature = "society-fixtures"\)\)\]\s*pub const ENGINE_VERSION: &str = "([^"]+)";/);
  assert.ok(productionEngine, 'Production engine version must be distinct from the fixture version');
  assert.equal(constant('RULES_VERSION'), rulesVersion);
  assert.equal(constant('POOL_VERSION'), cardPoolVersion);
  assert.equal(productionEngine[1], engineVersion);
});

test('actual admitted scans retain pinned original bytes, published bytes and both BQ030 URL mappings', () => {
  // Reuse the existing byte/provenance/TS/public mapping checker instead of duplicating it.
  const result = validateAdmittedScans(repository);
  assert.equal(result.checkedCards, 129);
  assert.ok(result.checkedIds.includes('BQ030'));
});

test('XQ44, JZ02 and JZ22 repaired provenance entries remain exactly as reviewed at e067b3b', () => {
  const current = readJson('web/public/card-scans.json');
  for (const id of ['XQ44', 'JZ02', 'JZ22']) {
    assert.ok(current[id], `${id}: reviewed repair entry missing`);
    assert.equal(digest(current[id]), repairedEntryPins[id], `${id}: previously reviewed source repair changed`);
  }
});
