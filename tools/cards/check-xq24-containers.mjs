// Actual Native-produced opaque inputs; production WASM restore entrypoints.
// Never parse/reserialize a state: the regression producer uses u64::MAX.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const [bindingsPath = 'rust-game-wasm/pkg/hegemony_wasm.js', output] = process.argv.slice(2);
const bindings = pathToFileURL(resolve(bindingsPath));
const abi = await import(bindings.href);
const wasm = readFileSync(new URL('./hegemony_wasm_bg.wasm', bindings));
abi.initSync({ module: wasm });
const root = new URL('../../', import.meta.url);
const bytes = readFileSync(new URL('sites/test/fixtures/xq24-container-invalid-v064.json', root));
const native = JSON.parse(bytes);
assert.equal(native.producer, 'xq24_entry_frame_container_restore_rejects_response_bypass');
assert.equal(native.cases.length, 4);
const report = { passed:false, rejectedStates:0, restoreEntrypoints:0,
  normalScenarios:0, normalCommands:0, seatViews:0,
  fixtureSha256:createHash('sha256').update(bytes).digest('hex'),
  wasmBytes:wasm.length, wasmSha256:createHash('sha256').update(wasm).digest('hex') };
for (const c of native.cases) {
  assert(c.state.includes('"seed":18446744073709551615'));
  const original = JSON.parse(abi.view(c.original, 0));
  assert(original.responseWindow);
  const command = JSON.stringify({ commandId:'container-restore-regression',
    expectedVersion:original.version, action:{ kind:'game', action:{ kind:'pass' } } });
  const validPass = JSON.parse(abi.applyRoom(c.original, 0, command, '0'));
  assert.equal(validPass.errorCode, 'invalid_action');
  assert.equal(validPass.state, c.original);
  assert.equal(JSON.parse(abi.pollRoom(c.original, 0, '0')).state, c.original);
  for (const restore of [() => abi.view(c.state, 0),
    () => abi.applyRoom(c.state, 0, command, '0'), () => abi.pollRoom(c.state, 0, '0')]) {
    assert.throws(restore, error => String(error).includes('XQ24进场响应帧不能保存于直接效果队列'));
    report.restoreEntrypoints++;
  }
  report.rejectedStates++;
}
// Previously exported real Native Room commands retain exactly the same result.
// Includes optional acceptance/decline, source movement/removal and lost regions.
const legal = JSON.parse(readFileSync(new URL('sites/test/fixtures/xq24-native-v064.json', root)));
assert.deepEqual(JSON.parse(abi.catalog()), legal.catalog);
for (const scenario of legal.scenarios) {
  let previous;
  for (const s of scenario.steps) {
    if (previous !== undefined) assert.equal(s.state, previous);
    const result = JSON.parse(abi.applyRoom(s.state, s.seat, JSON.stringify(s.command), s.serverNowMs));
    assert.deepEqual(result, s.expected);
    assert.equal(result.state, s.expected.state);
    for (let seat=0; seat<s.views.length; seat++) {
      assert.deepEqual(JSON.parse(abi.view(result.state, seat)), s.views[seat]);
      report.seatViews++;
    }
    previous=result.state; report.normalCommands++;
  }
  report.normalScenarios++;
}
assert.equal(report.normalScenarios, 12);
assert.equal(report.normalCommands, 117);
report.passed = true;
if (output) writeFileSync(output, JSON.stringify(report, null, 2)+'\n');
console.log(JSON.stringify(report));
