// Actual Native93/WASM views through production Table/choice/read controls.
import {render,screen,fireEvent,within} from '@testing-library/react';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {loadFixture,sha256} from '../../../tools/jc041-oracle.mjs';
import {kernel as abi,moduleBytes,catalog} from '../../../rust-game-wasm/tests/current-kernel.mjs';
import {Table} from './Table';
import {ReadModal} from './ReadModal';
import {actionForRoom} from './api';
const fixture=loadFixture();expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);expect(catalog).toEqual(fixture.catalog);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
const rows=fixture.scenarios.flatMap(s=>s.steps.map((row,i)=>({name:s.name,row,views:i?s.steps[i-1].views:s.initial.views})));
const choices=rows.filter(({row,views})=>!row.expected.errorCode&&row.command.action.action?.kind==='choose'&&views[row.seat].pendingChoice);
it.each(choices)('$name preserves its private compulsory discard draft across remount and applies the exact native transition',({row,views})=>{
 const view=views[row.seat],selected=row.command.action.action.selected,submit=vi.fn();
 const props={catalog,busy:false,connection:'online',onAction:submit};
 const first=render(<Table {...props} view={view}/>);let dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 expect(dialog).toHaveTextContent('禁婆：弃 1 张手牌');
 expect(dialog.querySelectorAll('[data-choice-option]')).toHaveLength(view.pendingChoice.options.length);
 expect(within(dialog).getByRole('button',{name:'确认选择'})).toBeDisabled();
 fireEvent.click(dialog.querySelector(`[data-choice-option="${selected[0]}"]`));first.unmount();
 const restored=JSON.parse(abi.view(row.state,row.seat));expect(restored).toEqual(view);
 const mounted=render(<Table {...props} view={restored}/>);dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 expect(dialog.querySelector(`[data-choice-option="${selected[0]}"]`)).toHaveAttribute('aria-pressed','true');
 fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenCalledOnce();
 const action=JSON.parse(JSON.stringify(actionForRoom(restored,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
 const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));expect(result).toEqual(row.expected);
 row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
 mounted.rerender(<Table {...props} view={restored} busy/>);fireEvent.click(within(dialog).getByRole('button',{name:'正在提交…'}));expect(submit).toHaveBeenCalledOnce();
});
const activations=rows.filter(({row})=>!row.expected.errorCode&&row.command.action.action?.kind==='activate'&&row.command.action.action.abilityId==='jc041-water-hidden-reveal');
it.each(activations)('$name exposes the real water-only hidden action and keeps it separate from paid reveal',({row,views})=>{
 const view=views[row.seat],core=row.command.action.action,submit=vi.fn();
 const props={catalog,busy:false,connection:'online',onAction:submit};const mounted=render(<Table {...props} view={view}/>);
 const card=mounted.container.querySelector(`[data-card-instance="${core.cardId}"]`);expect(card).not.toBeNull();fireEvent.click(card);
 const legal=view.legalActions.find(a=>a.kind==='activate'&&a.cardId===core.cardId&&a.abilityId===core.abilityId);expect(legal).toBeDefined();
 fireEvent.click(screen.getByRole('button',{name:legal.label}));expect(submit).toHaveBeenCalledOnce();
 const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
 expect(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'))).toEqual(row.expected);
 mounted.rerender(<Table {...props} view={view} busy/>);expect(screen.getByRole('button',{name:legal.label})).toBeDisabled();
});
it('only the hidden source controller sees its print and action; other seats receive neither',()=>{
 const s=fixture.scenarios.find(s=>s.name==='water-actor-0'),row=s.steps[0],id=row.command.action.action.cardId;
 for(let seat=1;seat<4;seat++){
  const view=s.initial.views[seat],card=view.regions.flatMap(r=>r.characters).find(c=>c.instanceId===id);
  expect(card.cardId).toBeUndefined();expect(card.name).toBe('暗藏者');expect(card.text).toBeUndefined();
  expect(view.legalActions.some(a=>a.cardId===id&&a.abilityId==='jc041-water-hidden-reveal')).toBe(false);
  const mounted=render(<Table catalog={catalog} view={view} busy={false} connection="online" onAction={vi.fn()}/>);
  expect(mounted.container.querySelector(`[data-card-instance="${id}"]`)).toHaveAccessibleName('查看暗藏者');mounted.unmount();
 }
});
it('only each deciding player gets discard controls and its own hand options',()=>{
 const {row,views}=choices[0];
 for(let seat=0;seat<4;seat++)if(seat!==row.seat){
  expect(views[seat].pendingChoice).toBeNull();
  const mounted=render(<Table catalog={catalog} view={views[seat]} busy={false} connection="online" onAction={vi.fn()}/>);
  expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();mounted.unmount();
 }
});
it('reads the complete original blue card and its water/compulsory text',()=>{
 const s=fixture.scenarios.find(s=>s.name==='water-actor-0'),view=s.steps.at(-1).views[0];
 const card=view.regions.flatMap(r=>r.characters).find(c=>c.cardId==='JC041'),definition=catalog.cards.find(c=>c.id==='JC041');
 expect(definition).toMatchObject({name:'禁婆',cost:4,color:'蓝',loyalty:[],magicIcon:'Death',subtypes:['不死生物','尸鬼'],defense:1,unique:false,permanentIcons:{investigation:0,combat:1,influence:0},temporaryIcons:{investigation:0,combat:1,influence:1}});
 render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);const dialog=screen.getByRole('dialog');
 expect(dialog).toHaveTextContent('水域地区');expect(dialog).toHaveTextContent('每位玩家各弃一张手牌');
 fireEvent.click(within(dialog).getByRole('button',{name:'原始牌面'}));
 expect(within(dialog).getByRole('img',{name:'禁婆原始牌面'})).toHaveAttribute('src','/cards/JC041.jpg');
});
