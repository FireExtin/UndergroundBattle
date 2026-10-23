import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const root = new URL('../../', import.meta.url);
const read = p => readFileSync(new URL(p, root));
const sha = b => createHash('sha256').update(b).digest('hex');
// Independently pinned from git show 5607a1c0; catalog hash uses compact JSON.
const baseline = {
  "rust-game/data/cards.json": "1d6b853c87259f46db3a9b7025c3f826ec2fa0ceb3c69160800a5b056cf5801a",
  "docs/factions/card-specifications.json": "809afcae7627ba828c533696c16b8dc5e69f1618bdc2653889955e4c296f9c9c",
  "docs/factions/batch20-fourth-2026-10-22.json": "9a33d2707ee56862316ba97c74c776dbb2ee9ee31236152ec2ce11c17c2ddea0",
  "rust-game/src/sentinel.rs": "1cc1fb4c63f93c5450f26f62e37c716ed7cf317c232c47832bb2fd96c400867f",
  "rust-game/src/batch4_intelligence.rs": "b14f4bff6ba1711e61a7fd77a995dd20771df43053aae0c1eef3f6f658eb6aa5",
  "resource/ymsj-fun.github.io/public/docs/帷幕之后说明书.jpg": "333388b60e719ee61a41a0009932b505315dff0042cf01edc0de4d8ac66f5761"
};
const pool = JSON.parse(read('rust-game/data/cards.json'));
const spec = JSON.parse(read('docs/factions/card-specifications.json')).cards.WM043;
const card = pool.cards.find(c => c.id === 'WM043');

test('WM043 original face and public image are the exact reviewed bytes', () => {
  const expected = '887f3be501c9dc2b64bca87889c5ba76a870cd75a8db401c0325cd0ea774445f';
  assert.equal(sha(read(spec.sourceVerification.imagePath)), expected);
  assert.equal(spec.sourceVerification.imageSha256, expected);
  assert.equal(sha(read('web/public/cards/WM043.jpg')), expected);
  assert.equal(card.source.imageSha256, expected);
});
test('WM043 catalog preserves the full printed behavior and remains unadmitted', () => {
  assert.equal(card.name, spec.fields.name); assert.equal(card.cost, spec.fields.printedCost);
  assert.equal(card.color, '灰'); assert.equal(card.kind, 'spell');
  assert.deepEqual(card.loyalty, ['灰色']); assert.deepEqual(card.subtypes, ['策略']);
  assert.deepEqual(card.icons.permanent, spec.fields.permanentIcons);
  assert.deepEqual(card.icons.temporary, spec.fields.temporaryIcons);
  assert.equal(card.defense, null); assert.equal(card.unique, false); assert.equal(card.supported, false);
  assert.deepEqual(card.text.split('\n'), spec.fields.printedRuleTextLines);
  assert.equal(pool.cards.filter(c => c.id === 'WM043').length, 1);
});
test('WM043 historical baseline remains exact after excluding only the other reviewed additions', () => {
  const reviewedAdds = new Set(['WM043', 'XQ20', 'LC16', 'WM037']);
  const historical = {...pool, engineVersion:'rust-v0.2.95-batch20-fourth', cardPoolVersion:'limited-v2.86-batch20-fourth', cards:pool.cards.filter(c => !reviewedAdds.has(c.id))};
  assert.equal(sha(JSON.stringify(historical)), baseline['rust-game/data/cards.json']);
  assert.equal(sha(read('docs/evidence/game-serial-20261024/batch-plan-5607.json')), baseline['docs/factions/batch20-fourth-2026-10-22.json']);
  for (const p of ['docs/factions/card-specifications.json',
    'rust-game/src/sentinel.rs', 'rust-game/src/batch4_intelligence.rs']) {
    assert.equal(sha(read(p)), baseline[p], p);
  }
});
test('formal marker and shown-card rule sources retain the pinned original bytes', () => {
  assert.equal(sha(read('resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf')),
    '2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35');
  const p = 'resource/ymsj-fun.github.io/public/docs/帷幕之后说明书.jpg';
  assert.equal(sha(read(p)), baseline[p]);
});
