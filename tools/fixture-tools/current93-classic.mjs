// Assemble current Native exports. Never load, relabel or replay an older fixture.
// Usage: node tools/fixture-tools/current93-classic.mjs <evidence-root>
//        [--groups=xq37,jz22,...] [--inputs=/absolute/directory-map.json]
// An input map may override each group's native/trace/invalid/room/restart directories.
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import path from 'node:path';
import {catalog} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog, currentIdentity, currentSourceManifestSha256,
  currentWasmSha256, sha256, writeCurrentFixture} from '../current-native-source.mjs';

const [rootArgument, ...options] = process.argv.slice(2);
assert(rootArgument, 'Provide the fresh Native evidence root.');
assert.equal(currentIdentity.engineVersion, 'rust-v0.2.93-batch20-next');
const root = path.resolve(rootArgument);
const option = name => options.find(value => value.startsWith(`--${name}=`))?.slice(name.length + 3);
assert(options.every(value => /^--(groups|inputs)=.+/.test(value)), 'Unknown assembler argument.');
const overrides = option('inputs') ? JSON.parse(readFileSync(option('inputs'), 'utf8')) : {};
const directory = (group, kind) => path.resolve(overrides[group]?.[kind] || path.join(root, group, kind));
const expectedVersions = {rules: currentIdentity.rulesVersion,
  cardPool: currentIdentity.cardPoolVersion, engine: currentIdentity.engineVersion};
const opaqueFields = new Set(['state', 'before', 'after', 'initialState', 'terminalState', 'restartState', 'nextChoiceState']);

function audit(value, label) {
  if (Array.isArray(value)) return value.forEach((item, index) => audit(item, `${label}[${index}]`));
  if (!value || typeof value !== 'object') return;
  if (value.versions) assert.deepEqual(value.versions, expectedVersions, `${label}: stale Native identity`);
  if (value.catalog) assertCurrentCatalog(value.catalog);
  for (const [key, child] of Object.entries(value)) {
    if (opaqueFields.has(key) && typeof child === 'string' && child.startsWith('{')) {
      // Read only to reject old inputs. The original string is returned unchanged.
      const saved = JSON.parse(child);
      assert.deepEqual(saved.versions, expectedVersions, `${label}.${key}: stale opaque state`);
      assert.deepEqual(saved.game.versions, expectedVersions, `${label}.${key}: stale game identity`);
    } else if (key !== 'catalog') audit(child, `${label}.${key}`);
  }
}
function read(file) {
  const bytes = readFileSync(file), value = JSON.parse(bytes);
  audit(value, file);
  return value;
}
function files(dir, pattern = /\.json$/) {
  return readdirSync(dir, {withFileTypes: true}).flatMap(entry => entry.isDirectory()
    ? files(path.join(dir, entry.name), pattern)
    : pattern.test(entry.name) ? [path.join(dir, entry.name)] : []).sort();
}
function records(group, kind, pattern) {
  return files(directory(group, kind), pattern).map(file => {
    const value = read(file);
    return {...value, inputName: path.relative(directory(group, kind), file),
      inputSha256: sha256(readFileSync(file))};
  });
}
const raw = (group, kind, name) => read(path.join(directory(group, kind), name));
const steps = group => records(group, 'trace', /^step-\d+\.json$/);
const checkpoints = group => records(group, 'trace', /^checkpoint-\d+\.json$/);
const game = row => JSON.parse(row.state).game;
const after = row => JSON.parse(row.expected.state).game;
const action = row => row.command?.action?.action;
const selected = row => action(row)?.selected;
const declaration = row => game(row).pending?.resolution?.Declare?.declaration;
const top = row => game(row).stack.at(-1)?.frame;
const source = row => declaration(row)?.source?.card?.definition;
const accepted = row => row.expected?.outcome === 'accepted' && !row.expected.errorCode;
function pick(rows, predicate, label) {
  const candidates = rows.filter(row => accepted(row) && predicate(row));
  assert(candidates.length, `Missing current Native sample: ${label}`);
  return candidates[0];
}
function beforeViews(row, snapshots, label) {
  const same = snapshots.filter(snapshot => snapshot.state === row.state);
  assert(same.length, `Missing byte-identical Native before views: ${label}`);
  for (const snapshot of same) assert.deepEqual(snapshot.views, same[0].views, label);
  return same[0].views;
}
function trace(value, label) {
  assert(value.steps?.length, `${label}: missing Native steps`);
  let previous = value.initialState;
  for (const step of value.steps) {
    assert(step.transition?.state && Array.isArray(step.views), `${label}: hash-only export is insufficient`);
    if (step.state !== undefined) assert.equal(step.state, previous, label);
    assert.equal(step.transition.view.version, step.transition.version, label);
    const returned = step.transition.view, snapshot = step.views[step.seat];
    if (returned.serverNowMs !== snapshot.serverNowMs) {
      // An unchanged request projects its supplied time; the Native checkpoint
      // may project the last persisted time. Preserve both original projections.
      assert.equal(step.transition.changed, false, `${label}: changed transition view time differs`);
      assert.equal(returned.serverNowMs, Number(step.serverNowMs), `${label}: request view time differs`);
      const saved = JSON.parse(step.transition.state);
      assert.equal(snapshot.serverNowMs, saved.pacing.lastServerNowMs, `${label}: persisted view time differs`);
      assert.deepEqual({...returned, serverNowMs: snapshot.serverNowMs}, snapshot, label);
    } else assert.deepEqual(returned, snapshot, label);
    previous = step.transition.state;
  }
  if (value.terminalState) assert.equal(value.terminalState, previous, label);
  return value;
}
function emit(name, value) {
  audit(value, name);
  writeCurrentFixture(`rust-game-wasm/tests/current93/${name}.json.gz`, value);
  process.stdout.write(`${name}: current Native fixture written\n`);
}

