// Actual new Native141/WASM89 views. These component checks are local only.
import {render,screen,fireEvent,within} from '@testing-library/react';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {loadFixture,sha256} from '../../../tools/jc035-oracle.mjs';
import {kernel,moduleBytes,catalog} from '../../../rust-game-wasm/tests/current89-kernel.mjs';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {actionForRoom} from './api';
const f=loadFixture();expect(sha256(moduleBytes)).toBe(f.wasmSha256);expect(catalog).toEqual(f.catalog);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
const rows=f.scenarios.flatMap(s=>s.steps.map((row,index)=>({name:s.name,row,views:index?s.steps[index-1].views:s.initial.views})));
const choices=rows.filter(({row,views})=>row.command.action.kind==='game'&&row.command.action.action?.kind==='choose'&&views[row.seat].pendingChoice?.kind==='jc035-preparation-order');
expect(choices).toHaveLength(13);
it.each(choices)('$name selects or declines the actual observed source and confirms the exact Native command',({row,views})=>{
 const view=views[row.seat],a=row.command.action.action,onAction=vi.fn();
 const m=render(<Table catalog={catalog} view={view} busy={false} connection='online' onAction={onAction}/>);
 expect(screen.getByText('传媒魔咒：选择下一项准备触发')).toBeInTheDocument();
 expect(screen.getByText('触发 1')).toBeInTheDocument();
 expect(screen.getByRole('button',{name:'确认选择',exact:true})).toBeDisabled();
 for(const id of a.selected||[])fireEvent.click(m.container.querySelector(`[data-choice-option="${id}"]`));
 if(a.selected?.length)expect(screen.getByText(/^下一项入栈：地区/)).toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:a.selected?.length?'确认选择':'跳过此选择',exact:true}));
 expect(onAction).toHaveBeenCalledOnce();
 const command=JSON.parse(JSON.stringify(actionForRoom(view,onAction.mock.calls[0][0])));expect(command).toEqual(row.command.action);
 const result=JSON.parse(kernel.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action:command}),'0'));expect(result).toEqual(row.expected);
 row.views.forEach((v,seat)=>expect(JSON.parse(kernel.view(result.state,seat))).toEqual(v));
 m.rerender(<Table catalog={catalog} view={view} busy connection='online' onAction={onAction}/>);
 expect(screen.getByRole('button',{name:'正在提交…',exact:true})).toBeDisabled();
 expect(screen.getByRole('button',{name:'跳过此选择',exact:true})).toBeDisabled();
 expect(onAction).toHaveBeenCalledOnce();
});
it('keeps a pending source selection after remount and clears it when the next real choice arrives',()=>{
 const entry=choices.find(({name,row})=>name==='same-controller-order'&&row.command.action.action.selected?.length),view=entry.views[entry.row.seat];
 const onAction=vi.fn(),props={catalog,view,busy:false,connection:'online',onAction};
 let m=render(<Table {...props}/>);const id=entry.row.command.action.action.selected[0];
 fireEvent.click(m.container.querySelector(`[data-choice-option="${id}"]`));m.unmount();
 m=render(<Table {...props}/>);expect(m.container.querySelector(`[data-choice-option="${id}"]`)).toHaveAttribute('aria-pressed','true');
 fireEvent.click(screen.getByRole('button',{name:'确认选择',exact:true}));expect(onAction).toHaveBeenCalledOnce();
 m.rerender(<Table {...props} view={entry.row.views[entry.row.seat]}/>);
 expect(m.container.querySelector(`[data-choice-option="${id}"]`)).toBeNull();
 expect(screen.getByRole('button',{name:'确认选择',exact:true})).toBeDisabled();
});
it('does not expose another player preparation controls or the stored enemy-hand observation',()=>{
 const entry=choices.find(({name})=>name==='actor-0');
 for(const[seat,view]of entry.views.entries()){
  expect(JSON.stringify(view)).not.toContain('jc035_preparation');
  if(seat===entry.row.seat)continue;
  const m=render(<Table catalog={catalog} view={view} busy={false} connection='online' onAction={vi.fn()}/>);
  expect(m.container.querySelector('[data-choice-option]')).toBeNull();m.unmount();
 }
});
it('reads every printed field and the pinned original card face for the real region attachment',()=>{
 const s=f.scenarios.find(s=>s.name==='ordinary-attach'),view=s.steps.at(-1).views[0];
 const card=view.attachments.find(a=>a.cardId==='JC035'),definition=catalog.cards.find(c=>c.id==='JC035');
 expect(definition).toMatchObject({name:'传媒魔咒',cost:3,color:'蓝',loyalty:['蓝色'],magicIcon:'Mind',subtypes:['结界','媒体'],unique:false,temporaryIcons:{investigation:1,combat:0,influence:0}});
 render(<ReadModal card={card} definition={definition} viewerId='p0' onClose={vi.fn()}/>);
 const dialog=screen.getByRole('dialog');expect(dialog).toHaveTextContent('准备步骤');expect(dialog).toHaveTextContent('少于或等于三张');
 fireEvent.click(within(dialog).getByRole('button',{name:'原始牌面'}));
 expect(within(dialog).getByRole('img',{name:'传媒魔咒原始牌面'})).toHaveAttribute('src','/cards/JC035.jpg');
});
