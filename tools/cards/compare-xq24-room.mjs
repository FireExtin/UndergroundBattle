// Replay exact, synthetic Native Room inputs through the production WASM ABI.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
const [bindingPath, evidencePath] = process.argv.slice(2);
assert.ok(bindingPath && evidencePath, 'Usage: compare-xq24-room.mjs <ESM bindings> <evidence root>');
const evidence = resolve(evidencePath), binding = pathToFileURL(resolve(bindingPath));
const abi = await import(binding);
abi.initSync({ module:readFileSync(new URL('./hegemony_wasm_bg.wasm', binding)) });
const report = { passed:false, engine:JSON.parse(abi.catalog()).engineVersion, commands:0,
  rejectedCommands:0, projections:0, checkpoints:0, invalidStates:0, continuousScenarios:0, sourceInputs:[] };
const files = folder => readdirSync(join(evidence, folder)).filter(n => n.endsWith('.json')).sort();
const read = (folder, name) => {
  const bytes = readFileSync(join(evidence, folder, name));
  report.sourceInputs.push({ path:`${folder}/${name}`, bytes:bytes.length, sha256:createHash('sha256').update(bytes).digest('hex') });
  return JSON.parse(bytes);
};
function step(s) {
  const result = JSON.parse(abi.applyRoom(s.state, s.seat, JSON.stringify(s.command), s.serverNowMs ?? '0'));
  assert.deepEqual(result, s.expected);
  assert.equal(result.state, s.expected.state);
  for (let seat=0; seat<s.views.length; seat++) {
    assert.deepEqual(JSON.parse(abi.view(result.state, seat)), s.views[seat]); report.projections++;
  }
  report.commands++; if (result.errorCode) report.rejectedCommands++;
}
try {
  for (const file of files('native')) {
    const record = read('native', file);
    if (record.command) step(record);
    else {
      assert.equal(typeof record.state, 'string');
      for (let seat=0; seat<record.views.length; seat++) {
        assert.deepEqual(JSON.parse(abi.view(record.state, seat)), record.views[seat]); report.projections++;
      }
      report.checkpoints++;
    }
  }
  for (const file of files('room')) {
    const scenario = read('room', file);
    assert.deepEqual(JSON.parse(abi.catalog()), scenario.catalog);
    let previous;
    for (const s of scenario.steps) {
      if (previous !== undefined) assert.equal(s.state, previous, `${scenario.name}: discontinuous original Native input`);
      step(s); previous = s.expected.state;
    }
    report.continuousScenarios++;
  }
  for (const file of files('invalid')) {
    const {state} = read('invalid', file);
    assert.throws(() => abi.view(state, 0));
    assert.throws(() => abi.applyRoom(state, 0, JSON.stringify({ commandId:'reject-invalid-xq24', expectedVersion:0,
      action:{kind:'game', action:{kind:'pass'}} }), '0'));
    report.invalidStates++;
  }
  assert.equal(report.continuousScenarios, 12);
  assert.ok(report.invalidStates >= 17);
  report.passed = true;
} finally {
  writeFileSync(join(evidence, 'wasm-parity.json'), JSON.stringify(report, null, 2)+'\n');
}
console.log(JSON.stringify({ ...report, sourceInputs:report.sourceInputs.length }));