const producers = {
  xq37() {
    const rows = steps('xq37');
    const entry = row => source(row) === 'XQ37' && declaration(row).ability.key === 'entry-existing-influence' && row.seat === 0;
    emit('xq37', {
      accept: pick(rows, row => entry(row) && selected(row)?.[0] === 'accept', 'XQ37 accept'),
      decline: pick(rows, row => entry(row) && selected(row)?.length === 0, 'XQ37 decline'),
      placement: raw('xq37', 'native', 'resolve.json'),
    });
    emit('xq37-invalid-rootface', raw('xq37', 'invalid', 'invalid-18.json'));
  },
  xq27() {
    emit('xq27', raw('xq27', 'native', 'resolve.json'));
    emit('xq27-invalid-rootface', raw('xq27', 'invalid', 'invalid-12.json'));
  },
  jc050() {
    emit('jc050', raw('jc050', 'native', 'choose.json'));
    emit('jc050-quote', raw('jc050', 'native', 'quote.json'));
  },
  jz22() {
    const rows = steps('jz22');
    const entry = row => source(row) === 'JZ22' && declaration(row).ability.key === 'entry-low-hand-influence' && row.seat === 0;
    const enemySubmit = pick(rows, row => row.command.action.kind === 'submitResponse'
      && action(row)?.kind === 'play' && top(row)?.source.card.definition === 'JZ22'
      && game(row).players[row.seat].hand.some(card => card.id === action(row).cardId && card.definition === 'JC008'), 'JZ22 enemy paid response');
    const enemyCompose = pick(rows, row => row.command.action.kind === 'beginResponse'
      && row.expected.state === enemySubmit.state, 'JZ22 real beginResponse predecessor');
    emit('jz22', {
      placement: pick(rows, row => row.command.action.kind === 'passResponse'
        && top(row)?.ability_key === 'entry-low-hand-influence'
        && after(row).regions[2].influence.reduce((a, b) => a + b, 0)
          - game(row).regions[2].influence.reduce((a, b) => a + b, 0) === 1, 'JZ22 influence placement'),
      accept: pick(rows, row => entry(row) && selected(row)?.[0] === 'accept', 'JZ22 accept'),
      decline: pick(rows, row => entry(row) && selected(row)?.length === 0, 'JZ22 decline'),
      deploy: pick(rows, row => action(row)?.kind === 'deploy'
        && game(row).players[row.seat].hand.some(card => card.id === action(row).cardId && card.definition === 'JZ22'), 'JZ22 deployment'),
      glory: pick(rows, row => source(row) === 'JZ22' && declaration(row).ability.key === 'jc089-combat-glory'
        && selected(row)?.[0] === 'accept', 'JZ22 granted combat glory'),
      enemyCompose, enemySubmit,
    });
  },
  death() {
    const names = ['priestMarks', 'casterDiscardGuard', 'casterRandomTrigger', 'casterDiscard',
      'casterTrigger', 'contractWinWindow', 'contractTrigger', 'casterRandomStack'];
    const snapshots = Object.fromEntries(names.map(name => [name, raw('death', 'native', `${name}.json`)]));
    emit('three-color-death', snapshots);
    const rows = steps('death');
    const declared = pick(rows, row => row.state === snapshots.casterRandomTrigger.state
      && selected(row)?.[0] === 'p1', 'death observer declaration');
    const random = pick(rows, row => top(row)?.source.card.definition === 'JC031'
      && top(row).source.observed_death?.vampire && after(row).players[1].hand.length < game(row).players[1].hand.length,
    'death observer random discard');
    const influence = pick(rows, row => top(row)?.source.card.definition === 'JC090'
      && after(row).regions.some((region, index) => region.influence.reduce((a, b) => a + b, 0)
        > game(row).regions[index].influence.reduce((a, b) => a + b, 0)), 'death observer influence');
    const discard = pick(rows, row => row.state === snapshots.casterDiscard.state && selected(row)?.length === 1,
      'death observer ordinary private discard');
    const marker = pick(rows, row => top(row)?.source.card.definition === 'JC045'
      && after(row).regions.flatMap(region => region.cards).some(card => card.definition === 'JC045'
        && card.time_markers > (game(row).regions.flatMap(region => region.cards).find(old => old.id === card.id)?.time_markers || 0)),
    'death observer priest time marker');
    emit('three-color-death-commands', Object.entries({declare: declared, randomDiscard: random,
      influence, ordinaryDiscard: discard, timeMarker: marker}).map(([name, row]) => ({...row, name})));
  },
  'bounded-entry'() {
    const names = {employee: 'bq104-choice', employeeEmpty: 'bq104-empty', hidden: 'bq104-hidden-result',
      borrowed: 'bq104-borrowed-result-0', passers: 'xq48-choice-3-0', passersEmpty: 'xq48-empty',
      entered: 'xq48-result-3-0', replaced: 'xq48-choice-0-0'};
    emit('bounded-entry', {scope: 'Current Native prepared entry-search layouts; opaque outputs unchanged.',
      states: Object.fromEntries(Object.entries(names).map(([key, name]) => [key, raw('bounded-entry', 'native', `${name}.json`)]))});
  },
  'deck-seal'() {
    const names = {xq44Choice: 'xq44-choice-0', xq44Empty: 'xq44-empty-0', xq44Sealed: 'xq44-sealed-result-0',
      jz02Choice: 'jz02-choice-0', jz02Empty: 'jz02-empty-0', jz02Sealed: 'jz02-sealed-result-0',
      jz02Skipped: 'jz02-skipped-false-0', event: 'xq41-event-0'};
    emit('deck-seal', {scope: 'Current Native prepared deck-sealing layouts; opaque outputs unchanged.',
      states: Object.fromEntries(Object.entries(names).map(([key, name]) => [key, raw('deck-seal', 'native', `${name}.json`)]))});
    const rooms = files(directory('deck-seal', 'room'), /^search-.+\.json$/).map(file => trace(read(file), file));
    assert.equal(rooms.length, 5, 'Require all five Native search/receipt scenarios');
    emit('deck-seal-room', rooms);
  },
  'gray-lock'() {
    emit('gray-lock', ['citizen-lock', 'citizen-damage', 'dog-chase-hidden'].map(name =>
      trace(raw('gray-lock', 'room', `${name}.json`), name)));
  },
  jz50() { emit('jz50-room', trace(raw('jz50', 'room', 'native-trace.json'), 'JZ50')); },
  pause() { emit('pause-resume', trace(raw('pause', 'room', 'native-trace.json'), 'pause')); },
  sealing() {
    const restart = trace(raw('sealing', 'restart', 'native-trace.json'), 'sealed restart');
    assert.equal(restart.scope, 'natural-fresh-match-terminal-state-then-real-restart-persist-next-choice',
      'Sealed restart requires the current fresh-match exporter, not a prepared terminal layout');
    emit('sealing-room', trace(raw('sealing', 'room', 'native-trace.json'), 'sealing'));
    emit('sealed-restart', restart);
  },
  xq18() {
    const rows = steps('xq18'), snapshots = checkpoints('xq18');
    const choices = rows.filter(row => accepted(row) && game(row).pending?.choice.kind === 'xq18-time-carrier'
      && selected(row)?.length === 1).map(row => ({...row, beforeViews: beforeViews(row, snapshots, 'XQ18 carrier')}));
    const sameOwners = choices.filter(row => {
      const options = row.beforeViews[row.seat].pendingChoice.options;
      return options.length === 2 && options.every(item => item.label.includes('1号席拥有') && item.label.includes('3个时间标志'));
    });
    assert(sameOwners.length, 'Missing equal-name equal-owner XQ18 carriers');
    const carriers = [0, 1].map(index => {
      const row = pick(sameOwners, row => selected(row)[0] === row.beforeViews[row.seat].pendingChoice.options[index].id,
        `XQ18 selected carrier ${index}`);
      return {...row, selected: selected(row)[0]};
    });
    const room = records('xq18', 'room', /^room-\d+\.json$/);
    const pause = pick(room, row => row.command.action.kind === 'pauseRoom', 'XQ18 pause');
    const resume = pick(room, row => row.command.action.kind === 'resumeRoom' && row.state === pause.expected.state, 'XQ18 resume');
    const choose = pick(room, row => row.state === resume.expected.state && action(row)?.kind === 'choose', 'XQ18 resumed choice');
    emit('xq18', {carriers, pauseResume: [pause, resume, choose]});
  },
};

