import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,within} from '@testing-library/react';
import {kernel as abi,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {groupObjectActions} from './objectActions';
import {actionForRoom} from './api';
import {validateDeckDraft} from './deckLibrary';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/bq045.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it('retains closed public admission and the actual private 50+1 natural input',()=>{
 const scenario=fixture.cases.find(c=>c.construction);expect(scenario.construction).toMatchObject({ordinaryCards:50,separateCore:1,minimumCommandCards:12,privateRegistryOnly:true,noLayoutOrResourceMutation:true});
 expect(catalog.cards.find(c=>c.id==='BQ045')).toMatchObject({supported:false,cost:3,loyalty:['灰色'],subtitle:'修格斯'});
 expect(validateDeckDraft(scenario.construction.deck,catalog).valid).toBe(false);
});
it('submits the real hunt with one visible original tentacle payment',()=>{
 const step=fixture.cases.find(c=>c.construction).steps.find(s=>s.command.action.action?.abilityId==='bq045-locked-hunt');expect(step).toBeDefined();
 const wanted=step.command.action.action,view=JSON.parse(abi.view(step.state,step.seat)),submit=vi.fn();expect(wanted.costSelected).toHaveLength(1);
 const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
 const legal=view.legalActions.find(a=>a.abilityId===wanted.abilityId&&a.targetId===wanted.targetId&&JSON.stringify(a.costSelected)===JSON.stringify(wanted.costSelected));expect(legal).toBeDefined();
 const card=ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`);expect(card).not.toBeNull();fireEvent.click(card);
 const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));expect(group).toBeDefined();fireEvent.click(screen.getByRole('button',{name:group.label,exact:true}));
 fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.targetId}"]`));
 const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);expect(button).toBeDefined();fireEvent.click(button);
 expect(submit).toHaveBeenCalledOnce();expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
 expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
it('keeps each invoked core target and repression choice distinct in the real UI',()=>{
 const scenario=fixture.cases.find(c=>c.name==='bq045-prepared-BQ063-core');let targets=0,opponents=0,repressions=0;
 for(const step of scenario.steps){if(step.command.action.action?.kind!=='choose')continue;const view=JSON.parse(abi.view(step.state,step.seat));if(!view.pendingChoice?.kind.startsWith('bq045-core-'))continue;
  const submit=vi.fn();render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  for(const id of step.command.action.action.selected){fireEvent.click(Array.from(dialog.querySelectorAll('[data-choice-option]')).find(el=>el.getAttribute('data-choice-option')===id));}
  fireEvent.click(within(dialog).getByRole('button',{name:/确认/}));expect(actionForRoom(view,submit.mock.calls[0][0])).toEqual(step.command.action);
  if(view.pendingChoice.kind==='bq045-core-target')targets++;if(view.pendingChoice.kind==='bq045-core-opponent')opponents++;if(view.pendingChoice.kind==='bq045-core-repress')repressions++;cleanup();
 }
 expect([targets,opponents,repressions]).toEqual([2,2,2]);
});
