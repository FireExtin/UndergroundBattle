// Select exact Native command records, then derive before views with real WASM.
// Usage: node tools/fixture-tools/current-bq030.mjs <evidenceRoot> <esmBindings>
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const [evidence, bindings] = process.argv.slice(2);
const root = fileURLToPath(new URL('../../', import.meta.url));
const abi = await import(pathToFileURL(resolve(bindings)));
abi.initSync({ module: readFileSync(new URL('./hegemony_wasm_bg.wasm', pathToFileURL(resolve(bindings)))) });
const catalog = JSON.parse(abi.catalog());
assert.equal(catalog.engineVersion, 'rust-v0.2.63-bq030-fixed-slots-candidate');
const records = readdirSync(resolve(evidence, 'room')).map(name => {
  const raw = readFileSync(resolve(evidence, 'room', name)), step = JSON.parse(raw);
  return { ...step, serverNowMs: '0', inputName: name,
    inputSha256: createHash('sha256').update(raw).digest('hex') };
}).sort((a, b) => Number(a.command.commandId.split('-').at(-1)) - Number(b.command.commandId.split('-').at(-1)));
const game = step => JSON.parse(step.state).game;
const afterGame = step => JSON.parse(step.expected.state).game;
const declaration = step => game(step).pending?.resolution?.Declare?.declaration;
const selected = step => step.command.action.action?.selected;
const observerChoice = step => declaration(step)?.ability?.key === 'attachment-observer-draw-one';
const offBoard = step => observerChoice(step) && !(game(step).regions ?? []).some(r =>
  r.cards.some(c => c.id === declaration(step).source.card.id));
const starts = [
  ['lethal-offboard-accept', s => offBoard(s) && selected(s)?.[0] === 'accept'],
  ['lethal-offboard-decline', s => offBoard(s) && selected(s)?.length === 0],
  ['multiple-observer-order', s => observerChoice(s) && game(s).pending.resolution.Declare.stage === 'AttachmentOrder' && selected(s)?.length === 1],
  ['empty-deck-elimination', s => !game(s).players[0].eliminated && afterGame(s).players[0].eliminated],
  ['granted-renown', s => declaration(s)?.ability.key === 'renown' && selected(s)?.[0] === 'accept' && game(s).world.length > 0],
  ['granted-combat-glory', s => declaration(s)?.ability.key === 'jc089-combat-glory' && selected(s)?.[0] === 'accept'],
  ['granted-renown-vacant', s => declaration(s)?.ability.key === 'renown' && selected(s)?.[0] === 'accept' && game(s).world.length === 0],
];
const scenarios = starts.map(([name, predicate]) => {
  const first = records.find(predicate); assert.ok(first, name);
  const steps = [first];
  for (;;) {
    const last = steps.at(-1), index = records.indexOf(last);
    const next = records.slice(index + 1).find(s => s.state === last.expected.state);
    if (!next || next.state === next.expected.state || steps.length >= 64) break;
    steps.push(next);
  }
  for (const step of steps) {
    assert.deepEqual(JSON.parse(abi.applyRoom(step.state, step.seat, JSON.stringify(step.command), '0')), step.expected);
    step.beforeViews = game(step).players.map((_, seat) => JSON.parse(abi.view(step.state, seat)));
    for (let seat = 0; seat < step.views.length; seat++) assert.deepEqual(JSON.parse(abi.view(step.expected.state, seat)), step.views[seat]);
  }
  return { name, steps };
});
const serial = JSON.stringify({ scenarios }) + '\n';
writeFileSync(resolve(root, 'sites/test/fixtures/bq030-native-v063.json'), serial);
writeFileSync(resolve(root, 'web/src/game/bq030Native63.fixture.json'), JSON.stringify({
  choices: scenarios.filter(s => !['empty-deck-elimination', 'granted-renown-vacant'].includes(s.name))
    .map(s => ({ name: s.name, step: s.steps[0] })) }) + '\n');
console.log(JSON.stringify({ scenarios: scenarios.length, steps: scenarios.map(s => ({ name: s.name, count: s.steps.length })) }));
