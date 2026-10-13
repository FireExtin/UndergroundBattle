// Real Native inputs and production WASM; no mocked game API or browser claim.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as abi from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { actionForRoom } from './api';
import { ChoicePanel } from './ChoicePanel';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { groupObjectActions } from './objectActions';

abi.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(abi.catalog()), definitions=new Map(catalog.cards.map(c=>[c.id,c]));
const fixtures=JSON.parse(readFileSync(resolve('src/game/xq24Native68.fixture.json')));

it('reads the complete original XQ24 face from the admitted public asset',()=>{
  const definition=definitions.get('XQ24');
  expect(definition).toMatchObject({name:'防暴警察',cost:4,loyalty:['灰色','灰色'],defense:2});
  render(<ReadModal card={{...definition,cardId:'XQ24',instanceId:'read-xq24',owner:'p0',controller:'p0'}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
  expect(screen.getByRole('img',{name:'防暴警察原始牌面'})).toHaveAttribute('src','/cards/XQ24.jpg');
});

it.each(['duel','teams'])('submits the real %s paid entry through the existing table controls',mode=>{
  const scenario=fixtures.scenarios.find(s=>s.name===`${mode}-accept`),step=scenario.steps[0];
  const view=JSON.parse(abi.view(step.state,0)),submit=vi.fn();
  const {container}=render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={submit}/>);
  fireEvent.click(container.querySelector(`[data-card-instance="${step.command.action.action.cardId}"]`));
  const group=groupObjectActions(view.legalActions.filter(a=>a.cardId===step.command.action.action.cardId))
    .find(g=>g.actions[0].kind==='deploy');
  fireEvent.click(screen.getByRole('button',{name:group.label}));
  const region=view.regions.find(r=>r.index===2);
  fireEvent.click(screen.getByRole('button',{name:`查看地区3 ${region.name}`}));
  const confirm=container.querySelector('[data-action-kind="deploy"][data-action-region="2"]');
  expect(confirm).toBeEnabled();
  fireEvent.click(confirm);
  expect(submit).toHaveBeenCalledOnce();
  const command={...step.command,action:actionForRoom(view,submit.mock.calls[0][0])};
  expect(JSON.parse(abi.applyRoom(step.state,0,JSON.stringify(command),'0'))).toEqual(step.expected);
});

it.each(['duel','teams'].flatMap(mode=>['accept','decline'].map(choice=>[mode,choice])))('submits exact %s %s declaration and matches Native', (mode,choice)=>{
  const scenario=fixtures.scenarios.find(s=>s.name===`${mode}-${choice}`);
  const step=scenario.steps.find(s=>s.command.action.action?.kind==='choose');
  const view=JSON.parse(abi.view(step.state,step.seat)),submit=vi.fn();
  const {container}=render(<ChoicePanel choice={view.pendingChoice} action={view.legalActions.find(a=>a.kind==='choose')} definitions={definitions} busy={false} viewerId={view.you} onSubmit={submit}/>);
  expect(view.pendingChoice.options.map(o=>o.id)).toEqual(['accept']);
  expect(view.pendingChoice.title).toMatch(/防暴警察/);
  if(choice==='accept') {
    fireEvent.click(container.querySelector('[data-choice-option="accept"]'));
    fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
  } else fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));
  const command={...step.command,action:actionForRoom(view,submit.mock.calls[0][0])};
  expect(JSON.parse(abi.applyRoom(step.state,step.seat,JSON.stringify(command),'0'))).toEqual(step.expected);
});

it.each(['duel','teams'])('renders %s opponents anonymous after the actual untargeted resolution without leaking their card face', mode=>{
  const step=fixtures.scenarios.find(s=>s.name===`${mode}-accept`).steps.at(-1);
  const view=JSON.parse(abi.view(step.expected.state,0));
  const hidden=view.regions[2].characters.find(c=>c.faceDown);
  expect(hidden).toBeDefined();
  for(const field of ['cardId','text','cost','icons','defense','color','magic']) expect(hidden[field]).toBeUndefined();
  const {container}=render(<Table view={view} catalog={catalog} busy={false} connection="online" onAction={vi.fn()}/>);
  const tile=container.querySelector(`[data-card-instance="${hidden.instanceId}"]`);
  expect(tile).toHaveTextContent('暗藏者');
  expect(tile.querySelector('img[src="/cards/JZ44.jpg"]')).toBeNull();
  expect([...container.querySelectorAll('.hg-board article')].map(el=>el.id)).toEqual(view.regions.map(r=>`hg-region-${r.index}`));
});
