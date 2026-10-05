// node source/tools/validation/verify-jc030.mjs source evidence
// Exact actual32/native JC030-only states, transitions, rejects and four-seat
// views. Explicit offline fixtures; no public room or full green union.
import {readFileSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root=resolve(process.argv[2]),evidence=resolve(process.argv[3]);
const load=async folder=>{const k=await import(pathToFileURL(root+'/rust-game-wasm/'+folder+'/hegemony_wasm.js'));const bytes=readFileSync(root+'/rust-game-wasm/'+folder+'/hegemony_wasm_bg.wasm');k.initSync({module:bytes});return{k,bytes,catalog:JSON.parse(k.catalog())};};
const {k,bytes,catalog}=await load('pkg'),{k:old,bytes:frozen,catalog:prior}=await load('legacy-v0.2.31');
assert.equal(catalog.engineVersion,'rust-v0.2.32-jc030-blue-bat-candidate');assert.equal(catalog.cardPoolVersion,'limited-v2.29-jc030-blue-bat-candidate');
assert.equal(catalog.cards.length,94);assert.equal(catalog.societies.length,7);
assert.deepEqual(catalog.cards.filter(c=>c.id!=='JC030'),prior.cards);assert.deepEqual(catalog.societies,prior.societies);assert.deepEqual(catalog.decks,prior.decks);
assert.equal(createHash('sha256').update(frozen).digest('hex'),'d5c544fde889abf066cf794b24b62106d439aa453e9e6a210e88217e3e872bbe');
let commands=0,checkpoints=0,rejects=0,views=0;
for(const directory of ['native-bodies','fixed-real-bodies','fixed-asset-bodies'])for(const file of readdirSync(evidence+'/'+directory).sort()){
 const j=JSON.parse(readFileSync(evidence+'/'+directory+'/'+file,'utf8'));
 if(file.startsWith('checkpoint-')){for(let s=0;s<4;s++){assert.deepEqual(JSON.parse(k.view(j.state,s)),j.views[s],directory+'/'+file+' seat'+s);views++;}checkpoints++;}
 else if(file.startsWith('step-')||file.startsWith('reject-')){
  const actual=JSON.parse(k.applyRoom(j.state,j.seat,JSON.stringify(j.command),'0'));assert.deepEqual(actual,j.expected,directory+'/'+file);
  for(let s=0;s<4;s++){assert.deepEqual(JSON.parse(k.view(actual.state,s)),j.views[s],directory+'/'+file+' seat'+s);views++;}
  if(file.startsWith('reject-')){assert.equal(actual.outcome,'rejected');assert.equal(actual.state,j.state);rejects++;}else commands++;
 }else throw Error(file);
}
const fixtures=JSON.parse(readFileSync(root+'/web/src/game/jc030Test.fixture.json','utf8')).fixtures;
for(const row of fixtures)for(let s=0;s<4;s++){assert.deepEqual(JSON.parse(k.view(row.state,s)),row.views[s]);views++;}
assert.equal(fixtures.length,5);assert(commands>0&&checkpoints>0&&rejects>=3);
const lobby=JSON.parse(old.newGame('old31-offline','LOCAL','teams','P0','watchers','1'));assert.throws(()=>k.view(lobby.state,0));assert.deepEqual(JSON.parse(old.view(lobby.state,0)),lobby.view);
console.log(JSON.stringify({ok:true,wasmSha256:createHash('sha256').update(bytes).digest('hex'),commands,checkpoints,rejects,comparedFullSeatViews:views,frontendWholeFixtures:fixtures.length,allPrior93DefinitionsExact:true,allSevenSocietiesExact:true,allFivePresetsExact:true,frozen31Exact:true,newKernelRejectsOldIdentities:true,scope:'JC030-only explicit native fixture bodies including preserved partial first attempts; successful distinct unit coverage in logs; not full green union or natural public play'},null,2));
