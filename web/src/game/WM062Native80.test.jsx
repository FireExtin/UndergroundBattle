// Fresh Native80 -> source-built WASM -> production choice UI; local component QA.
import { render, screen, fireEvent, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { loadFixture, sha256 } from '../../../tools/wm062-oracle.mjs';
import { kernel as abi, moduleBytes, catalog } from '../../../rust-game-wasm/tests/current80-kernel.mjs';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { actionForRoom } from './api';
const fixture=loadFixture();
expect(sha256(moduleBytes)).toBe(fixture.wasmSha256);
expect(catalog).toEqual(fixture.catalog);
const rows=fixture.scenarios.flatMap(s=>s.steps.flatMap((step,index)=>{
 const before=index?s.steps[index-1].views:s.initial.views;
 return !step.expected.errorCode && step.command.action.action?.kind==='choose'
  && before[step.seat].pendingChoice?.kind==='wm062-adjacent-move'
  ? [{...step,before,name:`${s.name}/${index}`}]:[];
}));
expect(rows.length).toBe(10);
beforeEach(()=>sessionStorage.clear());afterEach(()=>sessionStorage.clear());
it.each(rows)('$name restores the actual successor and submits only the optional second step',row=>{
 const view=row.before[row.seat],submit=vi.fn(),props={catalog,view,busy:false,connection:'online',onAction:submit};
 expect(JSON.parse(abi.view(row.state,row.seat))).toEqual(view);
 const first=render(<Table {...props}/>);
 let dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 expect(view.pendingChoice).toMatchObject({min:0,max:1,allowDecline:true});
 expect(view.pendingChoice.previewCards||[]).toEqual([]);
 expect(dialog).toHaveTextContent('潜伏已经完成');
 expect(within(dialog).getByRole('button',{name:'不移动并确认'})).toBeEnabled();
 first.unmount();
 const restored=JSON.parse(abi.view(row.state,row.seat));
 const mounted=render(<Table {...props} view={restored}/>);
 dialog=screen.getByRole('dialog',{name:'待完成的选择'});
 const selected=row.command.action.action.selected;
 if(selected.length){
  fireEvent.click(dialog.querySelector(`[data-choice-option="${selected[0]}"]`));
  fireEvent.click(within(dialog).getByRole('button',{name:'确认选择'}));
 }else fireEvent.click(within(dialog).getByRole('button',{name:'不移动并确认'}));
 expect(submit).toHaveBeenCalledOnce();
 const action=JSON.parse(JSON.stringify(actionForRoom(view,submit.mock.calls[0][0])));
 expect(action).toEqual(row.command.action);
 const result=JSON.parse(abi.applyRoom(row.state,row.seat,JSON.stringify({...row.command,action}),'0'));
 expect(result).toEqual(row.expected);
 row.views.forEach((v,seat)=>expect(JSON.parse(abi.view(result.state,seat))).toEqual(v));
 mounted.rerender(<Table {...props} busy/>);
 fireEvent.click(within(dialog).getByRole('button',{name:'正在提交…'}));
 expect(submit).toHaveBeenCalledOnce();
});
it('a teammate cannot choose the destination or inspect a newly enemy-controlled hidden front',()=>{
 const scenario=fixture.scenarios.find(s=>s.name==='controlled-foreign-successor');
 const row=scenario.steps.find(step=>step.views[0].pendingChoice?.kind==='wm062-adjacent-move');
 const actor=row.views[0],peer=row.views[2];
 expect(peer.pendingChoice).toBeNull();
 expect(actor.pendingChoice.previewCards||[]).toEqual([]);
 const hidden=actor.regions[2].characters.find(c=>c.faceDown);
 expect(hidden.cardId).toBeUndefined();expect(hidden.controller).toBe('p1');
 render(<Table catalog={catalog} view={peer} busy={false} connection="online" onAction={vi.fn()}/>);
 expect(screen.queryByRole('button',{name:'不移动并确认'})).not.toBeInTheDocument();
 expect(screen.queryByRole('button',{name:'确认选择'})).not.toBeInTheDocument();
});
it('the entire original transaction remains readable with cost, black loyalty and pinned art',()=>{
 const definition=catalog.cards.find(c=>c.id==='WM062');
 expect(definition).toMatchObject({kind:'spell',cost:2,color:'黑',loyalty:['黑色'],supported:true});
 expect(definition.subtypes).toEqual(['阴谋']);
 const card={instanceId:'read-art',cardId:'WM062',name:'跑路',kind:'spell',owner:'p0',controller:'p0',exhausted:false,faceDown:false};
 render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByRole('dialog')).toHaveTextContent('相邻地区');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
 expect(screen.getByRole('img',{name:'跑路原始牌面'})).toHaveAttribute('src','/cards/WM062.jpg');
});