const scenarioNames = {
  jc037: ['paid-deploy', 'paid-reveal', 'secret-deployment', 'source-control', 'source-return', 'source-hide', 'leader-replace', 'duel-paid-deploy'],
  jc060: ['paid-deploy-team', 'paid-reveal-team', 'stacking-return', 'source-hide', 'source-control', 'recipient-control',
    'current-magic-return', 'magic-target-revalidation', 'holy-water-current-domain', 'spirit-two-domains', 'duel-paid-deploy'],
  jc095: ['paid-deploy-accept', 'paid-deploy-decline', 'secret-reveal', 'response-increase', 'response-zero', 'source-return',
    'source-hide', 'leader-sacrifice-new', 'renown-team', 'win-window-fixed-winner', 'duel-paid-deploy'],
  jc046: ['aura-loss-before-next-seat', 'choose-self-among-others', 'combat-exhausted', 'combat-hidden', 'combat-kill-two',
    'combat-other-region', 'combat-two-sources', 'conceal-no-trigger', 'continuation-container-reference',
    'control-source-sacrifice-restores-next-seat', 'current-control-owner-graves', 'death-callbacks-after-all-seats', 'deploy-no-trigger',
    'duel-seat-zero-first', 'duplicate-frame-reference', 'illegal-and-stale-choices', 'paid-entry-guard-reference', 'paid-seat0',
    'paid-seat1', 'paid-seat2', 'paid-seat3', 'pending-guard-reference', 'pre-eliminated-seat-skip', 'prepared-florence-free-reveal',
    'prepared-granted-glory', 'prepared-granted-renown', 'prepared-leader-paused-continuation', 'prepared-response-source-destroyed',
    'prepared-source-control-change', 'response-source-bounce', 'response-source-hide', 'reveal-extra-target', 'reveal-low-assets',
    'reveal-no-red-loyalty', 'sacrifice-bypasses-protection', 'self-sacrifice-continues', 'two-real-reveals-independent-frames'],
};
for (const [card, names] of Object.entries(scenarioNames)) producers[card] = () => {
  const scenarios = records(card, 'native', /\.json$/);
  assert.deepEqual(scenarios.map(row => row.name).sort(), [...names].sort(), `${card}: missing or extra Native scenarios`);
  for (const row of scenarios) {
    assertCurrentCatalog(row.catalog);
    assert(row.steps.length, `${card}/${row.name}: empty trace`);
    let state = row.initial.state;
    for (const step of row.steps) {
      assert.equal(step.state, state, `${card}/${row.name}: discontinuous Native trace`);
      assert.deepEqual(step.expected.view, step.views[step.seat]);
      state = step.expected.state;
    }
  }
  let invalidManifest;
  if (card === 'jc046') {
    const expectedNames = [
      ...Array.from({length: 17}, (_, n) => `stack-${n}.json`), 'immediate-bypass.json',
      ...Array.from({length: 19}, (_, n) => `pending-${n}.json`), 'forged-optional-declaration.json',
      ...Array.from({length: 8}, (_, n) => `entry-${n}.json`),
      'same-frame-in-pending-and-effect.json', 'same-frame-in-two-effects.json',
      'same-frame-in-stack-and-effect.json', 'two-paused-frames-with-different-ids.json',
      'mandatory-trigger-shell-without-frame.json', 'effect-copy-with-unrelated-sacrifice-pending.json',
      'orphan-accepted-effect-without-forced-pause.json', 'unrelated-effect-before-paused-continuation.json',
      'leader-pause-without-required-recheck.json',
    ].sort();
    const entries = files(directory(card, 'invalid'), /\.json$/);
    assert.deepEqual(entries.map(file => path.relative(directory(card, 'invalid'), file)).sort(), expectedNames,
      'JC046 requires all 55 current Native invalid files, with their exact semantic names.');
    invalidManifest = {invalidRoomStates: 55, files: entries.map(file => {
      read(file); // Verify current identity without repairing the intentional invalid mechanism.
      const bytes = readFileSync(file);
      return {name: path.basename(file), bytes: bytes.length, sha256: sha256(bytes), json: bytes.toString('utf8')};
    })};
  }
  emit(card, {scope: 'Current Native prepared layouts and continuous Room commands; no state adaptation.',
    sourceManifestSha256: currentSourceManifestSha256, catalog, wasmSha256: currentWasmSha256, scenarios});
  if (invalidManifest) emit('jc046-invalid', invalidManifest);
};
const groups = option('groups')?.split(',') || Object.keys(producers);
assert.equal(new Set(groups).size, groups.length, 'Repeated assembler group');
for (const group of groups) {
  assert(Object.hasOwn(producers, group), `Unknown group: ${group}`);
  producers[group]();
}
