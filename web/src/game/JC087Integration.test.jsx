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
const fixture=JSON.parse(readFileSync(resolve('src/game/jc087Native69.fixture.json'),'utf8'));
const catalog=JSON.parse(abi.catalog());
const definition=catalog.cards.find(c=>c.id==='JC087');

it('exposes the original JC087 card face and complete conditional self-hide/influence/retreat definition',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.69-wm023-unique-hand-entry-candidate');
  expect(definition).toMatchObject({name:'格鲁尼的走私客',color:'黑',cost:4,loyalty:['黑色','黑色'],defense:2,
    subtypes:['人类','罪犯'],permanentIcons:{investigation:0,combat:1,influence:1},temporaryIcons:{investigation:1,combat:1,influence:0},keywords:['撤回']});
  expect(definition.abilities).toMatchObject([{key:'jc087-enter-region-hide-influence',costs:[],triggered:true}]);
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC087 格鲁尼的走私客.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('cbd39806bd8126021edff506a65dad651485c6661f181d6e8d8ff9d16911ce00');
  expect(readFileSync(resolve('public/cards/JC087.jpg'))).toEqual(original);
  expect(cardScanUrl('JC087')).toBe('/cards/JC087.jpg');
  render(<ReadModal card={{...definition,cardId:'JC087',instanceId:'read-jc087',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'格鲁尼的走私客原始牌面'})).toHaveAttribute('src','/cards/JC087.jpg');
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
