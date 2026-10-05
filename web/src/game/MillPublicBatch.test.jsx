import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent,cleanup} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as k from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import * as old from '../../../rust-game-wasm/legacy-v0.2.33/hegemony_wasm.js';
import {ReadModal} from './ReadModal';
import {ChoicePanel} from './ChoicePanel';
import {CardContent} from './CardTile';
import {cardScanUrl} from './cardScans';
import {validateDeckDraft} from './deckLibrary';
k.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
old.initSync({module:readFileSync(resolve('../rust-game-wasm/legacy-v0.2.33/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(k.catalog()),prior=JSON.parse(old.catalog());
const definitions=new Map(catalog.cards.map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/millPublicTest.fixture.json'),'utf8')).fixtures;
const view=(kind,seat=0)=>JSON.parse(k.view(fixtures.find(f=>f.kind===kind).state,seat));

it('admits two complete pinned originals while preserving every prior definition, society and preset',()=>{
 expect(catalog.engineVersion).toBe('rust-v0.2.34-mill-public-candidate');
 expect(catalog.cardPoolVersion).toBe('limited-v2.31-mill-public-candidate');
 expect(catalog.cards).toHaveLength(98);expect(catalog.societies).toHaveLength(8);
 expect(catalog.cards.filter(c=>!['XQ36','XQ46'].includes(c.id))).toEqual(prior.cards);
 expect(catalog.societies).toEqual(prior.societies);expect(catalog.decks).toEqual(prior.decks);
 expect(definitions.get('XQ36')).toMatchObject({name:'圣甲虫的清理员',kind:'character',cost:1,color:'黑',loyalty:['黑色'],subtypes:['人类','雇员'],defense:1,magic:'',unique:false,deckCopyLimit:3,permanentIcons:{investigation:0,combat:0,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0},abilities:[{key:'entry-mill-each-four',timing:'fast',costs:[],triggered:true}]});
 expect(definitions.get('XQ46')).toMatchObject({name:'无名尸体',kind:'character',cost:0,color:'中立',loyalty:[],subtypes:['人类'],defense:0,magic:'',unique:false,deckCopyLimit:3,keywords:['公开'],ruleTraits:{public:true},abilities:[]});
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));expect(Object.keys(scans)).toHaveLength(106);
 for(const [id,hash]of Object.entries({XQ36:'84b611615ecdbdf7940022160522187a28edb46322525e540fed1935de0526b9',XQ46:'d58e71df94753dde04595f5aa84de4039fcbbe594883acc018e2c06ddb103a89'})){
  expect(scans[id].sha256).toBe(hash);expect(cardScanUrl(id)).toBe('/cards/'+id+'.jpg');
  expect(createHash('sha256').update(readFileSync(resolve('public/cards/'+id+'.jpg'))).digest('hex')).toBe(hash);
 }
 const previous=JSON.parse(old.newGame('frozen33','LOCAL','teams','P0','watchers','1'));
 expect(()=>k.view(previous.state,0)).toThrow();expect(JSON.parse(old.view(previous.state,0))).toEqual(previous.view);
});

it('restores all 46 whole native fixtures through actual WASM and compares every seat projection',()=>{
 expect(fixtures).toHaveLength(46);
 for(const row of fixtures)for(let seat=0;seat<4;seat++)expect(JSON.parse(k.view(row.state,seat))).toEqual(row.views[seat]);
 const v=view('entry-choice-0-5');const submit=vi.fn();const action=v.legalActions.find(a=>a.kind==='choose');
 render(<ChoicePanel choice={v.pendingChoice} action={action} definitions={definitions} busy={false} onSubmit={submit} viewerId="p0"/>);
 fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(submit).toHaveBeenCalledExactlyOnceWith({...action,choiceId:v.pendingChoice.id,selected:[]});
 cleanup();
});

it('opens both original faces and renders the current corpse defense separately from its printed zero',()=>{
 for(const [id,kind]of [['XQ36','entry-choice-0-5'],['XQ46','corpse-aura-0']]){
  const card=view(kind).regions[2].characters.find(c=>c.cardId===id);
  render(<ReadModal card={card} definition={definitions.get(id)} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:definitions.get(id).name+'原始牌面'})).toHaveAttribute('src','/cards/'+id+'.jpg');cleanup();
 }
 const card=view('corpse-aura-0').regions[2].characters.find(c=>c.cardId==='XQ46');expect(card.defense).toBe(1);
 render(<CardContent card={card} definition={definitions.get('XQ46')} viewerId="p0"/>);
 expect(screen.getByText('当前防御 1 · 印刷防御 0')).toBeInTheDocument();cleanup();
 const back=view('corpse-forced-hidden').regions[2].characters.find(c=>c.cardId==='XQ46');
 for(let seat=1;seat<4;seat++){
  const projected=view('corpse-forced-hidden',seat).regions[2].characters.find(c=>c.instanceId===back.instanceId);expect(projected.cardId).toBeUndefined();
  render(<ReadModal card={projected} definition={definitions.get('XQ46')} viewerId={`p${seat}`} onClose={vi.fn()}/>);
  expect(screen.queryByRole('button',{name:'原始牌面'})).not.toBeInTheDocument();expect(screen.getByRole('dialog')).not.toHaveTextContent('无名尸体');cleanup();
 }
});

it('accepts a real 50-card mixed gray-black-red draft and rejects four copies of either new card',()=>{
 const cards=['XQ36','XQ46','JC059','JC049','JC093'].map(cardId=>({cardId,count:3}));cards.push({cardId:'JC125',count:35});
 const draft={id:'mill-mixed',name:'灰黑红50',description:'',societyId:null,cards,rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''};
 expect(validateDeckDraft(draft,catalog).valid).toBe(true);
 const room=JSON.parse(k.newGameWithDeck('mixed34','LOCAL','teams','P0',JSON.stringify(draft),'1'));expect(room.view.status).toBe('lobby');
 for(const cardId of ['XQ36','XQ46']){
  const invalid={...draft,cards:draft.cards.map(c=>({...c,count:c.cardId===cardId?4:c.cardId==='JC125'?34:c.count}))};
  expect(validateDeckDraft(invalid,catalog).valid).toBe(false);expect(()=>k.newGameWithDeck('invalid','LOCAL','teams','P0',JSON.stringify(invalid),'1')).toThrow();
 }
});
