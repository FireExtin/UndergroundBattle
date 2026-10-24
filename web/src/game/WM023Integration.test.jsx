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
import { groupObjectActions } from './objectActions';
const wasm=currentBytes;
abi.initSync({module:wasm});
const fixture = loadCurrentFixture('rust-game-wasm/tests/current112/wm023.json.gz');
const catalog=JSON.parse(abi.catalog()),definition=catalog.cards.find(c=>c.id==='WM023');
it('reads the complete pinned public/slow unique WM023 face and paid exhaustion ability',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.112-shared-keyword-rewards');
  expect(definition).toMatchObject({name:'尽职的珍珠',subtitle:'庄园仆役',color:'蓝',cost:4,loyalty:['蓝色'],unique:true,defense:1,
    subtypes:['人类','奴仆'],magic:'',keywords:['公开','迟缓'],permanentIcons:{investigation:0,combat:0,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0}});
  expect(definition.abilities).toMatchObject([{key:'wm023-unique-hand-entry',timing:'standard',triggered:false}]);
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/WM023 尽职的珍珠.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('b209fe484e37b930ce4d64be078bd3e72a10bdff5250bfd38ba97cb7648cee85');
  expect(readFileSync(resolve('public/cards/WM023.jpg')).equals(original)).toBe(true);expect(cardScanUrl('WM023')).toBe('/cards/WM023.jpg');
  render(<ReadModal card={{...definition,cardId:'WM023',instanceId:'read-wm023',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'尽职的珍珠原始牌面'})).toHaveAttribute('src','/cards/WM023.jpg');
});
it.each(fixture.cases)('$name submits the exact Native command after projection restore',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn(),first=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);first.unmount();
  const ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  if(wanted.kind==='activate') {
    const legal=view.legalActions.find(a=>a.kind==='activate'&&a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId&&a.region===wanted.region);
    fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
    const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
    fireEvent.click(screen.getByRole('button',{name:group.label}));
    const region=view.regions.find(r=>r.index===wanted.region);
    fireEvent.click(screen.getByRole('button',{name:`查看地区${wanted.region+1} ${region.name}`}));
    const confirm=ui.container.querySelector(`[data-action-kind="activate"][data-action-region="${wanted.region}"]`);
    expect(confirm).toHaveAttribute('data-action-id',legal.id);expect(confirm).toBeEnabled();fireEvent.click(confirm);
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
it.each(fixture.cases)('$name exposes hand choices only to the actor and never auto-selects',row=>{
  vi.useFakeTimers();try {row.beforeViews.forEach((view,seat)=>{
    const submit=vi.fn(),ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(row.command.action.action.kind==='choose'&&seat===row.seat);
    if(seat!==row.seat) expect(ui.container.querySelector('[data-choice-option]')).toBeNull();
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
