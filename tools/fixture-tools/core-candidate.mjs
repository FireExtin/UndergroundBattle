// Compare fresh Native transitions before writing a source-bound candidate fixture.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {kernel} from '../../rust-game-wasm/tests/current-kernel.mjs';
import {assertCurrentCatalog,assertCurrentSourceBindings,writeCurrentFixture,sha256} from '../current-native-source.mjs';
assert(process.argv[2],'Provide the fresh BQ046/BQ055 Native Room JSON.');
const bytes=readFileSync(process.argv[2]),raw=JSON.parse(bytes);
assertCurrentSourceBindings(raw.sourceBindings);assertCurrentCatalog(raw.catalog);
assert.deepEqual(raw.cases.map(c=>c.name),['bq046-paid-bottom','bq046-prepared-core-false','bq046-prepared-core-true','bq055-real-contest','bq046-deploy-death-return-false','bq046-deploy-death-return-true','bq048-command-core-false','bq048-command-core-true','bq056-core-entry-lock-repress']);
let commands=0,projections=0;
for(const scenario of raw.cases){let state=scenario.initial.state;
 for(const step of scenario.steps){
  assert.equal(step.state,state);const actual=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));assert.deepEqual(actual,step.expected);state=actual.state;commands++;
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(kernel.view(state,seat)),step.views[seat]);projections++;}
  assert.deepEqual(JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs)),step.duplicate);
 }
}
writeCurrentFixture('rust-game-wasm/tests/current113/core-candidate.json.gz',raw);
console.log(JSON.stringify({nativeRawSha256:sha256(bytes),scenarios:raw.cases.length,commands,projections,duplicates:commands,preparedLayout:true}));
