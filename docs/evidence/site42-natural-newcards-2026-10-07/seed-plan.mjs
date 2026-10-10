import fs from 'node:fs';
import * as abi from '/tmp/lantern-site-fce0-official/rust-game-wasm/pkg/hegemony_wasm.js';
abi.initSync({ module: fs.readFileSync('/tmp/lantern-site-fce0-official/rust-game-wasm/pkg/hegemony_wasm_bg.wasm') });
const catalog = JSON.parse(abi.catalog());
function deck(id, ids) {
  return { id, name: id, description: 'Local natural new-card QA: legal fixed deck, no board injection', societyId: null,
    cards: [...ids.map(cardId => ({cardId,count:3})),{cardId:'JC125',count:50-ids.length*3}],
    rulesVersion:catalog.rulesVersion, cardPoolVersion:catalog.cardPoolVersion, engineVersion:catalog.engineVersion, updatedAt:'2026-10-07' };
}
const decks = [deck('Local seal QA', ['XQ40','XQ41','XQ45','JZ58','JC104','JZ59','JZ61']),
  deck('Local reveal QA', ['XQ16','JZ50','JZ49','JC085','JC029','XQ12','JZ48'])];
const results=[];
for (let seed=1;seed<=32;seed++) {
  let room=JSON.parse(abi.newGameWithDeck('0123456789abcdef01234567','LOCALQA','duel','Local UI A',JSON.stringify(decks[0]),String(seed)));
  room=JSON.parse(abi.joinGameWithDeck(room.state,'Local API B',JSON.stringify(decks[1])));
  let n=0;
  for(const [seat,action] of [[0,{kind:'ready'}],[1,{kind:'ready'}],[0,{kind:'start'}]]) {
    room=JSON.parse(abi.applyRoom(room.state,seat,JSON.stringify({commandId:'seed-plan-'+seed+'-'+n++,expectedVersion:room.version,action:{kind:'game',action}}),'0'));
  }
  const hands=[0,1].map(seat=>JSON.parse(abi.view(room.state,seat)).hand.map(c=>c.cardId));
  const score=(hands[0].includes('XQ40')?8:0)+(hands[0].includes('XQ41')?8:0)
    +(hands[1].includes('JZ50')?6:0)+(hands[1].includes('XQ16')?5:0)
    +hands[0].filter(id=>id!=='JC125').length;
  results.push({seed,hands,score});
}
results.sort((a,b)=>b.score-a.score||a.seed-b.seed);
const report={scope:'Only32 offline actual-WASM factory seeds; legal decks, ready/start actions; no altered state fed back',decks,
  selected:results.slice(0,3),allOpeningHands:results};
fs.writeFileSync('/workspace/game-publication-evidence/site42-natural-seed-plan.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({selected:report.selected}));
