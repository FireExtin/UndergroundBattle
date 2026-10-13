import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
const wasm=readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
abi.initSync({module:wasm});
const fixture=JSON.parse(readFileSync(resolve('src/game/jc061Native72.fixture.json'),'utf8'));
const catalog=JSON.parse(abi.catalog());
const definition=catalog.cards.find(c=>c.id==='JC061');

it('exposes the original JC061 card face and complete paid-entry/mobility definition',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.72-jz35-host-return-candidate');
  expect(definition).toMatchObject({name:'危机处理特派小组',color:'灰',cost:5,loyalty:['灰色','灰色'],defense:2,
    subtypes:['人类','特工'],permanentIcons:{investigation:1,combat:2,influence:1},keywords:['机动']});
  expect(definition.abilities).toMatchObject([{key:'enter-region-paid-hidden-destroy',costs:[{Assets:1}],triggered:true},
    {key:'mobility',triggered:true}]);
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC061 危机处理特派小组.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('132918615db6a2233eed3f2d2d045c368c65348e43c52f101f17e4c4ecd756f6');
  expect(readFileSync(resolve('public/cards/JC061.jpg'))).toEqual(original);
  expect(cardScanUrl('JC061')).toBe('/cards/JC061.jpg');
  render(<ReadModal card={{...definition,cardId:'JC061',instanceId:'read-jc061',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'危机处理特派小组原始牌面'})).toHaveAttribute('src','/cards/JC061.jpg');
});

it.each(fixture.cases)('$name restores the choice and submits the exact current Native command',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn();const first=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
  first.unmount(); // Projection restore only; not browser session/seat persistence.
  const ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  const selected=row.command.action.action.selected;
  if(!selected.length)fireEvent.click(within(dialog).getByRole('button',{name:'跳过此选择'}));
  else {selected.forEach(id=>fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`)));
    fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));}
  expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);
  row.afterViews.forEach((expected,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(expected));
  ui.rerender(<Table catalog={catalog} view={row.afterViews[row.seat]} busy={false} onAction={submit}/>);
});

it.each(fixture.cases)('$name belongs to the chooser and never auto-submits while waiting',row=>{
  vi.useFakeTimers();try {row.beforeViews.forEach((view,seat)=>{
    const submit=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(seat===row.seat);
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
