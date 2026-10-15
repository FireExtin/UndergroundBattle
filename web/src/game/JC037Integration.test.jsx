// Fresh source-produced ABI75 inputs only; old opaque states remain unchanged.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {act,fireEvent,render,screen,within} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import {Table} from './Table';
import {actionForRoom} from './api';
import {groupObjectActions} from './objectActions';
const bytes=readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));abi.initSync({module:bytes});
const fixture=JSON.parse(readFileSync(resolve('src/game/jc037Native75.fixture.json'))),catalog=JSON.parse(abi.catalog());
it('binds the complete JC037 face and actual source-built ABI75',()=>{
 expect(catalog).toEqual(fixture.catalog);expect(catalog.engineVersion).toBe('rust-v0.2.75-jc060-occult-adviser-candidate');
 expect(catalog.cardPoolVersion).toBe('limited-v2.68-jc060-occult-adviser-candidate');expect(createHash('sha256').update(bytes).digest('hex')).toBe(fixture.wasmSha256);
 expect(catalog.cards.find(c=>c.id==='JC037')).toMatchObject({color:'蓝',subtitle:'寂静女士',keywords:['领袖'],permanentIcons:{investigation:0,combat:2,influence:2},abilities:[]});
});
it('rejects preserved Native73 states through both current ABI entrypoints',()=>{
 const old=JSON.parse(readFileSync(resolve('src/game/jc052Native73Current.fixture.json'))).cases[0];
 expect(()=>abi.view(old.before,0)).toThrow(/旧局schema\/固定版本/);
 expect(()=>abi.applyRoom(old.before,old.seat,JSON.stringify(old.command),'0')).toThrow(/旧局schema\/固定版本/);
});
const deploy=fixture.scenarios.find(s=>s.name==='paid-deploy').steps[0];
const leader=fixture.scenarios.find(s=>s.name==='leader-replace').steps.find(s=>s.command.action.action?.kind==='choose'&&!s.expected.errorCode);
it.each([{name:'paid deployment',row:deploy},{name:'mandatory different Leader sacrifice',row:leader}])('$name sends the exact complete Native action after restoring the table',({row})=>{
 const view=JSON.parse(abi.view(row.state,row.seat)),submit=vi.fn();let ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);ui.unmount();
 ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.state,row.seat))} busy={false} onAction={submit}/>);
 const wanted=row.command.action.action;
 if(wanted.kind==='choose') {
  const dialog=screen.getByRole('dialog',{name:'待完成的选择'});expect(within(dialog).getByRole('button',{name:'确认选择'})).toBeDisabled();
  wanted.selected.forEach(id=>fireEvent.click(Array.from(dialog.querySelectorAll('[data-choice-option]')).find(el=>el.getAttribute('data-choice-option')===id)));
  fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));
 } else {
  const legal=view.legalActions.find(a=>a.kind===wanted.kind&&a.cardId===wanted.cardId&&a.region===wanted.region);expect(legal).toBeDefined();
  fireEvent.click(Array.from(ui.container.querySelectorAll('[data-card-instance]')).find(el=>el.getAttribute('data-card-instance')===wanted.cardId));
  const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
  fireEvent.click(screen.getByRole('button',{name:group.label}));const region=view.regions.find(r=>r.index===wanted.region);
  fireEvent.click(screen.getByRole('button',{name:`查看地区${wanted.region+1} ${region.name}`}));
  fireEvent.click(Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id));
 }
 expect(submit).toHaveBeenCalledOnce();const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
 const actual=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));expect(actual).toEqual(row.expected);
 row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(actual.state,seat))).toEqual(v));
});
it.each(fixture.scenarios)('$name keeps every seat projection, public hidden count and current icons after restore',scenario=>{
 const last=scenario.steps.at(-1);
 last.views.forEach((v,seat)=>{
  expect(JSON.parse(abi.view(last.expected.state,seat))).toEqual(v);
  const ui=render(<Table catalog={catalog} view={v} busy={false} onAction={vi.fn()}/>);
  for(const region of v.regions)for(const c of region.characters) {
   if(c.cardId==='JC037'&&!c.faceDown) {
    const tile=Array.from(ui.container.querySelectorAll('[data-card-instance]')).find(el=>el.getAttribute('data-card-instance')===c.instanceId);
    expect(within(tile).getByLabelText(`图标：调查${c.icons.investigation}，战斗${c.icons.combat}，势力${c.icons.influence}`)).toBeInTheDocument();
    fireEvent.click(tile);
    expect(screen.getByLabelText(`当前有效图标：调查${c.icons.investigation}，战斗${c.icons.combat}，势力${c.icons.influence}`)).toBeInTheDocument();
    expect(screen.getByLabelText('印刷图标：调查0，战斗2，势力2')).toBeInTheDocument();
   }
   if(c.faceDown&&c.controller!==v.you){expect(c.cardId).toBeUndefined();expect(c.text).toBeUndefined();}
  }
  ui.unmount();
 });
});
it('mandatory Leader choice is private to its controller and automatic pass never chooses',()=>{
 const before=JSON.parse(abi.view(leader.state,0));vi.useFakeTimers();
 try{for(let seat=0;seat<4;seat++){
  const view=JSON.parse(abi.view(leader.state,seat)),submit=vi.fn(),ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
  expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(seat===leader.seat);
  if(seat===leader.seat)expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();
  act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
 }}finally{vi.useRealTimers();}
 expect(before.pendingChoice.min).toBe(1);expect(before.pendingChoice.allowDecline).toBe(false);
});
