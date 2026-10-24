// Pack newly generated current Native selector outputs, never an older save.
// Usage: node tools/fixture-tools/current-ui-selectors.mjs <fresh-native-root>
// Run the documented Native exporters and selector scripts before this step.
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import path from 'node:path';
import {catalog} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog, assertCurrentSourceBindings, currentIdentity,
  writeCurrentFixture} from '../current-native-source.mjs';

assert.equal(currentIdentity.engineVersion, 'rust-v0.2.112-shared-keyword-rewards');
const evidence = path.resolve(process.argv[2]);
const read = file => JSON.parse(readFileSync(file, 'utf8'));
const identities = {rules:currentIdentity.rulesVersion,
  cardPool:currentIdentity.cardPoolVersion,engine:currentIdentity.engineVersion};
const opaque = new Set(['state','before','after','initialState','terminalState','restartState','nextChoiceState','original']);
function audit(value) {
  if (Array.isArray(value)) return value.forEach(audit);
  if (!value || typeof value !== 'object') return;
  if (value.versions) assert.deepEqual(value.versions, identities);
  if (value.catalog) assertCurrentCatalog(value.catalog);
  if (value.sourceBindings) assertCurrentSourceBindings(value.sourceBindings);
  for (const [key, child] of Object.entries(value)) {
    if (opaque.has(key) && typeof child === 'string' && child.startsWith('{')) {
      const state = JSON.parse(child);
      if (state.game) assert.deepEqual(state.game.versions, identities);
      else if (state.versions) assert.deepEqual(state.versions, identities);
    } else audit(child);
  }
}
function pack(name, value) {
  audit(value);
  writeCurrentFixture(`rust-game-wasm/tests/current112/${name}.json.gz`, value);
  console.log(JSON.stringify({name, currentIdentity}));
}
for (const card of ['jc061','jc087','jc094','wm056','wm023','xq25','wm038','jz35'])
  pack(card, read(`web/src/game/${card}Native112.fixture.json`));
pack('jc052', read('web/src/game/jc052Native112Current.fixture.json'));
pack('jc052-worker', read('sites/test/fixtures/jc052-native-v0112.json'));
pack('lifecycle', read('web/src/game/currentLifecycle112.fixture.json'));
pack('fixed-slots', read('web/src/game/fixedSlotsNative112.fixture.json'));
pack('win-flow', read('sites/test/fixtures/win-flow-native-v0112.json'));
pack('bq030-web', read('web/src/game/bq030Native112.fixture.json'));
pack('bq030-sites', read('sites/test/fixtures/bq030-native-v0112.json'));
pack('seven-card', read('web/src/game/sevenCardUIV0112.fixture.json'));
const rows = readdirSync(path.join(evidence,'xq24')).filter(f=>f.endsWith('.json')).sort()
  .map(f=>read(path.join(evidence,'xq24',f)));
assert.equal(rows.length,12);
rows.forEach(row=>assert.deepEqual(row.catalog,catalog));
pack('xq24',{catalog:rows[0].catalog,scenarios:rows});
pack('xq24-invalid',read(path.join(evidence,'xq24-container-invalid.json')));
