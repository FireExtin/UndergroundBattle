import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent,cleanup} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import { kernel as k } from './testKernel';
import {ReadModal} from './ReadModal';
import {Table} from './Table';
import {ChoicePanel} from './ChoicePanel';
import {cardScanUrl} from './cardScans';
import {createDeckDraft,validateDeckDraft,saveDeckLibrary,readDeckLibrary,DECK_LIBRARY_STORAGE_KEY} from './deckLibrary';

const catalog=JSON.parse(k.catalog());
const definitions=new Map(catalog.cards.map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/jz55Test.fixture.json'),'utf8')).fixtures;
const view=(kind,seat=0)=>fixtures.find(f=>f.kind===kind).views[seat];
afterEach(()=>{cleanup();localStorage.removeItem(DECK_LIBRARY_STORAGE_KEY);});

it('checks the complete JZ55 printed definition',()=>{

 expect(definitions.get('JZ55')).toMatchObject({name:'传奇落幕',kind:'spell',cost:2,color:'黑',loyalty:['黑色'],subtypes:['命运'],magic:'',magicIcon:'None',unique:false,deckCopyLimit:3,abilities:[{key:'destroy-unique',timing:'fast',costs:[],triggered:false}]});
});

it('checks recorded immediate composition and subsequent death-trigger views',()=>{
 expect(fixtures).toHaveLength(21);
 const composed=view('real-composing',2);expect(composed.responseWindow.myIntentId).toBe('jz55-real-response');expect(composed.stack).toHaveLength(1);
 const after=view('immediate-response-complete',2);expect(after.stack).toHaveLength(1);expect(after.stack[0].cardId).toBe('JC063');
 expect(after.graveyard.filter(c=>c.cardId==='JZ55')).toHaveLength(1);
 const death=view('cascade-death-stack',2);expect(death.responseWindow).not.toBeNull();expect(death.stack).toHaveLength(1);
 expect(view('cascade-final',2).regions[2].influence).toEqual([0,1]);
});

it('opens the exact original and displays cannot respond without inventing a magic requirement',()=>{
 const hash='9155faa90be22aa66e56148dfacd0d517eb7b6f7b56fd1b45b71a6804719784a';
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));
 expect(Object.keys(scans)).toHaveLength(115);expect(scans.JZ55.sha256).toBe(hash);expect(cardScanUrl('JZ55')).toBe('/cards/JZ55.jpg');
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JZ55.jpg'))).digest('hex')).toBe(hash);
 const card=view('targets-seat0').hand.find(c=>c.cardId==='JZ55');
 render(<ReadModal card={card} definition={definitions.get('JZ55')} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByRole('dialog')).toHaveTextContent('传奇落幕不能被响应');expect(screen.getByRole('dialog')).toHaveTextContent('消灭目标独有角色');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
 expect(screen.getByRole('img',{name:'传奇落幕原始牌面'})).toHaveAttribute('src','/cards/JZ55.jpg');
});

