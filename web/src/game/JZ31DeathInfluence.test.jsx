import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render, screen, fireEvent, cleanup} from '@testing-library/react';
import {expect, it, vi} from 'vitest';
import { kernel as k } from './testKernel';
import {ReadModal} from './ReadModal';
import {ChoicePanel} from './ChoicePanel';
import {cardScanUrl} from './cardScans';
import {validateDeckDraft} from './deckLibrary';

const catalog=JSON.parse(k.catalog());
const definitions=new Map(catalog.cards.map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/jz31Test.fixture.json'),'utf8')).fixtures;
const view=(kind,seat=0)=>fixtures.find(f=>f.kind===kind).views[seat];

it('checks the complete JZ31 printed definition without a magic requirement',()=>{

 expect(definitions.get('JZ31')).toMatchObject({name:'破茧者秘教线人',kind:'character',cost:3,color:'红',loyalty:['红色'],subtypes:['人类','宿主'],defense:1,magic:'',magicIcon:'None',unique:false,deckCopyLimit:3,permanentIcons:{investigation:1,combat:0,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0},abilities:[{key:'death-source-influence',timing:'fast',costs:[],triggered:true}]});
});

it('checks recorded death-controller choice privacy and influence for every seat',()=>{
 expect(fixtures).toHaveLength(40);
 for(let actor=0;actor<4;actor++) {
  const v=view(`borrowed-death-${actor}`,actor);
  expect(v.pendingChoice.playerId).toBe(`p${actor}`);
  for(let seat=0;seat<4;seat++) if(seat!==actor) expect(view(`borrowed-death-${actor}`,seat).pendingChoice).toBeNull();
  const result=view(`death-result-JC091-${actor}`,actor);
  expect(result.regions[2].influence).toEqual(actor<2?[1,0]:[0,1]);
 }
});

it('offers both skip and one acceptance to the death controller using the existing choice UI',()=>{
 const v=view('borrowed-death-0');
 const action=v.legalActions.find(a=>a.kind==='choose'); const submit=vi.fn();
 render(<ChoicePanel choice={v.pendingChoice} action={action} definitions={definitions} busy={false} onSubmit={submit} viewerId="p0"/>);
 fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));
 expect(submit).toHaveBeenLastCalledWith({...action,choiceId:v.pendingChoice.id,selected:[]});
 fireEvent.click(screen.getByRole('button',{name:/发动触发能力/}));
 fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
 expect(submit).toHaveBeenLastCalledWith({...action,choiceId:v.pendingChoice.id,selected:['accept']});
 cleanup();
});

it('opens the pinned original and does not confuse death trigger text with a magic requirement',()=>{
 const hash='57a1fd51243ab4616af46f4fb8a6a1ef46a09ab3c9a11ed2fe2fc90d8b83babf';
 const scans=JSON.parse(readFileSync(resolve('src/game/cardScansV035.fixture.json'),'utf8'));
 expect(Object.keys(scans)).toHaveLength(107); expect(scans.JZ31.sha256).toBe(hash);
 expect(cardScanUrl('JZ31')).toBe('/cards/JZ31.jpg');
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JZ31.jpg'))).digest('hex')).toBe(hash);
 const card=view('deployed-no-magic').regions[2].characters.find(c=>c.cardId==='JZ31');
 render(<ReadModal card={card} definition={definitions.get('JZ31')} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByRole('dialog')).toHaveTextContent('死亡触发');
 expect(screen.getByRole('dialog')).toHaveTextContent('人类');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
 expect(screen.getByRole('img',{name:'破茧者秘教线人原始牌面'})).toHaveAttribute('src','/cards/JZ31.jpg');
 cleanup();
});

it('accepts a real fifty-card red deck with three copies and rejects four',()=>{
 const draft={id:'jz31-red',name:'红色死亡影响力',description:'',societyId:null,cards:[{cardId:'JZ31',count:3},{cardId:'JC049',count:3},{cardId:'JC047',count:3},{cardId:'JC125',count:41}],rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''};
 expect(validateDeckDraft(draft,catalog).valid).toBe(true);
 expect(JSON.parse(k.newGameWithDeck('jz31-deck','LOCAL','teams','P0',JSON.stringify(draft),'1')).view.status).toBe('lobby');
 const invalid={...draft,cards:draft.cards.map(c=>({...c,count:c.cardId==='JZ31'?4:c.cardId==='JC125'?40:c.count}))};
 expect(validateDeckDraft(invalid,catalog).valid).toBe(false);
 expect(()=>k.newGameWithDeck('invalid-jz31','LOCAL','teams','P0',JSON.stringify(invalid),'1')).toThrow();
});
