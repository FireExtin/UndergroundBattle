import { loadCurrentFixture } from '../../../tools/current-native-source.mjs';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act,fireEvent,render,screen,within } from '@testing-library/react';
import { expect,it,vi } from 'vitest';
import { kernel as abi, moduleBytes as currentBytes } from '../../../rust-game-wasm/tests/current-kernel.mjs';
import { actionForRoom } from './api';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
const wasm=currentBytes;
abi.initSync({module:wasm});
const fixture = loadCurrentFixture('rust-game-wasm/tests/current93/wm056.json.gz');
const catalog=JSON.parse(abi.catalog()),definition=catalog.cards.find(c=>c.id==='WM056');
it('reads the original WM056 and complete once-per-turn paid standard action',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.93-batch20-next');
  expect(definition).toMatchObject({name:'赃物贩子',color:'黑',cost:1,loyalty:['黑色'],defense:1,
    subtypes:['人类','罪犯','商人'],magic:'',unique:false,keywords:[],permanentIcons:{investigation:0,combat:0,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0}});
  expect(definition.abilities).toMatchObject([{key:'wm056-adjacent-influence',timing:'standard',triggered:false}]);
  expect(definition.abilities[0].label).toContain('每回合一次');
  expect(definition.text).toBe('行动②：将相邻地区上的一个本方势力标志移动至本地区。该能力每回合只能发动一次。');
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/WM056 赃物贩子.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('35c67bb88ff9547e63932fd51d1581e77aaf649e1124eef706e4d63a8f20b497');
  expect(readFileSync(resolve('public/cards/WM056.jpg')).equals(original)).toBe(true);expect(cardScanUrl('WM056')).toBe('/cards/WM056.jpg');
  render(<ReadModal card={{...definition,cardId:'WM056',instanceId:'read-wm056',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'赃物贩子原始牌面'})).toHaveAttribute('src','/cards/WM056.jpg');
});
it.each(fixture.cases)('$name restores the projection and submits the exact Native command',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn();const first=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);first.unmount();
  const ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  if(wanted.kind==='activate') {
    const legal=view.legalActions.find(a=>a.kind==='activate'&&a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId);
    expect(legal.targetId).toBeUndefined();
    fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
    fireEvent.click(screen.getByRole('button',{name:legal.label}));
  } else {
    const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
    expect(within(dialog).queryByRole('button',{name:'跳过此选择'})).toBeNull();
    wanted.selected.forEach(id=>fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`)));
    fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));
  }
  expect(submit).toHaveBeenCalledOnce();const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);row.afterViews.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
  ui.rerender(<Table catalog={catalog} view={row.afterViews[row.seat]} busy={false} onAction={submit}/>);
});
it.each(fixture.cases)('$name keeps chooser privacy and never submits automatically',row=>{
  vi.useFakeTimers();try {row.beforeViews.forEach((view,seat)=>{
    const submit=vi.fn(),ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(row.command.action.action.kind==='choose'&&seat===row.seat);
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
