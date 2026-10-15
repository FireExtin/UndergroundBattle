// Requires fresh Native75 inputs and its exact production ABI; no older fallback.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {act,fireEvent,render,screen,within} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import {Table} from './Table';
import {actionForRoom} from './api';
import {groupObjectActions} from './objectActions';
const wasm=readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
abi.initSync({module:wasm});
const catalog=JSON.parse(abi.catalog());
const fixture=JSON.parse(readFileSync(resolve('src/game/jc052Native75Current.fixture.json'),'utf8'));
it('uses the actual JC052 ABI73 and complete, untargeted optional entry definition',()=>{
  expect(catalog.engineVersion).toBe('rust-v0.2.75-jc060-occult-adviser-candidate');
  expect(catalog.cardPoolVersion).toBe('limited-v2.68-jc060-occult-adviser-candidate');
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.cards.find(c=>c.id==='JC052')).toMatchObject({color:'红',subtitle:'剪断命运者',keywords:['领袖'],abilities:[{key:'jc052-destroy-other-original-region-characters',triggered:true,timing:'fast',costs:[]}]});
});
it('rejects the preserved Native72 room through both current ABI read and action paths',()=>{
  const previous=JSON.parse(readFileSync(resolve('src/game/jz35Native72.fixture.json'),'utf8')).cases[0];
  expect(()=>abi.view(previous.before,previous.seat)).toThrow(/旧局schema\/固定版本/);
  expect(()=>abi.applyRoom(previous.before,previous.seat,JSON.stringify(previous.command),'0')).toThrow(/旧局schema\/固定版本/);
});
it.each(fixture.cases)('$name restores the real choice or paid command and sends its complete Native payload',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn();let ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);ui.unmount();
  ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  if(wanted.kind==='choose') {
    const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
    wanted.selected.forEach(id=>fireEvent.click(Array.from(dialog.querySelectorAll('[data-choice-option]')).find(el=>el.getAttribute('data-choice-option')===id)));
    const confirm=within(dialog).getByRole('button',{name:'确认选择'});
    expect(confirm).toBeEnabled();fireEvent.click(confirm);
  } else {
    const legal=view.legalActions.find(a=>a.kind===wanted.kind&&a.cardId===wanted.cardId&&a.region===wanted.region);
    expect(legal).toBeDefined();expect(legal.targetId).toBeUndefined();
    const source=Array.from(ui.container.querySelectorAll('[data-card-instance]')).find(el=>el.getAttribute('data-card-instance')===wanted.cardId);
    expect(source).toBeInTheDocument();fireEvent.click(source);
    const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
    fireEvent.click(screen.getByRole('button',{name:group.label}));
    const region=view.regions.find(r=>r.index===wanted.region);
    fireEvent.click(screen.getByRole('button',{name:`查看地区${wanted.region+1} ${region.name}`}));
    const button=Array.from(ui.container.querySelectorAll('[data-action-id]')).find(el=>el.getAttribute('data-action-id')===legal.id);
    expect(button).toBeEnabled();fireEvent.click(button);
  }
  expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));expect(result).toEqual(row.expected);
  row.afterViews.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
});
it.each(fixture.cases)('$name keeps forced sacrifices and optional triggers with their controller after restore',row=>{
  vi.useFakeTimers();try{row.beforeViews.forEach((view,seat)=>{
    expect(JSON.parse(abi.view(row.before,seat))).toEqual(view);
    const submit=vi.fn(),ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(row.command.action.action.kind==='choose'&&seat===row.seat);
    if(seat!==row.seat) expect(ui.container.querySelector('[data-choice-option]')).toBeNull();
    if(row.name==='leader-sacrifice'&&seat===row.seat) expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
