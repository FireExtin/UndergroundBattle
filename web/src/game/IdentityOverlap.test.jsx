// Explicit component fixtures; no browser or production-room claims.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { Table } from './Table';
import { ChoicePanel } from './ChoicePanel';
import { testView } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog=JSON.parse(kernel.catalog());const definitions=new Map([...catalog.cards,...catalog.societies].map(d=>[d.id,d]));
const red=definitions.get('JC042');const hermit=definitions.get('LC12');
const reds=[0,1,2,3].map(s=>({...red,instanceId:`red-${s}`,cardId:'JC042',owner:`p${s}`,controller:`p${s}`,faceDown:false,exhausted:false,region:0}));
const source={...hermit,instanceId:'hermit-owner2-controller0',cardId:'LC12',owner:'p2',controller:'p0',faceDown:false,exhausted:false,region:0};
const players=[0,1,2,3].map(s=>({...testView.players[0],id:`p${s}`,seat:s,name:`玩家${s}`,team:Math.floor(s/2)}));
const region={id:'region-0',index:0,cardId:'DQJC115',name:'死者之城',threshold:3,points:2,influence:[0,0],characters:[source,...reds]};
const action={id:'exact-red-instance-target',kind:'activate',cardId:source.instanceId,abilityId:'protect-local-character',targetId:'red-0',label:'阿瓦隆隐士：快速行动 → 末日信徒 [red-0]'};
const base={...testView,status:'playing',mode:'teams',you:'p0',players,hand:[],regions:[region],legalActions:[action]};
it('keeps four same-definition red tiles separate and submits only the exact controller-eligible target',()=>{
 const submit=vi.fn();const m=render(<Table view={base} catalog={catalog} busy={false} onAction={submit}/>);
 expect(m.container.querySelectorAll('[data-card-instance^="red-"]')).toHaveLength(4);
 fireEvent.click(m.container.querySelector(`[data-card-instance="${source.instanceId}"]`));fireEvent.click(screen.getByRole('button',{name:'阿瓦隆隐士：快速行动'}));
 for(const c of reds)expect(m.container.querySelector(`[data-card-instance="${c.instanceId}"]`)).toHaveAttribute('data-card-targeted',String(c.instanceId==='red-0'));
 fireEvent.click(m.container.querySelector('[data-card-instance="red-0"]'));fireEvent.click(screen.getByRole('button',{name:`确认 · ${action.label}`}));expect(submit).toHaveBeenCalledExactlyOnceWith(action);m.unmount();
});
it('clears old confirmation when a same-name fresh red instance replaces the bound target',()=>{
 const submit=vi.fn();const m=render(<Table view={base} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(m.container.querySelector(`[data-card-instance="${source.instanceId}"]`));fireEvent.click(screen.getByRole('button',{name:'阿瓦隆隐士：快速行动'}));fireEvent.click(m.container.querySelector('[data-card-instance="red-0"]'));
 const fresh={...reds[0],instanceId:'red-0-new'};m.rerender(<Table view={{...base,version:base.version+1,regions:[{...region,characters:[source,fresh,...reds.slice(1)]}],legalActions:[{...action,id:'fresh-target',targetId:fresh.instanceId,label:'阿瓦隆隐士：快速行动 → 末日信徒 [red-0-new]'}]}} catalog={catalog} busy={false} onAction={submit}/>);
 expect(screen.queryByRole('button',{name:`确认 · ${action.label}`})).not.toBeInTheDocument();expect(submit).not.toHaveBeenCalled();expect(m.container.querySelector('[data-card-instance="red-0"]')).toBeNull();expect(m.container.querySelector('[data-card-instance="red-0-new"]')).not.toBeNull();m.unmount();
});
it('allocates damage to one same-name red instance using its exact choice option id',()=>{
 const choice={id:'same-red-damage',kind:'damage',title:'分配伤害',playerId:'p0',amount:1,options:reds.map(c=>({id:c.instanceId,label:c.name,card:c}))};const a={id:'damage-choice',kind:'choose',choiceId:choice.id,label:'确认选择'};const submit=vi.fn();const m=render(<ChoicePanel choice={choice} action={a} definitions={definitions} busy={false} onSubmit={submit} onReadCard={vi.fn()} viewerId="p0"/>);
 fireEvent.click(m.container.querySelector('[data-choice-option="red-2"]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenCalledExactlyOnceWith({...a,allocations:{'red-0':0,'red-1':0,'red-2':1,'red-3':0}});m.unmount();
});
it('selects one of four same-definition societies by its instance and personal zone',()=>{
 const d=definitions.get('MSJC09');const zones=[0,1,2,3].map(s=>({id:`society:p${s}`,playerId:`p${s}`,card:{...d,instanceId:`head-${s}`,cardId:'MSJC09',owner:`p${s}`,controller:`p${s}`,faceDown:false,exhausted:false}}));const a={id:'personal-head0-action',kind:'activate',cardId:'head-0',abilityId:'drawWithInitiative',sourceZoneId:'society:p0',label:'秘社：先手抓牌'};const submit=vi.fn();const m=render(<Table view={{...base,regions:[],societyZones:zones,legalActions:[a]}} catalog={catalog} busy={false} onAction={submit}/>);
 expect(m.container.querySelectorAll('[data-card-instance^="head-"]')).toHaveLength(4);fireEvent.click(m.container.querySelector('[data-card-instance="head-0"]'));fireEvent.click(screen.getByRole('button',{name:'秘社：先手抓牌'}));expect(submit).toHaveBeenCalledExactlyOnceWith(a);m.unmount();
});
