import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {groupObjectActions} from './objectActions';
import {actionForRoom} from './api';
import {validateDeckDraft} from './deckLibrary';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/msjc12.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('keeps production admission closed and uses printed vampire/slave construction in the private test registry',()=>{
 const natural=fixture.cases.find(c=>c.construction),draft=natural.construction.deck;
 expect(validateDeckDraft(draft,catalog).valid).toBe(false);
 const candidate={...catalog,societies:catalog.societies.map(s=>s.id==='MSJC12'?{...s,supported:true}:s)};
 expect(validateDeckDraft(draft,candidate).valid).toBe(true);
 for(const card of catalog.cards.filter(c=>c.supported&&c.kind!=='region')){
  const probe={...draft,cards:[{cardId:'JC125',count:49},{cardId:card.id,count:1}]};
  const allowed=['蓝','中立'].includes(card.color)||(card.kind==='character'&&card.subtypes.some(s=>['吸血鬼','奴仆'].includes(s)));
  expect(validateDeckDraft(probe,candidate).issues.some(i=>i.code==='society-vampire-slave')).toBe(!allowed);
 }
});
const activations=fixture.cases.flatMap(c=>c.steps.filter(s=>s.command.action.action?.abilityId?.startsWith('msjc12-')).map(step=>({name:c.name,step})));
it.each(activations)('$name submits the exact three paid characters through the actual society action',({step})=>{
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.abilityId===wanted.abilityId&&JSON.stringify(a.costSelected)===JSON.stringify(wanted.costSelected)&&a.targetId===wanted.targetId);expect(legal).toBeDefined();expect(legal.costSelected).toHaveLength(3);
 const source=ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`);expect(source).toBeInTheDocument();fireEvent.click(source);
 if(wanted.targetId){const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));fireEvent.click(screen.getByRole('button',{name:group.label}));fireEvent.click(ui.container.querySelector(`[data-player-target="${wanted.targetId}"]`));}
 const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);expect(button).toBeDefined();fireEvent.click(button);
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
it('the enemy chooses their own actual hand card after the vampire payment',()=>{
 const step=fixture.cases.find(c=>c.name.includes('vampire')).steps.find(s=>s.command.action.action?.kind==='choose');
 const view=JSON.parse(abi.view(step.state,step.seat)),submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const dialog=screen.getByRole('dialog',{name:'待完成的选择'}),id=step.command.action.action.selected[0];fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`));fireEvent.click(within(dialog).getByRole('button',{name:/确认/}));
 expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
