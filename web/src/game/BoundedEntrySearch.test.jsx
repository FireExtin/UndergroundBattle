import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import * as wasm from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import native from './boundedEntryNative63Test.fixture.json';
import { ChoicePanel } from './ChoicePanel';
import { Table } from './Table';
import { ReadModal } from './ReadModal';
import { cardScanUrl } from './cardScans';
wasm.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
afterEach(cleanup);
const catalog=JSON.parse(wasm.catalog());
const fixture=name=>native.states[name];
const action=v=>v.legalActions.find(a=>a.kind==='choose');
function choice(name) {
 const v=fixture(name).views[0], submit=vi.fn();
 const rendered=render(<ChoicePanel choice={v.pendingChoice} action={action(v)} busy={false} onSubmit={submit} definitions={new Map(catalog.cards.map(c=>[c.id,c]))}/>);
 return {...rendered,v,submit};
}
describe('fixed printed entry searches; actual Native layouts and current WASM',()=>{
 it('BQ104 accepts exactly one employee and keeps zero and a second employee unavailable',()=>{
  const {v,submit,container}=choice('employee');
  expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();
  expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
  const [first,second]=v.pendingChoice.options;
  fireEvent.click(container.querySelector(`[data-choice-option="${first.id}"]`));
  fireEvent.click(container.querySelector(`[data-choice-option="${second.id}"]`));
  fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
  expect(submit).toHaveBeenCalledExactlyOnceWith({...action(v),choiceId:v.pendingChoice.id,selected:[first.id]});
 });
 it.each(['employeeEmpty','passersEmpty'])('%s has an enabled explicit zero completion and no decline',name=>{
  const {v,submit}=choice(name);const button=screen.getByRole('button',{name:'不取牌并洗牌'});
  expect(button).toBeEnabled();expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();
  fireEvent.click(button);expect(submit).toHaveBeenCalledWith({...action(v),choiceId:v.pendingChoice.id,selected:[]});
 });
 it.each([0,1,3])('XQ48 actual UI submits %i real selected instances into current WASM',count=>{
  let state=fixture('passers').state;const v=JSON.parse(wasm.view(state,0));let transition;
  const submit=a=>{transition=JSON.parse(wasm.applyRoom(state,0,JSON.stringify({commandId:`ui-entry-${count}`,expectedVersion:v.version,action:{kind:'game',action:a}}),'0'));state=transition.state;};
  const {container,rerender}=render(<Table view={v} catalog={catalog} busy={false} onAction={submit}/>);
  for(const o of v.pendingChoice.options.slice(0,count)) fireEvent.click(container.querySelector(`[data-choice-option="${o.id}"]`));
  fireEvent.click(screen.getByRole('button',{name:count?'确认选择':'不取牌并洗牌'}));
  expect(transition.outcome).toBe('accepted');
  const next=JSON.parse(wasm.view(state,0));rerender(<Table view={next} catalog={catalog} busy={false} onAction={submit}/>);
  expect(next.regions[2].characters.filter(c=>c.cardId==='JC125')).toHaveLength(count);
  expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();
  expect(JSON.parse(state).game.players[0].deck).toHaveLength(6-count);
 });
 it.each(['employee','passers'])('%s candidate instances stay absent from both opponents and the teammate',name=>{
  const {rerender,container}=render(<Table view={fixture(name).views[0]} catalog={catalog} busy={false} onAction={vi.fn()}/>);
  expect(container.querySelector('[data-choice-option]')).not.toBeNull();
  for(const seat of [1,2,3]) {rerender(<Table view={fixture(name).views[seat]} catalog={catalog} busy={false} onAction={vi.fn()}/>);
   expect(container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();}
 });
 it('unrelated mandatory empty choices cannot inherit entry-search zero confirmation',()=>{
  const v=fixture('employeeEmpty').views[0];render(<ChoicePanel choice={{...v.pendingChoice,kind:'unrelated'}} action={action(v)} busy={false} onSubmit={vi.fn()} definitions={new Map(catalog.cards.map(c=>[c.id,c]))}/>);
  expect(screen.getByRole('button',{name:'确认选择'})).toBeDisabled();
 });
 it('original card scans match full source bytes and collector identity is independent of file ID',()=>{
  for(const [id,sha] of Object.entries({BQ104:'dbee5b3829cf0c46b2820beccefda28708c3c8e87c589dbefce1e975faa916eb',XQ48:'0cd31207bda681093839e86e5bb87cee624a3c0a657c70b60fa765f439089746'})) {
   const bytes=readFileSync(resolve(`public/cards/${id}.jpg`));expect(createHash('sha256').update(bytes).digest('hex')).toBe(sha);
   expect(cardScanUrl(id)).toBe(`/cards/${id}.jpg`);
   const c=catalog.cards.find(c=>c.id===id);expect(c).toMatchObject({color:'中立',loyalty:[],magic:'',defense:1,permanentIcons:{influence:1},temporaryIcons:{investigation:0,combat:0,influence:0}});
  }
 });
 it('hidden borrowed employee is visible to frozen controller and remains opaque to its owner',()=>{
  const mine=fixture('borrowed').views[0].regions[2].characters.find(c=>c.cardId==='XQ36');expect(mine.faceDown).toBe(true);expect(mine.owner).toBe('p2');expect(mine.controller).toBe('p0');
  for(const seat of [1,2,3]) {const c=fixture('borrowed').views[seat].regions[2].characters.find(c=>c.instanceId===mine.instanceId);expect(c.cardId==null).toBe(true);}
 });
 it.each(['BQ104','XQ48'])('reader opens %s original face and printed abilities',id=>{
  const c=catalog.cards.find(c=>c.id===id);const projected={...c,instanceId:`reader-${id}`,cardId:id,owner:'p0',controller:'p0',faceDown:false,exhausted:false,damage:0};
  render(<ReadModal card={projected} definition={c} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:`${c.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);
  fireEvent.click(screen.getByRole('button',{name:'当前状态与文字'})); expect(screen.getByRole('dialog').textContent.replace(/\s+/g,'')).toContain(c.text.replace(/\s+/g,''));
 });
});
