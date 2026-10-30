import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,fireEvent,screen} from '@testing-library/react';
import {kernel,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {loadCurrentFixture} from '../../../tools/current-native-source.mjs';
import {Table} from './Table';
import {actionForRoom} from './api';
const fixture=loadCurrentFixture('rust-game-wasm/tests/current113/bq095.json.gz');
afterEach(()=>{cleanup();sessionStorage.clear();});
it.each([0,1,2,3])('seat%s sees headquarters as a distinct public zone with original card kinds',seat=>{
 const c=fixture.cases[0],view=JSON.parse(kernel.view(c.steps.at(-1).expected.state,seat)),send=vi.fn();
 expect(catalog.cards.find(c=>c.id==='BQ095').supported).toBe(false);expect(view.graveyard).toHaveLength(0);expect(view.headquarters).toHaveLength(1);expect(view.headquarters[0].cards).toHaveLength(1);expect(view.headquarters[0].cards[0].kind).toBe('character');
 const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={send}/>),hq=ui.container.querySelector('[data-pile-owner="p0"][data-pile-kind="headquarters"]'),grave=ui.container.querySelector('[data-pile-owner="p0"][data-pile-kind="graveyard"]');
 expect(hq).toBeTruthy();expect(hq.textContent).toContain('集团总部');expect(hq.textContent).toContain('无知路人');expect(grave.textContent).not.toContain('无知路人');fireEvent.click(hq.querySelector('summary'));expect(send).not.toHaveBeenCalled();
});
it.each([['prepared','jc035-preparation-order'],['prepared','discard'],['natural','jc035-preparation-order'],['natural','discard']])('%s %s submits actual Native selection after refresh',(origin,kind)=>{
 const scenario=origin==='natural'?fixture.cases.find(c=>c.construction):fixture.cases[1];expect(scenario).toBeTruthy();const step=scenario.steps.find(s=>JSON.parse(kernel.view(s.state,s.seat)).pendingChoice?.kind===kind&&(origin==='prepared'||JSON.parse(kernel.view(s.state,s.seat)).headquarters?.length)&&s.command.action.action?.selected?.length);expect(step).toBeTruthy();const view=JSON.parse(kernel.view(step.state,step.seat)),send=vi.fn();let ui=render(<Table catalog={catalog} view={view} busy={false} onAction={send}/>);
 for(const id of step.command.action.action.selected)fireEvent.click(ui.container.querySelector(`[data-choice-option="${id}"]`));ui.unmount();render(<Table catalog={catalog} view={JSON.parse(kernel.view(step.state,step.seat))} busy={false} onAction={send}/>);fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(send).toHaveBeenCalledOnce();expect(actionForRoom(view,send.mock.calls[0][0])).toEqual(step.command.action);expect(JSON.parse(kernel.applyRoom(step.state,step.seat,JSON.stringify(step.command),'0'))).toEqual(step.expected);
});
