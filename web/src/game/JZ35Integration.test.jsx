// Requires newly generated Native75 inputs and the matching production ABI.
// Intentionally has no fallback to Native71 or synthetic state expectations.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { render,screen,fireEvent,act } from '@testing-library/react';
import { it,expect,vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { Table } from './Table';
import { actionForRoom } from './api';
const wasm=readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'));
abi.initSync({module:wasm});
const catalog=JSON.parse(abi.catalog());
const fixture=JSON.parse(readFileSync(resolve('src/game/jz35Native75.fixture.json'),'utf8'));
it('uses the newly built JZ35 catalog and the exact matching Native75 ABI hash',()=>{
  expect(catalog.engineVersion).toBe('rust-v0.2.75-jc060-occult-adviser-candidate');
  expect(catalog.cardPoolVersion).toBe('limited-v2.68-jc060-occult-adviser-candidate');
  expect(createHash('sha256').update(wasm).digest('hex')).toBe(fixture.wasmSha256);
  expect(catalog.cards.find(c=>c.id==='JZ35').abilities).toMatchObject([{key:'jz35-sacrifice-return-controlled-hosts',timing:'standard',triggered:false,costs:[]}]);
});
it.each(fixture.cases)('$name submits the exact untargeted paid Native Room command after restore',row=>{
  const view=JSON.parse(abi.view(row.before,row.seat));expect(view).toEqual(row.beforeViews[row.seat]);
  const submit=vi.fn();let ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit}/>);ui.unmount();
  ui=render(<Table catalog={catalog} view={JSON.parse(abi.view(row.before,row.seat))} busy={false} onAction={submit}/>);
  const wanted=row.command.action.action;
  const legal=view.legalActions.find(a=>a.kind==='play'&&a.cardId===wanted.cardId&&a.abilityId===wanted.abilityId);
  expect(legal).toBeDefined();expect(legal.targetId).toBeUndefined();expect(legal.region).toBeUndefined();
  const source=ui.container.querySelector(`[data-card-instance="${wanted.cardId}"]`);expect(source).toBeInTheDocument();fireEvent.click(source);
  const button=Array.from(ui.container.querySelectorAll("[data-action-id]")).find(element=>element.getAttribute("data-action-id")===legal.id);expect(button).toBeEnabled();
  fireEvent.click(button);expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.before,row.seat,JSON.stringify({...row.command,action}),'0'));expect(result).toEqual(row.expected);
  row.afterViews.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
});
it.each(fixture.cases)('$name preserves four-seat privacy and never auto-plays the spell',row=>{
  vi.useFakeTimers();try{row.beforeViews.forEach((view,seat)=>{
    expect(JSON.parse(abi.view(row.before,seat))).toEqual(view);const submit=vi.fn();
    const ui=render(<Table catalog={catalog} view={view} busy={false} onAction={submit} autoPassEnabled/>);
    expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
    act(()=>vi.advanceTimersByTime(60000));expect(submit).not.toHaveBeenCalled();ui.unmount();
  });}finally{vi.useRealTimers();}
});
