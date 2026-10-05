// Local real MSJC09 + normal HTTP commands. No initial layout/DB state injection,
// no browser or remote natural-play claim. Persistence reopen is actual workerd/D1.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { Miniflare, convertV4MiniflareOptions } from 'miniflare';
import { RoomStore } from '../src/store.mjs';

test('real MSJC09 permits first/rear paid activation and restores four-seat frames, receipts, privacy and reset', async t => {
  const dist = resolve('dist/server'), persist = mkdtempSync(join(tmpdir(), 'hegemony-msjc09-'));
  const options = convertV4MiniflareOptions({ name:'printed-msjc09-local', modulesRoot:dist, resourcePersistencePath:persist,
    modules:[{type:'ESModule',path:join(dist,'index.js')},...readdirSync(dist).filter(n=>n.endsWith('.wasm')).map(n=>({type:'CompiledWasm',path:join(dist,n)}))],
    compatibilityDate:'2026-10-02',d1Databases:{DB:'printed-msjc09-db'} });
  let mf = new Miniflare(options); t.after(()=>mf.dispose()); let db = await mf.getD1Database('DB');
  for (const name of readdirSync('drizzle').filter(n=>n.endsWith('.sql'))) for (const sql of readFileSync('drizzle/'+name,'utf8').split('--> statement-breakpoint').filter(s=>s.trim())) await db.prepare(sql).run();
  let calls=0, commands=0, reopens=0, restoredBeforeExpiry=0;
  const api = async (path,body,session) => {
    calls++; const r = await mf.dispatchFetch('http://localhost'+path,{method:body?'POST':'GET',headers:{...(body?{'Content-Type':'application/json'}:{}),...(session?{Authorization:'Bearer '+session.token}:{})},...(body?{body:JSON.stringify(body)}:{})});
    return {status:r.status,body:r.status===204?null:await r.json()};
  };
  const cat = (await api('/api/catalog')).body;
  assert.equal(cat.engineVersion,'rust-v0.2.25-resource-policy-candidate'); assert.equal(cat.cards.length,87);
  assert.deepEqual(cat.societies.map(s=>s.id),['MSJC09','MSJC01']); assert(cat.deckBuildRules.societySupported);
  assert.equal(cat.societies[0].unique,true);
  assert.deepEqual([cat.societies[0].name,cat.societies[0].subtitle,cat.societies[0].color,cat.societies[0].startingHand,cat.societies[0].printedCost],['秘社','未知的聚会','中立',6,null]);
  const draft = societyId => ({id:'printed-msjc09-deck',name:'真实MSJC09验证',description:'',societyId,cards:[{cardId:'JC125',count:50}],rulesVersion:cat.rulesVersion,cardPoolVersion:cat.cardPoolVersion,engineVersion:cat.engineVersion,updatedAt:''});
  for (const d of [draft('MSJC01'),draft('MSJC16'),draft('FIXTURE_SOCIETY_SIX'),{...draft('MSJC09'),cards:[{cardId:'JC125',count:49}]}]) assert.equal((await api('/api/rooms',{name:'非法',mode:'teams',deckDraft:d,requestId:crypto.randomUUID()})).status,400);
  const create = {name:'A',mode:'teams',deckDraft:draft('MSJC09'),requestId:crypto.randomUUID()};
  const initial = await api('/api/rooms',create); assert.equal(initial.status,200); assert.deepEqual(await api('/api/rooms',create),initial);
  assert.equal((await api('/api/rooms',{...create,deckDraft:draft(null)})).status,409);
  const sessions = [initial.body];
  for (const [name,society] of [['B',null],['C','MSJC09'],['D',null]]) { const r = await api('/api/rooms/join',{name,inviteCode:initial.body.inviteCode,deckDraft:draft(society),requestId:crypto.randomUUID()}); assert.equal(r.status,200);sessions.push(r.body); }
  const path = (seat,kind)=>`/api/rooms/${sessions[seat].roomId}/${kind}`;
  const views = async ()=>Promise.all(sessions.map(async(s,seat)=>{const r=await api(path(seat,'state'),null,s);assert.equal(r.status,200);return r.body;}));
  const command = async (seat,action,expectedVersion) => {
    const c={commandId:crypto.randomUUID(),expectedVersion,action};const r=await api(path(seat,'commands'),c,sessions[seat]);
    assert.equal(r.status,200,JSON.stringify(r.body));commands++;return {command:c,result:r};
  };
  const game = (seat,action,version)=>command(seat,{kind:'game',action},version);
  let all=await views();
  for (let seat=0;seat<4;seat++) { assert(all[seat].societyZones.every(z=>z.card===null));assert.equal(all[seat].yourDeck.societyId,seat===0||seat===2?'MSJC09':null); }
  for (let seat=0;seat<4;seat++) { all=await views();await game(seat,{kind:'ready'},all[seat].version); }
  all=await views();await game(0,{kind:'start'},all[0].version);all=await views();
  assert.deepEqual(all[0].players.map(p=>p.handCount),[6,6,6,6]);assert.deepEqual(all[0].players.map(p=>p.deckCount),[44,44,44,44]);
  const owners=[0,2], sources=owners.map(seat=>all[0].societyZones[seat].card.instanceId);
  const paid=[null,null], resolved=[false,false], sawInitiative=new Set();
  const checkPrivacy = list => {
    for (let seat=0;seat<4;seat++) {
      const v=list[seat];assert.equal(v.societyZones.length,4);assert.equal(v.societyZones[1].card,null);assert.equal(v.societyZones[3].card,null);
      assert(v.societyZones.filter(z=>z.card).every(z=>z.card.cardId==='MSJC09' && !z.card.faceDown && z.card.region===undefined && z.card.cost===undefined));
      assert(v.hand.every(c=>c.owner===`p${seat}`));
      assert.deepEqual(v.societyZones,list[0].societyZones);
      const otherHands=list.filter((_,i)=>i!==seat).flatMap(v=>v.hand.map(c=>c.instanceId));
      assert(!otherHands.some(id=>JSON.stringify(v.pendingChoice||{}).includes(id)));
      const pendingSeats=list.filter(v=>v.pendingChoice);assert(pendingSeats.length<=1);
      for (const p of v.players) assert.equal(p.societyId,undefined);
    }
  };
  for (let n=0;n<2000;n++) {
    all=await views();checkPrivacy(all);
    for (let i=0;i<2;i++) if (paid[i] && !resolved[i] && !all[owners[i]].responseWindow) {
      const v=all[owners[i]];assert.equal(v.hand.length,paid[i].before+paid[i].draw);assert.equal(v.assets.filter(c=>c.controller===`p${owners[i]}` && c.exhausted).length,3);
      assert(v.societyZones[owners[i]].card.exhausted);resolved[i]=true;
    }
    if (resolved.every(Boolean) && all[0].turn>=4) break;
    const choosing=all.findIndex(v=>v.pendingChoice);
    if (choosing>=0) { const v=all[choosing],c=v.pendingChoice;await game(choosing,{kind:'choose',choiceId:c.id,selected:c.allowDecline?[]:c.options.slice(0,c.min??1).map(o=>o.id)},v.version);continue; }
    let acted=false;
    for (let i=0;i<2;i++) {
      if (paid[i]) continue;const seat=owners[i],v=all[seat],source=sources[i];
      const activate=v.legalActions.find(a=>a.kind==='activate' && a.cardId===source);
      if (activate) {
        assert.equal(activate.sourceZoneId,`society:p${seat}`);assert.equal(activate.region,undefined);
        const a={kind:'activate',cardId:source,abilityId:activate.abilityId};
        // Standard rights of a teammate do not authorize borrowing this source.
        for (const wrong of [seat^1,...owners.filter(s=>s!==seat)]) {
          assert(!all[wrong].legalActions.some(a=>a.cardId===source));
          const denied=await api(path(wrong,'commands'),{commandId:crypto.randomUUID(),expectedVersion:v.version,action:{kind:'game',action:a}},sessions[wrong]);assert.equal(denied.status,400);
        }
        const draw=Number(v.players.find(p=>p.id===`p${seat}`).team===v.firstTeam);sawInitiative.add(draw);
        const before=v.hand.length;const sent=await game(seat,a,v.version);
        assert.equal(sent.result.body.hand.length,before);assert.equal(sent.result.body.assets.filter(c=>c.controller===`p${seat}` && c.exhausted).length,3);assert(sent.result.body.societyZones[seat].card.exhausted);
        assert(sent.result.body.responseWindow);paid[i]={...sent,before,draw};
        const opaque=(await new RoomStore(db).room(sessions[0].roomId)).state;
        await mf.dispose();mf=new Miniflare(options);db=await mf.getD1Database('DB');reopens++;
        assert.equal((await new RoomStore(db).room(sessions[0].roomId)).state,opaque);
        assert.deepEqual(await api(path(seat,'commands'),sent.command,sessions[seat]),sent.result);
        assert.equal((await api(path(seat,'commands'),{...sent.command,action:{kind:'game',action:{...a,abilityId:'other'}}},sessions[seat])).status,409);
        all=await views();checkPrivacy(all);
        if (all[seat].responseWindow) {restoredBeforeExpiry++;assert.equal(all[seat].responseWindow.id,sent.result.body.responseWindow.id);assert.deepEqual(all[seat].responseWindow.members,sent.result.body.responseWindow.members);assert.equal(all[seat].hand.length,before);}
        else assert.equal(all[seat].hand.length,before+draw); // Actual elapsed expiry may resolve once.
        acted=true;break;
      }
      const asset=v.legalActions.find(a=>a.kind==='asset');
      if (asset) {await game(seat,{kind:'asset',cardId:asset.cardId},v.version);acted=true;break;}
    }
    if (acted) continue;
    let passed=false;
    for (let seat=0;seat<4;seat++) {const v=all[seat];if (v.legalActions.some(a=>a.kind==='pass')) {await command(seat,v.responseWindow?{kind:'passResponse',windowId:v.responseWindow.id}:{kind:'game',action:{kind:'pass'}},v.version);passed=true;break;}}
    assert(passed,'normal pass advances bounded neutral deck');
  }
  assert(resolved.every(Boolean) && all[0].turn>=4);assert.deepEqual([...sawInitiative].sort(),[0,1]);assert.equal(reopens,2);
  for (let i=0;i<2;i++) {
    const seat=owners[i],s=all[seat].societyZones[seat].card;assert.equal(s.instanceId,sources[i]);assert(!s.exhausted);assert(all[seat].assets.every(a=>!a.exhausted));
    assert.deepEqual(await api(path(seat,'commands'),paid[i].command,sessions[seat]),paid[i].result);
  }
  t.diagnostic(JSON.stringify({printedSociety:'MSJC09',syntheticInitialLayout:false,publicNaturalUiAcceptance:false,httpCalls:calls,acceptedCommands:commands,firstAndRearBothPaid:true,fourSeats:true,paidFrameReopens:reopens,restoredBeforeExpiry,sameInstanceNaturalNextTurnReset:true}));
});
