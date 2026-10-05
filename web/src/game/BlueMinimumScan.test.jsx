import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent,cleanup} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as k from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import * as accepted33 from '../../../rust-game-wasm/legacy-v0.2.33/hegemony_wasm.js';
import * as accepted32 from '../../../rust-game-wasm/legacy-v0.2.32/hegemony_wasm.js';
import {ChoicePanel} from './ChoicePanel';
import {ReadModal} from './ReadModal';
import {cardScanUrl} from './cardScans';
import {validateDeckDraft} from './deckLibrary';
k.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
accepted33.initSync({module:readFileSync(resolve('../rust-game-wasm/legacy-v0.2.33/hegemony_wasm_bg.wasm'))});
accepted32.initSync({module:readFileSync(resolve('../rust-game-wasm/legacy-v0.2.32/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(k.catalog()),old=JSON.parse(accepted32.catalog());
const definitions=new Map([...catalog.cards,...catalog.societies].map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/blueMinimumTest.fixture.json'),'utf8')).fixtures;
const view=(kind,seat)=>JSON.parse(accepted33.view(fixtures.find(f=>f.kind===kind).state,seat));
const choiceAction=v=>v.legalActions.find(a=>a.kind==='choose');

it('blue33 preserves all old94 definitions, seven societies, five presets and opens only the three original scans',()=>{
 expect(catalog.engineVersion).toBe('rust-v0.2.34-mill-public-candidate');expect(catalog.cardPoolVersion).toBe('limited-v2.31-mill-public-candidate');
 expect(catalog.cards).toHaveLength(98);expect(catalog.societies).toHaveLength(8);
 expect(catalog.cards.filter(c=>!['JC032','JZ24','XQ36','XQ46'].includes(c.id))).toEqual(old.cards);
 expect(catalog.societies.filter(c=>c.id!=='MSJC03')).toEqual(old.societies);expect(catalog.decks).toEqual(old.decks);
 expect(definitions.get('JC032')).toMatchObject({cost:5,loyalty:['蓝色','蓝色'],magic:'死亡',subtypes:['吸血鬼'],defense:3,permanentIcons:{investigation:1,combat:1,influence:2},unique:false});
 expect(definitions.get('JZ24')).toMatchObject({cost:4,loyalty:['蓝色'],magic:'死亡',subtypes:['吸血鬼','罪犯'],defense:1,permanentIcons:{investigation:1,combat:2,influence:0},unique:false});
 expect(definitions.get('MSJC03')).toMatchObject({startingHand:6,deckConstraints:[{kind:'minimumColor',color:'蓝',count:25}],subtypes:['法师结社','吸血鬼']});
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));expect(Object.keys(scans)).toHaveLength(106);
 for(const [id,hash]of Object.entries({JC032:'ebeb0a13c62cdf10569f4aa9f300998aff53a65788d756f055e9696f35489b4b',JZ24:'e01995d70e06a6b179dcbbf81c3df73ab548269fc0efe9fafc33e2e2f3c8722d',MSJC03:'07463a18a5cc6726f098bedc10304ef6c4032929cff72535d5f9227338e45f29'})){
  expect(scans[id].sha256).toBe(hash);expect(cardScanUrl(id)).toBe('/cards/'+id+'.jpg');
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/'+id+'.jpg'))).digest('hex')).toBe(hash);
 }
 expect(cardScanUrl('XQ11')).toBeUndefined();
 const lobby=JSON.parse(accepted32.newGame('old32-reader','LOCAL','teams','P0','watchers','1'));
 expect(()=>k.view(lobby.state,0)).toThrow();expect(JSON.parse(accepted32.view(lobby.state,0))).toEqual(lobby.view);
});

it('blue33 restores every complete native fixture through actual WASM for four seats',()=>{
 expect(fixtures).toHaveLength(11);
 for(const row of fixtures)for(let seat=0;seat<4;seat++)expect(JSON.parse(accepted33.view(row.state,seat))).toEqual(row.views[seat]);
});

it('JC032 privately previews all six including Bat and human, reads without choosing, and requires one printed Vampire',()=>{
 const v=view('jc032-positive',0),p=v.pendingChoice,submit=vi.fn(),read=vi.fn();
 expect(p.previewCards).toHaveLength(6);expect(p.options.map(o=>o.card.cardId)).toEqual(['JC029','JZ24']);
 expect(p.previewCards.map(c=>c.cardId)).toEqual(['JC030','JC029','XQ16','JZ24','JC003','JC030']);
 for(let seat=1;seat<4;seat++){
  const other=view('jc032-positive',seat);expect(other.pendingChoice).toBeNull();
  for(const c of p.previewCards)expect(JSON.stringify(other)).not.toContain(c.instanceId);
 }
 const {container}=render(<ChoicePanel choice={p} action={choiceAction(v)} definitions={definitions} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0"/>);
 expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
 fireEvent.click(screen.getAllByRole('button',{name:'放大阅读巨型蝙蝠'})[0]);expect(read.mock.calls[0][0].cardId).toBe('JC030');expect(submit).not.toHaveBeenCalled();
 expect(container.querySelector(`[data-choice-option="${p.previewCards[0].instanceId}"]`)).toBeNull();
 fireEvent.click(container.querySelector(`[data-choice-option="${p.options[0].id}"]`));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
 expect(submit).toHaveBeenCalledExactlyOnceWith({...choiceAction(v),choiceId:p.id,selected:[p.options[0].id]});
 cleanup();render(<ReadModal card={read.mock.calls[0][0]} definition={definitions.get('JC030')} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'巨型蝙蝠原始牌面'})).toHaveAttribute('src','/cards/JC030.jpg');
});

it('JC032 zero-hit view remains readable and submits explicit zero confirmation without a skip',()=>{
 const v=view('jc032-zero-hit',0),submit=vi.fn(),read=vi.fn();
 render(<ChoicePanel choice={v.pendingChoice} action={choiceAction(v)} definitions={definitions} busy={false} onSubmit={submit} onReadCard={read} viewerId="p0"/>);
 expect(v.pendingChoice.options).toEqual([]);expect(v.pendingChoice.previewCards).toHaveLength(6);
 fireEvent.click(screen.getAllByRole('button',{name:'放大阅读巨型蝙蝠'})[0]);expect(submit).not.toHaveBeenCalled();expect(read).toHaveBeenCalledOnce();
 expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
 expect(submit).toHaveBeenCalledExactlyOnceWith({...choiceAction(v),choiceId:v.pendingChoice.id,selected:[]});
});

it('JC032 publicly shows the selected name but only its current controller can read the new borrowed hidden instance',()=>{
 const actual=view('jc032-borrowed-hidden-result',0).regions[2].characters.find(c=>c.cardId==='JC029');
 expect(actual).toMatchObject({owner:'p2',controller:'p0',faceDown:true});
 for(let seat=0;seat<4;seat++){
  cleanup();const v=view('jc032-borrowed-hidden-result',seat),card=v.regions[2].characters.find(c=>c.instanceId===actual.instanceId);
  expect(!!card.cardId).toBe(seat===0);render(<ReadModal card={seat===2?actual:card} definition={definitions.get('JC029')} viewerId={`p${seat}`} onClose={vi.fn()}/>);
  expect(!!screen.queryByRole('button',{name:'原始牌面'})).toBe(seat===0);
  if(seat!==0){expect(screen.getByRole('dialog')).not.toHaveTextContent('新生血族');expect(screen.queryByRole('img')).not.toBeInTheDocument();}
  expect(v.log.some(e=>e.text.includes('展示检索的 新生血族'))).toBe(true);
 }
});

it('JZ24 makes the second enemy with four cards choose after BQ022 return and refresh resets the prior selection',()=>{
 const a=view('jz24-first-enemy',0),b=view('jz24-bq022-second-enemy',1),submit=vi.fn();
 expect(b.hand).toHaveLength(4);expect(b.hand.some(c=>c.cardId==='BQ022')).toBe(true);
 for(let seat=0;seat<4;seat++)expect(!!view('jz24-bq022-second-enemy',seat).pendingChoice).toBe(seat===1);
 const props=v=>({choice:v.pendingChoice,action:choiceAction(v),definitions,busy:false,onSubmit:submit,viewerId:v.you});
 const {container,rerender}=render(<ChoicePanel key={a.pendingChoice.id} {...props(a)}/>);
 fireEvent.click(container.querySelector(`[data-choice-option="${a.pendingChoice.options[0].id}"]`));
 expect(screen.getByRole('button',{name:'确认选择'})).toBeEnabled();
 rerender(<ChoicePanel key={b.pendingChoice.id} {...props(b)}/>);
 expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
 fireEvent.click(container.querySelector(`[data-choice-option="${b.pendingChoice.options[0].id}"]`));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
 expect(submit).toHaveBeenCalledExactlyOnceWith({...choiceAction(b),choiceId:b.pendingChoice.id,selected:[b.pendingChoice.options[0].id]});
});

it('blue minimum draft enforces the 24/25/27 boundary and ordinary copy limit',()=>{
 const blues=['XQ12','XQ16','JC036','JZ27','XQ14','JC029','JC030','JC032','JZ24'];
 const draft=n=>{let left=n;const cards=blues.flatMap(cardId=>{const count=Math.min(3,left);left-=count;return count?[{cardId,count}]:[];});cards.push({cardId:'JC125',count:50-n});return{id:'blue-reader',name:'蓝色最小50',description:'',societyId:'MSJC03',cards,rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''};};
 for(const n of [24,25,27])expect(validateDeckDraft(draft(n),catalog).valid).toBe(n>=25);
 const four=draft(27);four.cards[0].count=4;four.cards.at(-1).count--;expect(validateDeckDraft(four,catalog).valid).toBe(false);
});

it('all three new cards open their actual original faces from complete projected views',()=>{
 for(const [id,kind,seat]of [['JC032','jc032-positive',0],['JZ24','jz24-first-enemy',2],['MSJC03','msjc03-search-result',0]]){
  cleanup();const v=view(kind,seat),card=id==='MSJC03'?v.societyZones[0].card:v.regions[2].characters.find(c=>c.cardId===id);
  render(<ReadModal card={card} definition={definitions.get(id)} viewerId={`p${seat}`} onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definitions.get(id).name+'原始牌面'})).toHaveAttribute('src','/cards/'+id+'.jpg');
 }
});
