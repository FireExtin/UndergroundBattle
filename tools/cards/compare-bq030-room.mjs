// Replay unmodified Native Room exports through the production ESM WASM ABI.
// Usage: node tools/cards/compare-bq030-room.mjs <productionESMbindingPath> <evidenceDir>
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const argumentsRemaining = process.argv.slice(2);
const [bindingsArgument, evidenceArgument] = argumentsRemaining;
const evidenceDirectory = evidenceArgument ? path.resolve(evidenceArgument) : null;
const reportPath = evidenceDirectory && path.join(evidenceDirectory, 'wasm-parity.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const canonical = value => {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
  }
  return value;
};
const valueHash = value => hash(JSON.stringify(canonical(value)));
const result = {
  passed: false,
  scope: 'BQ030 Native Room command parity; browser playtesting remains with parent.',
  commands: 0,
  acceptedCommands: 0,
  rejectedCommands: 0,
  fullTransitionComparisons: 0,
  opaqueStateByteComparisons: 0,
  views: 0,
  invalidStates: 0,
  invalidViewRejections: 0,
  invalidCommandRejections: 0,
  catalogComparisons: 0,
  bq030AcceptedDeclarations: 0,
  bq030AttachmentOrderAcceptedDeclarations: 0,
  bq030MultiObserverAcceptedDeclarations: 0,
  bq030AcceptedFrozenSourcesOffBoard: 0,
  bq030ReattachmentObservedTransitions: 0,
  bq030LethalBQ030ObservedTransitions: 0,
  bq030CoverageSamples: [],
  sourceInputs: [],
  roomChecksums: [],
};
let currentCheck = 'arguments';
const save = () => writeFileSync(reportPath, JSON.stringify(result, null, 2) + '\n');
const read = relative => {
  const raw = readFileSync(path.join(evidenceDirectory, relative));
  result.sourceInputs.push({ path: relative, bytes: raw.length, sha256: hash(raw) });
  return JSON.parse(raw.toString('utf8'));
};
const jsonFiles = relative => {
  const files = readdirSync(path.join(evidenceDirectory, relative), { withFileTypes: true });
  assert.ok(files.length > 0, `${relative}: expected nonempty Native evidence`);
  for (const file of files) {
    assert.ok(file.isFile() && file.name.endsWith('.json'), `${relative}/${file.name}: expected a JSON file`);
  }
  return files.map(file => file.name).sort();
};
const observationInOps = ops => {
  const observation = ops?.[0]?.BQ030AttachmentDraw?.observation;
  return observation && typeof observation === 'object' && !Array.isArray(observation)
    ? observation : null;
};
const observationInDeclaration = declaration => declaration?.source?.card?.definition === 'BQ030'
  && declaration.ability?.event === 'AttachmentCommittedObserved'
  ? observationInOps(declaration.ability.ops) : null;
const observationInFrame = frame => frame?.source?.card?.definition === 'BQ030'
  && frame.ability_key === 'attachment-observer-draw-one'
  ? observationInOps(frame.steps?.map(step => step.op)) : null;
const observationsInGame = game => [
  observationInDeclaration(game.pending?.resolution?.Declare?.declaration),
  observationInFrame(game.pending?.resolution?.Frame?.frame),
  ...(game.stack ?? []).map(item => observationInFrame(item.frame)),
  ...(game.effects ?? []).map(effect => observationInDeclaration(effect.Declare?.declaration)
    ?? observationInFrame(effect.Frame?.frame)),
].filter(Boolean);
const boardHas = (game, id) => (game.regions ?? []).some(region =>
  (region.cards ?? []).some(card => card.id === id));
