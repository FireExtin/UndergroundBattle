// Compiler-bound Native81 -> fresh WASM81 -> real Table. Local component QA only.
import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { loadFixture, sha256 } from '../../../tools/jc054-oracle.mjs';
import { kernel as abi, catalog, moduleBytes } from '../../../rust-game-wasm/tests/current81-kernel.mjs';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { actionForRoom } from './api';
const fixture=loadFixture();
expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);expect(catalog).toEqual(fixture.catalog);
const choices=fixture.scenarios.flatMap(s=>s.steps.flatMap((row,index)=>{
  const before=index?s.steps[index-1].views:s.initial.views;
  return !row.expected.errorCode&&row.command.action.action?.kind==='choose'
    &&before[row.seat].pendingChoice?.kind==='jc054-trigger-order'?[{...row,before,name:`${s.name}/${index}`}]:[];
}));
expect(choices.length).toBe(3);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
it.each(choices)('$name restores a mandatory ordering choice with no decline and one exact submission',row=>{
  const view=row.before[row.seat],submit=vi.fn(),props={catalog,view,busy:false,connection:'online',onAction:submit};
  expect(JSON.parse(abi.view(row.state,row.seat))).toEqual(view);
  expect(view.pendingChoice).toMatchObject({min:1,max:1,allowDecline:false});
  const app=render(<Table {...props}/>);app.unmount();
  const restored=JSON.parse(abi.view(row.state,row.seat));render(<Table {...props} view={restored}/>);
  const dialog=screen.getByRole('dialog',{name:'待完成的选择'});
  expect(dialog).toHaveTextContent('必须将全部触发入栈');
  expect(within(dialog).queryByRole('button',{name:/跳过|不发动|放弃本次/})).not.toBeInTheDocument();
  const confirm=within(dialog).getByRole('button',{name:'确认选择'});expect(confirm).toBeDisabled();
  const id=row.command.action.action.selected[0];fireEvent.click(dialog.querySelector(`[data-choice-option="${id}"]`));
  fireEvent.click(confirm);fireEvent.click(confirm);expect(submit).toHaveBeenCalledOnce();
  const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));expect(action).toEqual(row.command.action);
  expect(JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'))).toEqual(row.expected);
  row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(row.expected.state,seat))).toEqual(v));
});
it('keeps both printed abilities, separate permanent/temporary combat icons and original scan readable',()=>{
  const d=catalog.cards.find(c=>c.id==='JC054');expect(d.abilities).toHaveLength(2);expect(d.abilities.every(a=>a.triggered)).toBe(true);
  expect(d.permanentIcons.combat).toBe(1);expect(d.temporaryIcons.combat).toBe(1);
  const card={instanceId:'jc054-reader',cardId:d.id,name:d.name,kind:d.kind,owner:'p0',controller:'p0',faceDown:false,exhausted:false};
  render(<ReadModal card={card} definition={d} viewerId="p0" onClose={vi.fn()}/>);
  const dialog=screen.getByRole('dialog');expect(dialog).toHaveTextContent('潜伏强制触发');expect(dialog).toHaveTextContent('结束步骤强制触发');
  fireEvent.click(within(dialog).getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'蒙古死亡蠕虫原始牌面'})).toHaveAttribute('src','/cards/JC054.jpg');
});
it('the paid reveal response exposes no hidden front to the other team before revealing',()=>{
  const s=fixture.scenarios.find(s=>s.name==='paid-response-reveal');
  const actor=s.initial.views[0],other=s.initial.views[2];
  expect(actor.regions.flatMap(r=>r.characters).find(c=>c.faceDown).cardId).toBe('JC054');
  const hidden=other.regions.flatMap(r=>r.characters).find(c=>c.faceDown);
  expect(hidden.cardId).toBeUndefined();expect(hidden.name).toBe('暗藏者');
  render(<Table view={other} catalog={catalog} busy={false} connection="online" onAction={vi.fn()}/>);
  expect(screen.queryByText('候选·快速')).not.toBeInTheDocument();
});
