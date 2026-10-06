// Compare a single continuous RoomEnvelope chain including real composition.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import * as kernel from '../pkg/hegemony_wasm.js';
kernel.initSync({module:readFileSync(new URL('../pkg/hegemony_wasm_bg.wasm',import.meta.url))});
assert.equal(JSON.parse(kernel.catalog()).engineVersion,'rust-v0.2.35-jz31-death-influence-candidate');
const fixture=JSON.parse(readFileSync(process.argv[2],'utf8'));
let state=fixture.initialState, projections=0;
const kinds={};
for(const step of fixture.steps) {
 assert.equal(state,step.state); // full chain continuity, without reconstructed windows.
 const result=JSON.parse(kernel.applyRoom(state,step.seat,JSON.stringify(step.command),step.serverNowMs));
 assert.deepEqual(result,step.expected);
 assert.equal(result.errorCode,undefined);
 state=result.state;
 kinds[step.command.action.kind]=(kinds[step.command.action.kind]||0)+1;
 for(let seat=0;seat<4;seat++) {assert.deepEqual(JSON.parse(kernel.view(state,seat)),step.views[seat]);projections++;}
}
assert.equal(state,fixture.finalState);
assert.equal(kinds.beginResponse,1); assert.equal(kinds.submitResponse,1);
assert.ok(kinds.passResponse>=4 && kinds.game>=3);
assert.deepEqual(JSON.parse(kernel.view(state,0)).regions[2].influence,[1,0]);
console.log(JSON.stringify({commands:fixture.steps.length,commandKinds:kinds,projections,finalInfluence:fixture.finalInfluence}));
