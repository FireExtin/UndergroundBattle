// XQ24 admission is pinned independently of the historical source-only catalog.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { validateAdmittedScans } from './validate-admitted-scans.mjs';

const root = new URL('../../', import.meta.url);
const read = name => readFileSync(new URL(name, root));
const json = name => JSON.parse(read(name));
const canonical = v => Array.isArray(v) ? v.map(canonical) : v && typeof v === 'object'
  ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canonical(v[k])])) : v;
const digest = v => createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');

test('XQ24 matches its reviewed original and preserves the entire 7809971 baseline', () => {
  const spec = json('docs/factions/card-specifications.json').cards.XQ24;
  assert.equal(digest({ fields: spec.fields, sourceVerification: spec.sourceVerification, edition: spec.edition }),
    '733a75092c102e631f1e96971c04250047e5349853c43aa37bb826b1ce0ea5d5');
  const pool = json('rust-game/data/cards.json');
  const { cards, rulesVersion, cardPoolVersion, engineVersion, ...metadata } = pool;
  assert.equal(cards.length, 130);
  assert.equal(new Set(cards.map(c => c.id)).size, 130);
  assert.equal(digest(cards.filter(c => c.id !== 'XQ24')),
    'da45496e0eb1497cb141809f4d3aaa1b173a15983497f65d0a957230caf6a556');
  assert.equal(digest(metadata), '5a331fc2c9cacb7f40248ca854ccc7245819162d675d3424130c060b22e48f58');
  assert.equal(rulesVersion, 'hegemony-pdf-v1');
  assert.equal(cardPoolVersion, 'limited-v2.57-xq24-riot-police-candidate');
  assert.equal(engineVersion, 'rust-v0.2.64-xq24-riot-police-candidate');
  const f = spec.fields, source = spec.sourceVerification;
  assert.deepEqual(cards.find(c => c.id === 'XQ24'), {
    id: 'XQ24', name: f.name, kind: 'character', subtypes: f.subtypes,
    cost: f.printedCost, loyalty: ['灰色', '灰色'], loyaltyText: '灰色 / 灰色',
    color: f.colorKey, magic: '', icons: { permanent: f.permanentIcons, temporary: f.temporaryIcons },
    defense: f.defense, text: f.printedRuleTextLines.join('\n'), keywords: f.printedKeywords,
    unique: false, source: { file: 'docs/factions/card-specifications.json', record: 'XQ24',
      image: source.imagePath, sha256: source.imageSha256 }
  });
  assert.deepEqual(spec.openQuestionIds, []);
  assert.equal(spec.sourceVerification.status, 'completeGameplayFieldsForPinnedImage');
  assert.deepEqual(spec.sourceVerification.blockingFields, []);
  assert.equal(spec.selectionEnabled, false);
  assert.equal(spec.gameAcceptance, 'notAccepted');
  assert.equal(spec.strategyUiPlaytest, 'notRun');
  assert.equal(spec.edition.printedCollectorCode, '24/50');
  assert.equal(spec.edition.releaseYearClaimed, false);
});

test('the rule pages and analogous FAQ retain the exact locally reviewed source bytes', () => {
  // A pinned analogy is evidence for a question, never a direct XQ24 ruling.
  for (const [file, sha256] of [
    ['隐秘世界规则手册.pdf', '2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35'],
    ['霸权说明书.pdf', 'a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886'],
    ['隐秘世界勘误及释疑.pdf', '916aa6a9f50ac7e737b517358e0c4270f715d0ed7391d55c8fcb629392a71d38']
  ]) assert.equal(createHash('sha256').update(read(`resource/ymsj-fun.github.io/public/docs/${file}`)).digest('hex'), sha256);
});

test('XQ24 original, published card face and both client mappings agree byte for byte', () => {
  const original = read('resource/ymsj-fun.github.io/cards/XQ24 防暴警察.jpg');
  assert.equal(createHash('sha256').update(original).digest('hex'),
    '420b412908d7b86112e26c2ba0f94593a29c40bf8f6b54e8809ce489a66f61b8');
  assert.deepEqual(read('web/public/cards/XQ24.jpg'), original);
  const result = validateAdmittedScans();
  assert.equal(result.checkedCards, 130);
  assert.ok(result.checkedIds.includes('XQ24'));
});
