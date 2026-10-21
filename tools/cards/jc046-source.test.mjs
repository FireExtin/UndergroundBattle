import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {validateAdmittedScans} from './validate-admitted-scans.mjs';

const root = new URL('../../', import.meta.url);
const read = name => readFileSync(new URL(name, root));
const json = name => JSON.parse(read(name));
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]))
    : value;
const digest = value => createHash('sha256').update(JSON.stringify(canonical(value))).digest('hex');
const spec = json('docs/factions/card-specifications.json').cards.JC046;
const pool = json('rust-game/data/cards.json');


test('JC046 complete nonunique face, kill 2 and mandatory reveal match pinned046/135 original', () => {
  // Source evidence remains the original source-only record; implementation
  // acceptance is not written back into its fields or edition evidence.
  assert.equal(digest({fields: spec.fields, sourceVerification: spec.sourceVerification, edition: spec.edition}), 'a2568741ea75534fd7830ecef3e76a87247bb6843eae1aabc7aa2113e87d1279');
  assert.deepEqual(spec.openQuestionIds, []);
  assert.deepEqual(spec.sourceVerification.blockingFields, []);
  const fields = spec.fields;
  const source = spec.sourceVerification;
  assert.equal(fields.nameInk, 'white');
  assert.deepEqual(fields.domains, [{id: 'blood', count: 1}]);
  assert.deepEqual(fields.printedRuleTextLines, ['杀伤2', '【现身强制触发】每位玩家各牺牲一个角色。']);
  const card = pool.cards.find(card => card.id === 'JC046');
  assert.ok(card);
  assert.deepEqual({...card, unique: card.unique ?? false}, {
    id: 'JC046', name: fields.name, kind: 'character', subtypes: ['恶魔', '异界'], cost: 5,
    loyalty: ['红色'], loyaltyText: '红色', color: '红', magic: '鲜血',
    icons: {permanent: fields.permanentIcons, temporary: fields.temporaryIcons}, defense: 3,
    text: fields.printedRuleTextLines.join('\n'), keywords: ['杀伤2'], unique: false,
    source: {file: 'docs/factions/card-specifications.json', record: 'JC046', image: source.imagePath, sha256: source.imageSha256},
  });
});

test('JC046 public scan is the byte-identical original and all admitted scans remain valid', () => {
  const original = read(spec.sourceVerification.imagePath);
  assert.equal(createHash('sha256').update(original).digest('hex'), '5d22023e0848d5d2e6566b2fdfe94ef62a80c7b97812f3f81066392aa97cc1ab');
  assert.equal(spec.sourceVerification.imageSha256, '5d22023e0848d5d2e6566b2fdfe94ef62a80c7b97812f3f81066392aa97cc1ab');
  assert.deepEqual(read('web/public/cards/JC046.jpg'), original);
  const result = validateAdmittedScans();


});
