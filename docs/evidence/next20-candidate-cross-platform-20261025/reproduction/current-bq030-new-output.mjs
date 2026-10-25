// Select exact Native commands and views, checking all of them with current WASM.
// Usage: node tools/fixture-tools/current-bq030.mjs <evidenceRoot> <esmBindings>
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const [evidence, bindings] = process.argv.slice(2);
const webOnly = process.argv.includes("--web-only");
const root = "/home/ddd/Documents/Codex/2026-10-24/task-7/candidate-cross-platform-20261025/raw-selectors";
const abi = await import(pathToFileURL(resolve(bindings)));
abi.initSync({ module: readFileSync(new URL('./hegemony_wasm_bg.wasm', pathToFileURL(resolve(bindings)))) });
const catalog = JSON.parse(abi.catalog());
const pool = JSON.parse(readFileSync(resolve(root, 'rust-game/data/cards.json')));
assert.equal(catalog.engineVersion, pool.engineVersion);
const suffix = catalog.engineVersion.match(/^rust-v0\.2\.(\d+)-/)[1];
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
    assert.equal(step.beforeViews.length, game(step).players.length, 'Export Native before-views; never invent them');
    step.beforeViews.forEach((view, seat) => assert.deepEqual(JSON.parse(abi.view(step.state, seat)), view));
    for (let seat = 0; seat < step.views.length; seat++) assert.deepEqual(JSON.parse(abi.view(step.expected.state, seat)), step.views[seat]);
  }
  return { name, steps };
});
const serial = JSON.stringify({ scenarios }) + '\n';
if (!webOnly) writeFileSync(resolve(root, `sites/test/fixtures/bq030-native-v0${suffix}.json`), serial, {flag: 'wx'});
writeFileSync(resolve(root, `web/src/game/bq030Native${suffix}.fixture.json`), JSON.stringify({
  choices: scenarios.filter(s => !['empty-deck-elimination', 'granted-renown-vacant'].includes(s.name))
    .map(s => ({ name: s.name, step: s.steps[0] })) }) + '\n', {flag: 'wx'});
console.log(JSON.stringify({ scenarios: scenarios.length, steps: scenarios.map(s => ({ name: s.name, count: s.steps.length })) }));