const recordBQ030Coverage = (sample, transition, relative) => {
  // These decoded copies identify coverage only. The ABI still receives the
  // exact original Native state string; no decoded state is written back.
  const before = JSON.parse(sample.state).game;
  const after = JSON.parse(transition.state).game;
  const coverage = { path: relative };
  const declarationChoice = before.pending?.resolution?.Declare;
  const event = observationInDeclaration(declarationChoice?.declaration);
  const action = sample.command.action?.kind === 'game' ? sample.command.action.action : null;
  if (transition.outcome === 'accepted' && !transition.errorCode && event
      && action?.kind === 'choose' && action.choiceId === before.pending.choice.id
      && Array.isArray(action.selected) && action.selected.length > 0) {
    const selectedSource = declarationChoice.stage === 'Accept'
      ? event.remaining[0] : action.selected[0];
    const previousFrames = new Set((before.stack ?? []).map(item => item.frame?.frame_id));
    const newFrame = (after.stack ?? []).map(item => item.frame).find(frame => {
      const frameEvent = observationInFrame(frame);
      return frameEvent && !previousFrames.has(frame.frame_id)
        && frame.source.card.id === selectedSource && frameEvent.event_id === event.event_id;
    });
    assert.ok(newFrame, `${relative}: a real BQ030 acceptance must add its observed draw to the stack`);
    result.bq030AcceptedDeclarations++;
    coverage.acceptedEventId = event.event_id;
    coverage.acceptedSourceId = selectedSource;
    coverage.newFrameId = newFrame.frame_id;
    if (declarationChoice.stage === 'AttachmentOrder') {
      result.bq030AttachmentOrderAcceptedDeclarations++;
      coverage.attachmentOrder = true;
    }
    if (event.observers.length > 1) {
      result.bq030MultiObserverAcceptedDeclarations++;
      coverage.multipleObservers = true;
    }
    if (!boardHas(before, selectedSource)) {
      result.bq030AcceptedFrozenSourcesOffBoard++;
      coverage.frozenSourceOffBoard = true;
    }
  }
  const previousEvents = new Set(observationsInGame(before).map(observed => observed.event_id));
  for (const observed of observationsInGame(after)) {
    if (previousEvents.has(observed.event_id)) continue;
    // Compare actual before/after identities, without simulating attachment rules.
    const previousAttachment = (before.attachments ?? []).find(attachment =>
      attachment.card.id === observed.attachment.id);
    if (previousAttachment && previousAttachment.hostId !== observed.host.card.id) {
      result.bq030ReattachmentObservedTransitions++;
      coverage.reattachmentEventId = observed.event_id;
    }
    if (observed.attachment.definition === 'JC089' && observed.host.card.definition === 'BQ030'
        && !boardHas(after, observed.host.card.id)) {
      result.bq030LethalBQ030ObservedTransitions++;
      coverage.lethalFrozenEventId = observed.event_id;
    }
    previousEvents.add(observed.event_id);
  }
  if (Object.keys(coverage).length > 1) result.bq030CoverageSamples.push(coverage);
};

