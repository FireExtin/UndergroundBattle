// The reviewed P2 assertions use a fresh ABI93 paid frame and owner trajectory.
// Historical Native88 bytes remain in their original archives, untouched.
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {loadFixture,sha256} from './xq23-oracle.mjs';
import {loadOwnerFixture,validateOwnerNative,compareOwner,packOwnerFixture} from './xq23-current-owner-oracle.mjs';
export {loadOwnerFixture as loadP2Fixture,validateOwnerNative as validateP2Native};

export function reviewSummaryCases(){
 const current=loadFixture(),scenario=current.scenarios.find(s=>s.name==='actor-0');
 assert(scenario,'Current Native actor-0 trace is required');
 const paid=scenario.steps.find(r=>!r.expected.errorCode&&r.command.action.action?.abilityId==='xq23-return-asset-become-asset');
 assert(paid,'Current Native paid activation is required');
 const control=paid.expected.state,original=JSON.parse(control),stack=original.game.stack[0];
 assert.equal(stack.frame.ability_key,'xq23-return-asset-become-asset');
 const targetId=stack.target,otherAssetId=original.game.players[0].assets.find(c=>c.id!==targetId)?.id;
 assert(targetId&&otherAssetId&&targetId!==otherAssetId);
 assert.equal(stack.frame.targets[0].id,targetId);
 assert.equal(stack.frame.targets[0].public.instanceId,targetId);
 assert.equal(stack.frame.targets[0].public.kind,'asset');
 const invalid=[
  ['target-summary-other-asset-uid','instanceId',null],
  ['target-summary-empty-uid','instanceId',''],
  ['target-summary-hidden-kind','kind','hidden'],
  ['target-summary-character-kind','kind','character'],
 ].map(([name,field,value])=>{
  const native=current.invalid.find(r=>r.name===name);assert(native?.roomError,name);
  // Read the compiler-exported malformed Room directly. Do not derive or
  // reserialize a substitute state from the legal DOM control above.
  const room=JSON.parse(native.state),entry=room.game.stack[0],publicTarget=entry.frame.targets[0].public;
  assert.equal(room.game.room_id,native.roomId);assert.equal(room.game.invite_code,native.inviteCode);assert.equal(room.revision,native.revision);
  assert.equal(entry.frame.ability_key,'xq23-return-asset-become-asset');assert.equal(entry.frame.targets[0].id,entry.target);
  if(value===null){
   assert.notEqual(publicTarget.instanceId,entry.target);
   assert(room.game.players[entry.frame.actor].assets.some(c=>c.id===publicTarget.instanceId));
  }else assert.equal(publicTarget[field],value);
  return{...native,changedPath:'/game/stack/0/frame/targets/0/public/'+field,nativeStateSha256:sha256(native.state)};
 });
 return{control,invalid,targetId,otherAssetId,views:paid.views,revision:original.revision,controlSha256:sha256(control),nativeRawSha256:current.raw.sha256};
}

export function compareP2(f,abi,wasm){
 const result=compareOwner(f,abi,wasm),boundary=reviewSummaryCases();
 for(let seat=0;seat<4;seat++){
  const view=JSON.parse(abi.view(boundary.control,seat));
  assert.deepEqual(view,boundary.views[seat]);assert.equal(view.stack[0].targetSummaries[0].instanceId,boundary.targetId);
 }
 for(const bad of boundary.invalid){
  const error=e=>{assert.equal(String(e),bad.roomError);return true;};
  for(let seat=0;seat<4;seat++)assert.throws(()=>abi.view(bad.state,seat),error);
  assert.throws(()=>abi.pollRoom(bad.state,0,'0'),error);
  assert.throws(()=>abi.applyRoom(bad.state,0,JSON.stringify({commandId:'p2-'+bad.name,expectedVersion:bad.revision,action:{kind:'pauseRoom'}}),'0'),error);
 }
 return{...result,currentPaidStateSha256:boundary.controlSha256,currentCoreNativeRawSha256:boundary.nativeRawSha256,currentNativeInvalidStateSha256:Object.fromEntries(boundary.invalid.map(bad=>[bad.name,bad.nativeStateSha256])),currentPaidStateRejectedCases:4,originalHistoricalInputsUntouched:true};
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const [nativeFile,wasmFile,destination]=process.argv.slice(2);
 await packOwnerFixture(nativeFile,wasmFile,destination,compareP2);
}
