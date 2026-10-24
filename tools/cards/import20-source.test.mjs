import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { import20, currentImport20, priorSpecifications } from './import20-spec-history.mjs';

const read = name => readFileSync(new URL('../../' + name, import.meta.url));
const sha = value => createHash('sha256').update(value).digest('hex');
const canonical = x => Array.isArray(x) ? x.map(canonical) : x && typeof x === 'object'
  ? Object.fromEntries(Object.keys(x).sort().map(k => [k, canonical(x[k])])) : x;
const pool = JSON.parse(read('rust-game/data/cards.json'));
const specifications = JSON.parse(read('docs/factions/card-specifications.json'));
const specs = specifications.cards;

test('first import slice preserves the exact prior252 and keeps both new cards unadmitted', () => {
  assert.equal(import20.priorCatalogCount, 252);
  assert.equal(sha(JSON.stringify(canonical(pool.cards.slice(0,252)))), import20.priorCatalogCardsCanonicalSha256);
  assert.deepEqual(pool.cards.slice(252,254).map(c => c.id), ['BQ090', 'BQ064']);
  assert.equal(new Set(pool.cards.map(c => c.id)).size, 258);
  assert.deepEqual(pool.cards.slice(256).map(c=>c.id), ['WM025','WM024']);
  assert.equal(pool.cards.filter(c => c.supported !== false).length, 213);
  for (const id of import20.nativePilotCards) assert.equal(pool.cards.find(c => c.id === id).supported, false);
  assert.equal(import20.current50ClosedAdded, 0);
  assert.equal(import20.publicAdmission, false);
  assert.equal(import20.publicPublished, false);
});

test('both printed programs retain exact images, costs, loyalty and fast/standard timing', () => {
  for (const [id, imageHash, cost, loyalty] of [
    ['BQ090','67ffb95243390ea8d1a9533830ef26241ec7625a1df314e203547b3804857c3e',2,['黑色']],
    ['BQ064','4ae03aeedb44416e0daa489cf4df68613dbf32ece07d6ac642db7169a6142e10',3,['灰色','灰色']],
  ]) {
    const row = pool.cards.find(c => c.id === id), spec = specs[id];
    assert.equal(sha(read(spec.sourceVerification.imagePath)), imageHash);
    assert.equal(sha(read(`web/public/cards/${id}.jpg`)), imageHash);
    assert.equal(row.source.imageSha256, imageHash);
    assert.equal(row.cost, cost); assert.deepEqual(row.loyalty, loyalty);
    assert.equal(row.text, spec.fields.printedRuleTextLines.join('\n'));
    assert.deepEqual(row.subtypes, spec.fields.subtypes);
  }
  assert.match(specs.BQ090.fields.printedRuleTextLines[1], /^快速行动/);
  assert.equal(specs.BQ064.fields.printedRuleTextLines[0], '隐秘+4');
  assert.match(specs.BQ064.fields.printedRuleTextLines[1], /^行动/);
});

test('four exact BQ064 corrections preserve every other byte of the historical source document', () => {
  assert.deepEqual(import20.sourceCorrections, [
    {id:'BQ064',path:['fields','printedRuleTextLines',0],old:'隐秘地区势力+4',new:'隐秘+4'},
    {id:'BQ064',path:['abilities',0,'printedText'],old:'隐秘地区势力+4',new:'隐秘+4'},
    {id:'BQ064',path:['fields','additionalPrintedSymbols',0,'interpretation'],old:'隐秘地区势力栏，独立于普通能力图标。',new:'隐秘对抗展示加值，独立于地区势力及普通能力图标。'},
    {id:'BQ064',path:['confirmedMechanismIds',0],old:'starlightInfluence',new:'mysticContest'},
  ]);
  const prior = priorSpecifications(specifications);
  assert.equal(sha(JSON.stringify(prior,null,2)+'\n'), import20.priorSpecificationsSha256);
  assert.equal(pool.sourceSha256, sha(read('docs/factions/card-specifications.json')));
});

test('one current native identity is distinct from published ABI95 and retained guest artifacts', () => {
  assert.equal(pool.engineVersion, currentImport20.engineVersion);
  assert.equal(pool.cardPoolVersion, currentImport20.cardPoolVersion);
  const rust = read('rust-game/src/catalog.rs').toString();
  assert(rust.includes(`ENGINE_VERSION: &str = "${pool.engineVersion}"`));
  assert(!rust.includes('rust-v0.2.99-game-serial'));
  assert.equal(import20.WASM, 'notBuiltOrRun'); assert.equal(import20.Worker, 'notBuiltOrRun');
  assert.equal(import20.browser, 'notRun'); assert.equal(import20.independentNativeReview, 'notRun');
});
