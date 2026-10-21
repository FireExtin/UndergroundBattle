import { loadCurrentFixture } from '../../../tools/current-native-source.mjs';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act,fireEvent,render,screen } from '@testing-library/react';
import { expect,it,vi } from 'vitest';
import { kernel as abi, moduleBytes as currentBytes } from '../../../rust-game-wasm/tests/current-kernel.mjs';
import { actionForRoom } from './api';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
import { groupObjectActions } from './objectActions';
const wasm=currentBytes;
abi.initSync({module:wasm});
const fixture = loadCurrentFixture('rust-game-wasm/tests/current93/xq25.json.gz');
const catalog=JSON.parse(abi.catalog()),definition=catalog.cards.find(c=>c.id==='XQ25');
it('reads the complete pinned XQ25 face with original temporary investigation and no invented tapping ability',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.93-batch20-next');
  expect(definition).toMatchObject({name:'监控探头',kind:'attachment',color:'灰',cost:2,loyalty:['灰色','灰色'],unique:false,
    subtypes:['设施'],magic:'',keywords:[],permanentIcons:{investigation:0,combat:0,influence:0},temporaryIcons:{investigation:1,combat:0,influence:0}});
  expect(definition.defense).toBeNull();expect(definition.abilities).toMatchObject([{key:'xq25-attach-region',timing:'standard',triggered:false}]);
  expect(definition.abilities).toHaveLength(1);
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/XQ25 监控探头.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('c755f3d8ac0b85c3ab619b01626b92ee316f73885a5dd5a8102d53f3691306a0');
  expect(readFileSync(resolve('public/cards/XQ25.jpg')).equals(original)).toBe(true);expect(cardScanUrl('XQ25')).toBe('/cards/XQ25.jpg');
  render(<ReadModal card={{...definition,cardId:'XQ25',instanceId:'read-xq25',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent('未横置');expect(screen.getByRole('dialog')).toHaveTextContent('额外增加1');
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'监控探头原始牌面'})).toHaveAttribute('src','/cards/XQ25.jpg');
});
it.each(fixture.cases)('$name chooses the current priced region action and submits the exact Native Room command',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn(),first=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);first.unmount();
  const ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  const legal=view.legalActions.find(a=>a.kind===wanted.kind&&a.cardId===wanted.cardId&&a.region===wanted.region);
  expect(legal).toBeDefined();
  fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
  const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
  if(row.name==='conceal-surcharge') expect(group.label).toContain('支付2');
  if(row.name==='conceal-stacked') expect(group.label).toContain('支付3');
  if(row.name==='conceal-tapped') expect(group.label).not.toContain('费用');
  fireEvent.click(screen.getByRole('button',{name:group.label}));
  const region=view.regions.find(r=>r.index===wanted.region);
  fireEvent.click(screen.getByRole('button',{name:`查看地区${wanted.region+1} ${region.name}`}));
  const confirm=ui.container.querySelector(`[data-action-kind="${wanted.kind}"][data-action-region="${wanted.region}"]`);
  expect(confirm).toHaveAttribute('data-action-id',legal.id);expect(confirm).toBeEnabled();fireEvent.click(confirm);
  expect(submit).toHaveBeenCalledOnce();const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);row.afterViews.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
  ui.rerender(<Table catalog={catalog} view={row.afterViews[row.seat]} busy={false} onAction={submit}/>);
});
it.each(fixture.cases)('$name retains all-seat projection and makes no automatic action after one minute',row=>{
  vi.useFakeTimers();try {row.beforeViews.forEach((view,seat)=>{
    expect(JSON.parse(abi.view(row.before,seat))).toEqual(view);
    const submit=vi.fn(),ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(screen.queryByRole('dialog',{name:'待完成的选择'})).toBeNull();
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
