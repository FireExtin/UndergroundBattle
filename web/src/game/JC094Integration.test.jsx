import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
import { groupObjectActions } from './objectActions';

const wasm=readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
abi.initSync({module:wasm});
const fixture=JSON.parse(readFileSync(resolve('src/game/jc094Native73.fixture.json'),'utf8'));
const catalog=JSON.parse(abi.catalog());
const definition=catalog.cards.find(c=>c.id==='JC094');

it('reads the complete original JC094 face and standard local-other-sacrifice ability',()=>{
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.engineVersion).toBe('rust-v0.2.73-jc052-original-region-candidate');
  expect(definition).toMatchObject({name:'威达的汀巴巴迪',subtitle:'巫毒大师',color:'黑',unique:true,cost:6,
    loyalty:['黑色','黑色','黑色'],magic:'死亡',defense:3,subtypes:['人类','法师','罪犯'],
    permanentIcons:{investigation:0,combat:0,influence:2},temporaryIcons:{investigation:3,combat:0,influence:0},keywords:[]});
  expect(definition.abilities).toMatchObject([{key:'jc094-local-sacrifice-destroy',timing:'standard',triggered:false}]);
  const original=readFileSync(resolve('../resource/ymsj-fun.github.io/cards/JC094 威达的汀巴巴迪.jpg'));
  expect(createHash('sha256').update(original).digest('hex')).toBe('5050b8e0b6d8101d6badcd0aed22f4fa75a2baa61c85e98c17bcd811a5ec72fd');
  expect(readFileSync(resolve('public/cards/JC094.jpg'))).toEqual(original);
  expect(cardScanUrl('JC094')).toBe('/cards/JC094.jpg');
  render(<ReadModal card={{...definition,cardId:'JC094',instanceId:'read-jc094',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'威达的汀巴巴迪原始牌面'})).toHaveAttribute('src','/cards/JC094.jpg');
});

it.each(fixture.cases)('$name submits the exact paid activation after projection restore and matches Native',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn();const first=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);
  first.unmount(); // Projection restore only; no browser seat/session persistence claim.
  const ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  const legal=view.legalActions.find(a=>a.kind===wanted.kind&&a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId&&a.targetId===wanted.targetId&&JSON.stringify(a.costSelected)===JSON.stringify(wanted.costSelected));
  expect(legal).toBeDefined();
  fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`));
  const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===wanted.cardId)).find(g=>g.actions.some(a=>a.id===legal.id));
  expect(group.label).toContain('费用：牺牲');
  fireEvent.click(screen.getByRole('button',{name:group.label}));
  fireEvent.click(ui.container.querySelector(`[data-card-instance="${wanted.targetId}"]`));
  fireEvent.click(screen.getByRole('button',{name:`确认 · ${legal.label}`}));
  expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);
  row.afterViews.forEach((expected,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(expected));
  ui.rerender(<Table catalog={catalog} view={row.afterViews[row.seat]} busy={false} onAction={submit}/>);
});

it.each(fixture.cases)('$name never auto-activates on any seat',row=>{
  vi.useFakeTimers();try {row.beforeViews.forEach(view=>{
    const submit=vi.fn();const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
