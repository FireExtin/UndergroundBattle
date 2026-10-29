import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,fireEvent} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';
import {validateDeckDraft} from './deckLibrary';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/mswm01.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('keeps the blue family closed and validates only printed off-color Vampire/Specter characters',()=>{
 const draft=fixture.cases[0].construction.deck;
 expect(catalog.societies.find(s=>s.id==='MSWM01')).toMatchObject({color:'蓝',supported:false,startingHand:6});
 expect(validateDeckDraft(draft,catalog).valid).toBe(false);
 const candidate={...catalog,societies:catalog.societies.map(s=>s.id==='MSWM01'?{...s,supported:true}:s)};
 expect(validateDeckDraft(draft,candidate).valid).toBe(true);
 for(const card of catalog.cards.filter(c=>c.supported&&c.kind!=='region')){
  const probe={...draft,cards:[{cardId:'JC125',count:49},{cardId:card.id,count:1}]};
  const allowed=['蓝','黄','中立'].includes(card.color)||(card.kind==='character'&&card.subtypes.some(s=>['吸血鬼','阴魂'].includes(s)));
  expect(validateDeckDraft(probe,candidate).issues.some(i=>i.code==='society-vampire-specter'),card.id).toBe(!allowed);
 }
});
const steps=fixture.cases[0].steps;
it.each(steps.filter(s=>s.command.action.action?.abilityId?.startsWith('mswm01-')).map(step=>({name:step.command.action.action.abilityId,step})))('$name sends its original paid action and survives a restored view',({step})=>{
 const view=JSON.parse(abi.view(step.state,step.seat)),wanted=step.command.action.action,submit=vi.fn();
 const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.abilityId===wanted.abilityId&&JSON.stringify(a.costSelected)===JSON.stringify(wanted.costSelected));expect(legal).toBeDefined();
 fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
 const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);expect(button).toBeDefined();fireEvent.click(button);
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
 ui.unmount();render(<Table catalog={catalog} view={JSON.parse(abi.view(step.expected.state,step.seat))} busy={false} onAction={vi.fn()}/>);
});
