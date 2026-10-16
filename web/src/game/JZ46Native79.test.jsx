// Fresh Native79 -> real WASM -> actual Table choice; component QA, not online play.
import { render, screen, fireEvent, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { loadFixture, sha256 } from '../../../tools/jz46-oracle.mjs';
import { kernel as abi, moduleBytes, catalog } from '../../../rust-game-wasm/tests/current79-kernel.mjs';
import { Table } from './Table';
import { CardContent } from './CardTile';
import { actionForRoom } from './api';
const fixture=loadFixture();
expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);
expect(catalog).toEqual(fixture.catalog);
const rows=fixture.scenarios.flatMap(s=>s.steps.flatMap((step,index)=>{
  const before=index?s.steps[index-1].views:s.initial.views;
  return !step.expected.errorCode && step.command.action.action?.kind==='choose'
    && before[step.seat].pendingChoice?.kind==='jz46-hidden-inspect'
    ? [{...step,before,name:`${s.name}/${index}`}] : [];
}));
expect(rows.length).toBeGreaterThanOrEqual(8);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
it.each(rows)('$name privately restores the front and confirms the exact whole Native operation',row=>{
  const view=row.before[row.seat],submit=vi.fn(),props={catalog,view,busy:false,connection:'online',onAction:submit};
  expect(JSON.parse(abi.view(row.state,row.seat))).toEqual(view);
  const first=render(<Table {...props}/>);
  let dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  expect(within(dialog).getByRole('heading',{name:'你检视的暗藏者'})).toBeVisible();
  expect(view.pendingChoice).toMatchObject({min:0,max:0,allowDecline:false,options:[]});
  expect(within(dialog).queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
  expect(within(dialog).getByRole('button',{name:'确认选择'})).toBeEnabled();
  first.unmount();
  const remount=render(<Table {...props} view={JSON.parse(abi.view(row.state,row.seat))}/>);
  dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));
  expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
  expect(action).toEqual(row.command.action);
  const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));
  expect(result).toEqual(row.expected);
  row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
  remount.rerender(<Table {...props} busy/>);
  fireEvent.click(within(dialog).getByRole('button',{name:'正在提交…'}));
  expect(submit).toHaveBeenCalledOnce();
});
it('a teammate receives neither the private preview nor the inspection controls',()=>{
  const row=rows.find(r=>r.seat===0);const view=row.before[1];
  expect(view.pendingChoice).toBeUndefined();expect(view.waitingChoice.kind).toBe('jz46-hidden-inspect');
  render(<Table catalog={catalog} view={view} busy={false} connection="online" onAction={vi.fn()}/>);
  expect(screen.queryByRole('heading',{name:'你检视的暗藏者'})).not.toBeInTheDocument();
  expect(screen.queryByRole('button',{name:'确认选择'})).not.toBeInTheDocument();
});
it('the complete JZ46 face retains its region host, loyalty, domain, permanent influence and pinned original',()=>{
  const definition=catalog.cards.find(c=>c.id==='JZ46');
  expect(definition).toMatchObject({cost:2,color:'灰',loyalty:['灰色'],magic:'神圣',permanentIcons:{influence:1},supported:true});
  const card={instanceId:'art-only',cardId:'JZ46',name:definition.name,kind:'attachment',owner:'p0',controller:'p0',exhausted:false,faceDown:false};
  const {container}=render(<CardContent card={card} definition={definition}/>);
  expect(container.textContent).toContain('狴犴铜雕');expect(container.textContent).toContain('检视');
  expect(container.querySelector('img')).toHaveAttribute('src','/cards/JZ46.jpg');
});