it.each([0,2])('selects only native face-up unique instances, including an owned hidden unique counterexample, for seat%s',seat=>{
 const v=view(`targets-seat${seat}`,seat),spell=v.hand.find(c=>c.cardId==='JZ55');
 const actions=v.legalActions.filter(a=>a.kind==='play'&&a.cardId===spell.instanceId);expect(actions).toHaveLength(4);
 const submit=vi.fn(),m=render(<Table view={v} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看传奇落幕',exact:true}));
 fireEvent.click(screen.getByRole('button',{name:/传奇落幕：快速行动.*不可响应/}));
 for(const region of v.regions)for(const card of region.characters){
  const node=m.container.querySelector(`[data-card-instance="${card.instanceId}"]`);
  expect(node).toHaveAttribute('data-card-targeted',String(actions.some(a=>a.targetId===card.instanceId)));
 }
 for(const attachment of v.attachments)expect(actions.some(a=>a.targetId===attachment.instanceId)).toBe(false);
 const selected=actions.find(a=>v.regions[2].characters.some(c=>c.instanceId===a.targetId));
 fireEvent.click(m.container.querySelector(`[data-card-instance="${selected.targetId}"]`));
 fireEvent.click(screen.getByRole('button',{name:`确认 · ${selected.label}`}));
 expect(submit).toHaveBeenCalledExactlyOnceWith(selected);
 m.rerender(<Table view={view(`ordinary-seat${seat}`,seat)} catalog={catalog} busy={false} onAction={submit}/>);
 expect(m.container.querySelector('[aria-label="确认目标行动"]')).not.toBeInTheDocument();
});

it('keeps actual composing selection readable and immediately shows only the old underlying stack after submission',()=>{
 const v=view('real-composing',2),submit=vi.fn(),m=render(<Table view={v} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看传奇落幕',exact:true}));
 fireEvent.click(screen.getByRole('button',{name:/传奇落幕：快速行动.*不可响应/}));
 const a=v.legalActions.find(a=>a.kind==='play'&&a.cardId===v.hand.find(c=>c.cardId==='JZ55').instanceId);
 fireEvent.click(m.container.querySelector(`[data-card-instance="${a.targetId}"]`));
 fireEvent.click(screen.getByRole('button',{name:`确认 · ${a.label}`}));expect(submit).toHaveBeenCalledExactlyOnceWith(a);
 const after=view('immediate-response-complete',2);m.rerender(<Table view={after} catalog={catalog} busy={false} onAction={submit}/>);
 expect(after.stack[0].id).toBe(v.stack[0].id);expect(after.responseWindow.id).not.toBe(v.responseWindow.id);
 expect(m.container.querySelector('[aria-label="确认目标行动"]')).not.toBeInTheDocument();
});

it('offers ordinary optional acceptance to the cascade death controller and leaves its effect respondable',()=>{
 const v=view('cascade-death-choice',2),action=v.legalActions.find(a=>a.kind==='choose'),submit=vi.fn();
 expect(view('cascade-death-choice',0).pendingChoice).toBeNull();
 render(<ChoicePanel choice={v.pendingChoice} action={action} definitions={definitions} busy={false} onSubmit={submit} viewerId="p2"/>);
 fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(submit).toHaveBeenLastCalledWith({...action,choiceId:v.pendingChoice.id,selected:[]});
 fireEvent.click(screen.getByRole('button',{name:/发动触发能力/}));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
 expect(submit).toHaveBeenLastCalledWith({...action,choiceId:v.pendingChoice.id,selected:['accept']});
 expect(view('cascade-death-stack',0).responseWindow.canBegin).toBe(true);
 expect(view('cascade-response-composing',0).responseWindow.myIntentId).toBe('jz55-cascade-response');
 expect(view('cascade-response-top',0).stack.map(s=>s.cardId)).toEqual(['JZ31','JC063']);
});

it('saves and restores a fifty-card JZ55 draft and rejects four copies with the actual ABI',()=>{
 const draft={...createDeckDraft(catalog),name:'黑色传奇落幕',cards:[{cardId:'JZ55',count:3},{cardId:'JC125',count:47}]};
 expect(validateDeckDraft(draft,catalog).valid).toBe(true);saveDeckLibrary([draft]);expect(readDeckLibrary().drafts[0]).toEqual(draft);
 expect(JSON.parse(k.newGameWithDeck('jz55-ui-deck','LOCAL','teams','P0',JSON.stringify(draft),'1')).view.status).toBe('lobby');
 const invalid={...draft,cards:[{cardId:'JZ55',count:4},{cardId:'JC125',count:46}]};
 expect(validateDeckDraft(invalid,catalog).valid).toBe(false);expect(()=>k.newGameWithDeck('jz55-ui-invalid','LOCAL','teams','P0',JSON.stringify(invalid),'1')).toThrow();
});
