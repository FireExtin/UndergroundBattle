// Compiler-bound Native85 -> actual WASM -> production Table/choice controls.
import {render,screen,fireEvent,within} from '@testing-library/react';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {loadFixture,sha256} from '../../../tools/jc044-oracle.mjs';
import {kernel as abi,moduleBytes,catalog} from '../../../rust-game-wasm/tests/current85-kernel.mjs';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {actionForRoom} from './api';
const fixture=loadFixture();expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);expect(catalog).toEqual(fixture.catalog);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
const scenarios=fixture.scenarios.filter(s=>s.name.startsWith('actor-')||['target-self','target-partner','physical-attachment-cost'].includes(s.name));
it.each(scenarios)('$name keeps the exact physical time carrier and target through ordinary UI confirmation',scenario=>{
 const row=scenario.steps.find(r=>!r.expected.errorCode&&r.command.action.action?.kind==='activate');
 const view=JSON.parse(abi.view(row.state,row.seat)),expected=row.command.action.action;
 const actions=view.legalActions.filter(a=>a.kind==='activate'&&a.cardId===expected.cardId&&a.targetId===expected.targetId&&JSON.stringify(a.costSelected)===JSON.stringify(expected.costSelected));
 expect(actions).toHaveLength(1);expect(actions[0].label).toContain('移除1个时间标志');expect(actions[0].label).not.toContain('牺牲');
 const submit=vi.fn();const mounted=render(<Table catalog={catalog} view={view} busy={false} connection="online" onAction={submit}/>);
 const tile=mounted.container.querySelector(`[data-card-instance="${expected.cardId}"]`);expect(tile).not.toBeNull();fireEvent.click(tile);
 const group=screen.getAllByRole('button').find(button=>button.hasAttribute('data-object-action')&&button.getAttribute('aria-label')?.includes(`移除1个时间标志`));
 expect(group).toBeTruthy();fireEvent.click(group);
 const player=mounted.container.querySelector(`[data-player-target="${expected.targetId}"]`);expect(player).not.toBeNull();fireEvent.click(player);
 const confirm=within(screen.getByLabelText('确认目标行动')).getAllByRole('button').find(button=>button.dataset.actionId===actions[0].id);
 expect(confirm).toBeTruthy();fireEvent.click(confirm);expect(submit).toHaveBeenCalledOnce();
 const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
 const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));expect(result).toEqual(row.expected);
 row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
 mounted.rerender(<Table catalog={catalog} view={view} busy connection="online" onAction={submit}/>);
 expect(submit).toHaveBeenCalledOnce();
});
const choices=fixture.scenarios.flatMap(s=>s.steps.flatMap((row,i)=>{
 const views=i?s.steps[i-1].views:s.initial.views;
 return !row.expected.errorCode&&row.command.action.action?.kind==='choose'&&views[row.seat].pendingChoice?.title==='遏制1：选择移除本方势力的地区'
  ?[{name:s.name,row,views}]:[];
}));
it.each(choices)('$name restores the actual target-player choice and removes only its selected team influence',({row,views})=>{
 const view=views[row.seat],submit=vi.fn(),props={catalog,busy:false,connection:'online',onAction:submit};
 const first=render(<Table {...props} view={view}/>);first.unmount();
 const restored=JSON.parse(abi.view(row.state,row.seat));expect(restored).toEqual(view);
 const mounted=render(<Table {...props} view={restored}/>);const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 const confirm=within(dialog).getByRole('button',{name:'确认选择'});expect(confirm).toBeDisabled();
 fireEvent.click(dialog.querySelector(`[data-choice-option="${row.command.action.action.selected[0]}"]`));fireEvent.click(confirm);expect(submit).toHaveBeenCalledOnce();
 const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
 const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));expect(result).toEqual(row.expected);
 for(let seat=0;seat<4;seat++)expect(JSON.parse(abi.view(result.state,seat))).toEqual(row.views[seat]);
 mounted.rerender(<Table {...props} view={view} busy/>);fireEvent.click(within(dialog).getByRole('button',{name:'正在提交…'}));expect(submit).toHaveBeenCalledOnce();
});
it('only the selected target receives the repression choice and the entire print remains readable',()=>{
 const {row,views}=choices[0];const outsider=views[(row.seat+1)%4];expect(outsider.pendingChoice).toBeNull();
 const first=render(<Table catalog={catalog} view={outsider} busy={false} connection="online" onAction={vi.fn()}/>);
 expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();first.unmount();
 const definition=catalog.cards.find(c=>c.id==='JC044');expect(definition).toMatchObject({cost:3,color:'红',magic:'死亡',loyalty:['红色'],defense:1,supported:true});
 const card=fixture.scenarios.find(s=>s.name==='actor-0').initial.views[0].regions[0].characters.find(c=>c.cardId==='JC044');
 render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByRole('dialog')).toHaveTextContent('自本方卡牌上移除一个时间标志');fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
 expect(screen.getByRole('img',{name:'灾难使者原始牌面'})).toHaveAttribute('src','/cards/JC044.jpg');
});
