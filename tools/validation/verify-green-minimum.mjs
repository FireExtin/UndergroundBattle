// Portable review: node source/tools/validation/verify-green-minimum.mjs source evidence
// Read-only. Compares complete native transitions, complete four-seat views and
// rejected payloads with the supplied actual default WASM. No live-room calls.
import {readFileSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root=resolve(process.argv[2]||fileURLToPath(new URL('../..',import.meta.url)));
const evidence=resolve(process.argv[3]||root+'/docs/evidence/green-minimum-2026-10-05');
const load=async folder=>{const module=await import(pathToFileURL(root+'/rust-game-wasm/'+folder+'/hegemony_wasm.js'));const bytes=readFileSync(root+'/rust-game-wasm/'+folder+'/hegemony_wasm_bg.wasm');module.initSync({module:bytes});return{module,bytes,catalog:JSON.parse(module.catalog())};};
const {module:k,bytes,catalog:c}=await load('pkg');
const {module:old30,catalog:p}=await load('legacy-v0.2.30');
const {module:old29}=await load('legacy-v0.2.29');
assert.equal(c.engineVersion,'rust-v0.2.31-green-minimum-candidate');
assert.equal(c.cardPoolVersion,'limited-v2.28-green-minimum-candidate');
assert.equal(c.cards.length,93);assert.equal(c.societies.length,7);
assert.deepEqual(c.cards.filter(x=>!['LC30','JC018','JC015'].includes(x.id)),p.cards);
assert.deepEqual(c.societies.slice(0,6),p.societies);assert.deepEqual(c.decks,p.decks);
assert(!c.cards.some(x=>x.id==='XQ11'));assert(!c.societies.some(x=>x.id==='MSJC03'));
for(const old of [old29,old30]){const room=JSON.parse(old.newGame('green-old-local','LOCAL','teams','P0','watchers','1'));assert.throws(()=>k.view(room.state,0));assert.deepEqual(JSON.parse(old.view(room.state,0)),room.view);}
const ids=['JC014','JC016','JZ08','BQ022','JC020','XQ07','LC30','JC018','JC015'];
function draft(n,total=50){let left=n;const cards=ids.flatMap(cardId=>{const count=Math.min(3,left);left-=count;return count?[{cardId,count}]:[];});cards.push({cardId:'JC125',count:total-n});return{id:'green-wasm',name:'绿色最小50',description:'',societyId:'MSJC02',cards,rulesVersion:c.rulesVersion,cardPoolVersion:c.cardPoolVersion,engineVersion:c.engineVersion,updatedAt:''};}
assert.throws(()=>k.newGameWithDeck('reject24','LOCAL','teams','P0',JSON.stringify(draft(24)),'19'));
assert.throws(()=>k.newGameWithDeck('reject49','LOCAL','teams','P0',JSON.stringify(draft(25,49)),'19'));
const four=draft(27);four.cards[0].count=4;four.cards.at(-1).count--;
assert.throws(()=>k.newGameWithDeck('reject-copy','LOCAL','teams','P0',JSON.stringify(four),'19'));
assert.throws(()=>k.newGameWithDeck('reject-blue','LOCAL','teams','P0',JSON.stringify({...draft(27),societyId:'MSJC03'}),'19'));
JSON.parse(k.newGameWithDeck('valid25','LOCAL','teams','P0',JSON.stringify(draft(25)),'19'));
let room=JSON.parse(k.newGameWithDeck('green-wasm-normal-start','LOCAL','teams','P0',JSON.stringify(draft(27)),'19'));
for(let seat=1;seat<4;seat++)room=JSON.parse(k.joinGameWithDeck(room.state,'P'+seat,JSON.stringify(draft(27))));
for(const [seat,kind]of [[0,'ready'],[1,'ready'],[2,'ready'],[3,'ready'],[0,'start']]){room=JSON.parse(k.applyRoom(room.state,seat,JSON.stringify({commandId:'normal-'+seat+'-'+kind,expectedVersion:room.version,action:{kind:'game',action:{kind}}}),'0'));assert.equal(room.outcome,'accepted');}
for(let seat=0;seat<4;seat++)assert.equal(JSON.parse(k.view(room.state,seat)).hand.length,6);
let checkpoints=0,commands=0,rejections=0,views=0,beginResponses=0,submitResponses=0;
for(const directory of ['union-bodies','guard-combat-bodies'])for(const file of readdirSync(evidence+'/'+directory).sort()){
 const row=JSON.parse(readFileSync(evidence+'/'+directory+'/'+file,'utf8'));
 if(file.startsWith('checkpoint-')){for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(row.state,seat)),row.views[seat],file+' seat'+seat);views++;}checkpoints++;}
 else if(file.startsWith('step-')||file.startsWith('reject-')){
  const actual=JSON.parse(k.applyRoom(row.state,row.seat,JSON.stringify(row.command),'0'));assert.deepEqual(actual,row.expected,file);
  for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(actual.state,seat)),row.views[seat],file+' seat'+seat);views++;}
  if(file.startsWith('reject-')){assert.equal(actual.outcome,'rejected');assert.equal(actual.state,row.state);rejections++;}
  else{commands++;beginResponses+=Number(row.command.action.kind==='beginResponse');submitResponses+=Number(row.command.action.kind==='submitResponse');}
 }else throw Error('Unexpected evidence body '+file);
}
assert(checkpoints>0&&commands>0&&rejections>0&&beginResponses>0&&submitResponses>0);
const fixture=JSON.parse(readFileSync(root+'/web/src/game/greenMinimumTest.fixture.json','utf8'));
for(const row of fixture.fixtures)for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(k.view(row.state,seat)),row.views[seat]);views++;}
console.log(JSON.stringify({ok:true,wasmSha256:createHash('sha256').update(bytes).digest('hex'),commands,checkpoints,rejections,comparedFullSeatViews:views,beginResponses,submitResponses,frontendWholeFixtures:fixture.fixtures.length,old90DefinitionsEqual:true,oldSixSocietiesEqual:true,oldFivePresetsEqual:true,old29And30LocalLobbiesReadable:true,newKernelRejectsOldIdentities:true,normalFourSeatStartSix:true,rejectedCreations:4,scope:'default actual WASM/native fixture union with complete schema3 states, transitions, rejection bodies and four-seat views; not natural public gameplay, live old-room compatibility or full433 audit'},null,2));
