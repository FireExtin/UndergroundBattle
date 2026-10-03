// Read-only source and in-memory deck validation; no browser, API or game start.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { transform } from '/workspace/UndergroundBattle/sites/node_modules/esbuild/lib/main.js';
import * as wasm from '/workspace/UndergroundBattle/rust-game-wasm/pkg/hegemony_wasm.js';
const root = '/workspace/UndergroundBattle';
const ui = '/workspace/UndergroundBattle-v027-player-sessions';
const sha = b => createHash('sha256').update(b).digest('hex');
const wasmBytes = fs.readFileSync(path.join(root, 'rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
wasm.initSync({module:wasmBytes});
const catalog = JSON.parse(wasm.catalog());
const transformed = await transform(fs.readFileSync(path.join(root,'web/src/game/deckLibrary.ts'),'utf8'),{loader:'ts',format:'esm'});
const frontend = await import(`data:text/javascript;base64,${Buffer.from(transformed.code).toString('base64')}`);
const ids = ['JC088','JC001','BQ083','JC006','XQ16','JC047','JC007'];
const spec = JSON.parse(fs.readFileSync(path.join(root,'docs/factions/card-specifications.json'),'utf8'));
const raw = JSON.parse(fs.readFileSync(path.join(root,'rust-game/data/cards.json'),'utf8'));
const getAt = (ref, file) => execFileSync('git',['show',`${ref}:${file}`],{cwd:root});
const ensure = (condition, label) => { if(!condition) throw new Error(label); };
function draftFrom(presetId, name, neutralCount, added) {
  const preset = catalog.decks.find(d=>d.id===presetId);
  const draft = frontend.createDeckDraft(catalog,preset);
  draft.id = `proposal-${presetId}`;
  draft.name = name;
  draft.updatedAt = '2026-10-03T00:00:00Z';
  draft.cards = draft.cards.map(e=> e.cardId==='JC125'?{...e,count:neutralCount}:{...e});
  draft.cards.push(...added.map(cardId=>({cardId,count:3})));
  return draft;
}
function validate(draft) {
  const client = frontend.validateDeckDraft(draft,catalog);
  let kernel;
  try {
    const result = JSON.parse(wasm.newGameWithDeck('IN_MEMORY_VALIDATION','NO_INVITE','teams','Validation',JSON.stringify(draft),'1'));
    // Only frozen lobby construction is inspected; no game commands are run.
    kernel = {accepted:true,status:result.view.status,total:result.view.yourDeck.cards.reduce((n,e)=>n+e.count,0),frozenCards:result.view.yourDeck.cards};
    ensure(result.view.status==='lobby','Validation must stay in lobby');
  } catch(error) {kernel = {accepted:false,error:String(error)}}
  return {client,kernel};
}
const membership = {};
for(const ref of ['37a09a2','cdf85ce','4b49609','618065c','8952f19']) {
  const source = JSON.parse(getAt(ref,'rust-game/data/cards.json'));
  membership[ref] = {decks:source.decks,presetSha:sha(JSON.stringify(source.decks)),addedCardCounts:Object.fromEntries(source.decks.map(d=>[d.id,d.cards.filter(e=>ids.includes(e.cardId))]))};
}
const recipes = [
  draftFrom('watchers','本地亲测草稿：黄调查与检索',8,['JC001','JC006','JC007']),
  draftFrom('reclaimers','本地亲测草稿：黑回收与蓝潜伏',2,['BQ083','XQ16']),
  draftFrom('responders','本地亲测草稿：红牺牲与群伤',14,['JC047']),
];
const recipeChecks = recipes.map(draft=>{
 const validation=validate(draft);
 ensure(validation.client.valid && validation.kernel.accepted && validation.client.total===50,'Recipe must be a legal fifty-card deck');
 const colorCounts={},magicCounts={};
 for(const e of draft.cards){
  const c=catalog.cards.find(c=>c.id===e.cardId);
  colorCounts[c.color]=(colorCounts[c.color]||0)+e.count;
  if(c.magic)magicCounts[c.magic]=(magicCounts[c.magic]||0)+e.count;
 }
 return {draft,validation,colorCounts,magicCounts};
});
const cardChecks=ids.map(id=>{
 const c=catalog.cards.find(c=>c.id===id);
 const s=spec.cards[id];
 const scan=fs.readFileSync(path.join(root,s.sourceVerification.imagePath));
 const publicScan=fs.readFileSync(path.join(root,`web/public/cards/${id}.jpg`));
 ensure(sha(scan)===s.sourceVerification.imageSha256 && sha(scan)===sha(publicScan),`${id} scan mismatch`);
 ensure(c.supported===true && c.deckCopyLimit===3 && frontend.isEditableDeckCard(c),`${id} not editable`);
 const valid=validate(draftFrom('watchers',`仅验证${id}`,14,[id]));
 ensure(valid.client.valid && valid.kernel.accepted,`${id} legal deck rejected`);
 const four= draftFrom('watchers',`超限验证${id}`,13,[id]);
 four.cards.find(e=>e.cardId===id).count=4;
 const rejected=validate(four);
 ensure(!rejected.client.valid && !rejected.kernel.accepted,`${id} four copies unexpectedly accepted`);
 return {id,name:c.name,kind:c.kind,supported:c.supported,deckCopyLimit:c.deckCopyLimit,cost:c.cost,loyalty:c.loyalty,color:c.color,magic:c.magic,source:s.sourceVerification,edition:s.edition,openQuestionIds:s.openQuestionIds,originalSha:sha(scan),uiSha:sha(publicScan),threeCopies:valid,fourCopies:rejected};
});
const oldDraft=structuredClone(recipes[0]);oldDraft.engineVersion='rust-v0.2.7';oldDraft.cardPoolVersion='limited-v2.5';
const wrongVersion=validate(oldDraft);
ensure(!wrongVersion.client.valid && !wrongVersion.kernel.accepted,'Pinned old version must be rejected');
const uiPaths=['Lobby.tsx','DeckLibrary.tsx','deckLibrary.ts','api.ts'].map(f=>{
  const file=`web/src/game/${f}`;const a=fs.readFileSync(path.join(root,file)),b=fs.readFileSync(path.join(ui,file));
 if(f==='deckLibrary.ts'||f==='DeckLibrary.tsx') ensure(a.equals(b),`${file} deck editor or validator unexpectedly differs`);
 return {file,cardBranchSha256:sha(a),uiBranchSha256:sha(b),identicalBetweenCardAndUiBranches:a.equals(b)};
});
const publicPool=JSON.parse(getAt('37a09a2','rust-game/data/cards.json'));
const artifact={
 scope:'Source audit and isolated in-memory lobby validation only; no UI natural-play acceptance, live room, API, browser, publish or production preset edit.',
 implementationCommit:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),
 uiCommit:execFileSync('git',['rev-parse','HEAD'],{cwd:ui,encoding:'utf8'}).trim(),
 publicImplementationBaseline:'97d6c82dc920e0b7f7cc10141e1831b2f055cb84',
 currentVersions:{rules:catalog.rulesVersion,pool:catalog.cardPoolVersion,engine:catalog.engineVersion},
 catalogDefinitions:catalog.cards.length,editablePlayerDefinitions:catalog.cards.filter(frontend.isEditableDeckCard).length,
 wasm:{bytes:wasmBytes.length,sha256:sha(wasmBytes)},cardChecks,membership,recipeChecks,wrongVersion,uiPaths,
 publicBaselinePresence:Object.fromEntries(ids.map(id=>[id,publicPool.cards.some(c=>c.id===id)])),
 unaffected:{presetDataMatchesSource:JSON.stringify(catalog.decks)===JSON.stringify(raw.decks)},
 disk:execFileSync('df',['-h','/workspace','/tmp'],{encoding:'utf8'}),
};
fs.writeFileSync('/workspace/UndergroundBattle/docs/evidence/seven-card-deck-audit-2026-10-03/audit.json',JSON.stringify(artifact,null,2)+'\n');
console.log(JSON.stringify({cardChecks:cardChecks.map(c=>({id:c.id,editable:true,threeAccepted:c.threeCopies.kernel.accepted,fourRejected:!c.fourCopies.kernel.accepted})),recipes:recipeChecks.map(r=>({name:r.draft.name,total:r.validation.client.total,valid:r.validation.client.valid,colors:r.colorCounts,magic:r.magicCounts})),wrongVersionRejected:!wrongVersion.kernel.accepted,wasm:artifact.wasm},null,2));
