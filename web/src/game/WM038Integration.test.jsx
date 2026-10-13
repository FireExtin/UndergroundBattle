import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act,fireEvent,render,screen,within } from '@testing-library/react';
import { expect,it,vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
import { groupObjectActions } from './objectActions';
const wasm=readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
abi.initSync({module:wasm});
const fixture=JSON.parse(readFileSync(resolve('src/game/wm038Native73.fixture.json'),'utf8'));
const catalog=JSON.parse(abi.catalog()),definition=catalog.cards.find(c=>c.id==='WM038');
it('reads all four rules of the complete pinned WM038 face and its Blood identity',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.73-jc052-original-region-candidate');
  expect(definition).toMatchObject({name:'信徒烙印',kind:'attachment',color:'红',cost:1,loyalty:['红色'],unique:false,subtypes:['纹身'],magic:'鲜血',keywords:[],permanentIcons:{investigation:0,combat:0,influence:0},temporaryIcons:{investigation:0,combat:0,influence:0}});
  expect(definition.defense).toBeNull();expect(definition.abilities).toHaveLength(3);
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/WM038 信徒烙印.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('847dabd861d6350ae21df04ac548a975e99a7cfd4814f213444409d4c42d7acd');
  expect(readFileSync(resolve('public/cards/WM038.jpg'))).toEqual(original);expect(cardScanUrl('WM038')).toBe('/cards/WM038.jpg');
  render(<ReadModal card={{...definition,cardId:'WM038',instanceId:'read-wm038',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  expect(screen.getByRole('dialog')).toHaveTextContent('邪教徒');expect(screen.getByRole('dialog')).toHaveTextContent('血脉领域');expect(screen.getByRole('dialog')).toHaveTextContent('时间标志');
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'信徒烙印原始牌面'})).toHaveAttribute('src','/cards/WM038.jpg');
});
it.each(fixture.cases)('$name submits the exact paid or optional Native Room command after projection restore',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn(),first=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);first.unmount();
  const ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  if(wanted.kind==='choose') {
    const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
    wanted.selected.forEach(id=>fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`)));
    fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));
  } else {
    const legal=view.legalActions.find(a=>a.kind===wanted.kind&&a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId&&a.targetId===wanted.targetId&&a.region===wanted.region&&JSON.stringify(a.costSelected)===JSON.stringify(wanted.costSelected));
    expect(legal).toBeDefined();
    if(row.name==='discounted-blood-deploy') expect(view.hand.find(c=>c.instanceId===wanted.cardId).effectiveCost).toBe(0);
    const source=ui.container.querySelector(`[data-card-instance="${wanted.cardId}"], [data-attachment-instance="${wanted.cardId}"]`);
    expect(source).toBeInTheDocument();fireEvent.click(source);
    const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
    if(wanted.kind==='activate') expect(group.label).toContain('费用：牺牲');
    fireEvent.click(screen.getByRole('button',{name:group.label}));
    if(wanted.targetId) {
      fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.targetId}"]`));
      fireEvent.click(screen.getByRole('button',{name:`确认 · ${legal.label}`}));
    } else if(wanted.region!==undefined) {
      const region=view.regions.find(r=>r.index===wanted.region);
      fireEvent.click(screen.getByRole('button',{name:`查看地区${wanted.region+1} ${region.name}`}));
      const confirm=ui.container.querySelector(`[data-action-kind="${wanted.kind}"][data-action-region="${wanted.region}"]`);
      expect(confirm).toHaveAttribute('data-action-id',legal.id);expect(confirm).toBeEnabled();fireEvent.click(confirm);
    }
  }
  expect(submit).toHaveBeenCalledOnce();const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);row.afterViews.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
  ui.rerender(<Table catalog={catalog} view={row.afterViews[row.seat]} busy={false} onAction={submit}/>);
});
it.each(fixture.cases)('$name retains all-seat privacy and never chooses or activates automatically',row=>{
  vi.useFakeTimers();try {row.beforeViews.forEach((view,seat)=>{
    expect(JSON.parse(abi.view(row.before,seat))).toEqual(view);const submit=vi.fn(),ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(!!screen.queryByRole('dialog',{name:'待完成的选择'})).toBe(row.command.action.action.kind==='choose'&&seat===row.seat);
    if(seat!==row.seat) expect(ui.container.querySelector('[data-choice-option]')).toBeNull();
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useFakeTimers();vi.useRealTimers();}
});