try {
  assert.equal(argumentsRemaining.length, 2,
    'Usage: node tools/cards/compare-bq030-room.mjs <productionESMbindingPath> <evidenceDir>');
  assert.ok(bindingsArgument && evidenceArgument, 'Both argument paths must be nonempty');
  // Replace any previous successful report before loading or comparing evidence.
  currentCheck = 'initialize failure-closed report';
  save();

  currentCheck = 'production ESM bindings and WASM module';
  const bindingsPath = path.resolve(bindingsArgument);
  const bindingBytes = readFileSync(bindingsPath);
  const wasmPath = path.join(path.dirname(bindingsPath), 'hegemony_wasm_bg.wasm');
  const wasmBytes = readFileSync(wasmPath);
  result.bindingsPath = bindingsPath;
  result.bindingsBytes = bindingBytes.length;
  result.bindingsSha256 = hash(bindingBytes);
  result.wasmPath = wasmPath;
  result.wasmBytes = wasmBytes.length;
  result.wasmSha256 = hash(wasmBytes);
  const productionABI = await import(pathToFileURL(bindingsPath).href);
  for (const name of ['initSync', 'catalog', 'applyRoom', 'view']) {
    assert.equal(typeof productionABI[name], 'function', `Production ABI export missing: ${name}`);
  }
  productionABI.initSync({ module: wasmBytes });

  currentCheck = 'native-catalog.json: complete catalog';
  const nativeCatalog = read('native-catalog.json');
  const wasmCatalog = JSON.parse(productionABI.catalog());
  assert.deepEqual(wasmCatalog, nativeCatalog, currentCheck);
  assert.ok(wasmCatalog.cards.some(card => card.id === 'BQ030' && card.supported === true),
    'Parity evidence requires admitted BQ030 in the actual production catalog');
  assert.equal(typeof wasmCatalog.engineVersion, 'string', 'Catalog engineVersion must be a string');
  result.engine = wasmCatalog.engineVersion;
  result.catalogSha256 = valueHash(wasmCatalog);
  result.catalogComparisons++;

  currentCheck = 'room: Native evidence directory';
  for (const filename of jsonFiles('room')) {
    const relative = `room/${filename}`;
    currentCheck = `${relative}: Native sample format`;
    const sample = read(relative);
    assert.equal(typeof sample.state, 'string', `${relative}: opaque input state must be a string`);
    assert.ok(Number.isInteger(sample.seat) && sample.seat >= 0 && sample.seat < 4,
      `${relative}: authenticated seat must be 0..3`);
    assert.ok(sample.command && typeof sample.command === 'object' && !Array.isArray(sample.command),
      `${relative}: complete Room command missing`);
    assert.ok(sample.expected && typeof sample.expected === 'object' && !Array.isArray(sample.expected),
      `${relative}: expected complete transition missing`);
    assert.equal(typeof sample.expected.state, 'string', `${relative}: expected opaque state missing`);
    assert.ok(Array.isArray(sample.views) && sample.views.length === 4,
      `${relative}: expected exactly four seat views`);

    // Input state and command come directly from the original Native export.
    currentCheck = `${relative}: applyRoom complete transition`;
    const transition = JSON.parse(productionABI.applyRoom(
      sample.state, sample.seat, JSON.stringify(sample.command), '0'));
    assert.deepEqual(transition, sample.expected, currentCheck);
    result.fullTransitionComparisons++;

    currentCheck = `${relative}: opaque state bytes`;
    assert.equal(transition.state, sample.expected.state, currentCheck);
    assert.deepEqual(Buffer.from(transition.state, 'utf8'), Buffer.from(sample.expected.state, 'utf8'), currentCheck);
    result.opaqueStateByteComparisons++;
    currentCheck = `${relative}: real BQ030 input/output coverage`;
    recordBQ030Coverage(sample, transition, relative);
    const viewSha256 = [];
    for (let seat = 0; seat < 4; seat++) {
      currentCheck = `${relative}: seat ${seat} view`;
      const view = JSON.parse(productionABI.view(transition.state, seat));
      assert.deepEqual(view, sample.views[seat], currentCheck);
      viewSha256.push(valueHash(view));
      result.views++;
    }
    result.roomChecksums.push({
      path: relative,
      commandId: sample.command.commandId,
      inputStateBytes: Buffer.byteLength(sample.state, 'utf8'),
      inputStateSha256: hash(sample.state),
      outputStateBytes: Buffer.byteLength(transition.state, 'utf8'),
      outputStateSha256: hash(transition.state),
      transitionSha256: valueHash(transition),
      viewSha256,
    });
    result.commands++;
    if (sample.expected.errorCode) result.rejectedCommands++;
    else result.acceptedCommands++;
  }
  currentCheck = 'room: accepted and rejected command coverage';
  assert.ok(result.acceptedCommands > 0, 'Expected at least one accepted Native Room command');
  assert.ok(result.rejectedCommands > 0, 'Expected the BQ030 rejected-command Native Room regressions');
  currentCheck = 'room: real BQ030 acceptance and attachment-event coverage';
  assert.ok(result.bq030AcceptedDeclarations > 0,
    'Expected a real BQ030 observed declaration accepted into a new draw stack frame');
  assert.ok(result.bq030AttachmentOrderAcceptedDeclarations > 0
    && result.bq030MultiObserverAcceptedDeclarations > 0,
  'Expected actual multi-observer AttachmentOrder acceptance');
  assert.ok(result.bq030ReattachmentObservedTransitions > 0,
    'Expected actual existing-attachment host change producing a new BQ030 observation');
  assert.ok(result.bq030LethalBQ030ObservedTransitions > 0
    && result.bq030AcceptedFrozenSourcesOffBoard > 0,
  'Expected lethal JC089 attachment and acceptance of a frozen BQ030 source already off board');

  currentCheck = 'invalid: Native evidence directory';
  for (const filename of jsonFiles('invalid')) {
    const relative = `invalid/${filename}`;
    currentCheck = `${relative}: Native invalid state format`;
    const sample = read(relative);
    assert.equal(typeof sample.state, 'string', `${relative}: opaque invalid state must be a string`);
    currentCheck = `${relative}: view must throw`;
    assert.throws(() => productionABI.view(sample.state, 0), currentCheck);
    result.invalidViewRejections++;
    currentCheck = `${relative}: applyRoom must throw`;
    assert.throws(() => productionABI.applyRoom(sample.state, 0, JSON.stringify({
      commandId: `bq030-invalid-${filename}`,
      expectedVersion: 0,
      action: { kind: 'game', action: { kind: 'pass' } },
    }), '0'), currentCheck);
    result.invalidCommandRejections++;
    result.invalidStates++;
  }

  result.sourceInputCount = result.sourceInputs.length;
  result.sourceInputChecksumsSha256 = valueHash(result.sourceInputs);
  result.stateInputsUsedVerbatim = true;
  result.passed = true;
  currentCheck = 'write successful parity report';
  save();
  console.log(JSON.stringify({ ...result,
    sourceInputs: result.sourceInputCount, roomChecksums: result.roomChecksums.length,
    bq030CoverageSamples: result.bq030CoverageSamples.length, reportPath }));
} catch (error) {
  result.passed = false;
  result.failure = { check: currentCheck, message: String(error?.message ?? error) };
  if (reportPath) {
    try { save(); }
    catch (writeError) { result.reportWriteError = String(writeError?.message ?? writeError); }
  }
  console.error(JSON.stringify({ ...result,
    sourceInputs: result.sourceInputs.length, roomChecksums: result.roomChecksums.length,
    bq030CoverageSamples: result.bq030CoverageSamples.length, reportPath }));
  process.exitCode = 1;
}
