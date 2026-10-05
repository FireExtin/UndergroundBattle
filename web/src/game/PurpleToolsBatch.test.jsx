import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { Table } from './Table';
import { testView, testCard } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
for (const [id, sha, fields] of [
 ['JC103','8eee910793908999e04487b65f0e9a398d11ef5b307b360ba6e9d2012adaed63',{name:'丧失理智的画家',cost:3,loyalty:['紫色'],color:'紫',kind:'character',subtypes:['人类','艺术家'],magic:'',unique:false,defense:1,permanentIcons:{investigation:1,combat:0,influence:1},temporaryIcons:{investigation:0,combat:1,influence:0}}],
 ['JC107','ce402d531c9c9bb28da130de0627b4860ced56fb0fa7db1d487082c36e0462e2',{name:'崩塌的时空',cost:3,loyalty:['紫色','紫色'],color:'紫',kind:'spell',subtypes:['事务','灾难'],magic:'',unique:false}],
]) it(`preserves original ${id} bytes and actual compiled printed fields`,()=>{
 const d=catalog.cards.find(c=>c.id===id);expect(d).toMatchObject(fields);
 expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
 const m=render(<ReadModal card={{...d,cardId:id,instanceId:id,owner:'p0',controller:'p0',exhausted:false,faceDown:false}} definition={d} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:`${d.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);m.unmount();
});
const d=catalog.cards.find(c=>c.id==='JC107');
const spell={...d,instanceId:'collapse',cardId:'JC107',owner:'p0',controller:'p0',exhausted:false,faceDown:false,effectiveCost:2};
const players=[0,1,2,3].map(s=>({...testView.players[0],id:`p${s}`,seat:s,name:`玩家${s}`,team:Math.floor(s/2)}));
const assets=[0,1,2,3].map(s=>({instanceId:`asset-${s}`,kind:'asset',name:'资产',owner:s===2?'p3':`p${s}`,controller:`p${s}`,exhausted:false,faceDown:false}));
const actions=assets.map(a=>({id:`collapse-${a.instanceId}`,kind:'play',cardId:'collapse',abilityId:'destroy-attachment-or-asset',targetId:a.instanceId,label:`崩塌的时空：标准行动 → 资产 [${a.instanceId}]`}));
const region={id:'r0',index:0,cardId:'DQJC115',name:'死者之城',threshold:3,points:2,influence:[0,0],characters:[{...testCard,instanceId:'character-not-target',owner:'p2',controller:'p2',region:0}]};
const base={...testView,status:'playing',mode:'teams',you:'p0',players,assets,hand:[spell],regions:[region],legalActions:actions};
it('selects public assets in each holder mat and forwards the exact server action while keeping identities hidden',()=>{
 const submit=vi.fn();const m=render(<Table view={base} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看崩塌的时空'}));fireEvent.click(screen.getByRole('button',{name:'崩塌的时空：标准行动'}));
 for(let s=0;s<4;s++){const mat=screen.getByLabelText(`玩家${s}的资产`);const button=within(mat).getByRole('button',{name:'查看资产'});expect(button).toHaveAttribute('data-card-targeted','true');expect(button).not.toHaveAttribute('data-card-id');}
 expect(m.container.querySelector('[data-card-instance="character-not-target"]')).toHaveAttribute('data-card-targeted','false');
 fireEvent.click(within(screen.getByLabelText('玩家2的资产')).getByRole('button',{name:'查看资产'}));
 fireEvent.click(screen.getByRole('button',{name:`确认 · ${actions[2].label}`}));expect(submit).toHaveBeenCalledExactlyOnceWith(actions[2]);m.unmount();
});
it.each(['p0','p1','p2','p3'])('reads anonymized assets without reconstructing original scans for %s',you=>{
 const m=render(<Table view={{...base,you,hand:[],legalActions:[]}} catalog={catalog} busy={false} onAction={vi.fn()}/>);
 fireEvent.click(within(screen.getByLabelText('玩家2的资产')).getByRole('button',{name:'查看资产'}));fireEvent.click(screen.getByRole('button',{name:'放大文字与图标 ↗'}));
 expect(screen.getByRole('dialog',{name:'放大阅读资产'})).toBeInTheDocument();expect(screen.queryByRole('button',{name:'原始牌面'})).not.toBeInTheDocument();m.unmount();
});
it('drops a removed asset target before confirmation even when a stale legal action remains',()=>{
 const submit=vi.fn();const m=render(<Table view={base} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看崩塌的时空'}));fireEvent.click(screen.getByRole('button',{name:'崩塌的时空：标准行动'}));fireEvent.click(within(screen.getByLabelText('玩家2的资产')).getByRole('button',{name:'查看资产'}));
 m.rerender(<Table view={{...base,version:2,assets:assets.filter(a=>a.instanceId!=='asset-2')}} catalog={catalog} busy={false} onAction={submit}/>);
 expect(screen.queryByRole('button',{name:`确认 · ${actions[2].label}`})).not.toBeInTheDocument();expect(submit).not.toHaveBeenCalled();m.unmount();
});
it('selects the mounted attachment across regions without treating its host as an allowed target',()=>{
 const attachment={...testCard,instanceId:'vest',cardId:'XQ47',kind:'attachment',name:'防弹背心',owner:'p2',controller:'p2',region:1,hostId:'host'};
 const host={...testCard,instanceId:'host',owner:'p2',controller:'p2',region:1};const a={...actions[0],targetId:'vest',label:'崩塌的时空：标准行动 → 防弹背心 [vest]'};
 const submit=vi.fn();const m=render(<Table view={{...base,attachments:[attachment],regions:[region,{...region,id:'r1',index:1,characters:[host]}],legalActions:[a]}} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看崩塌的时空'}));expect(m.container.querySelector('[data-card-instance="host"]')).toHaveAttribute('data-card-targeted','false');fireEvent.click(screen.getByRole('button',{name:'查看附属防弹背心'}));fireEvent.click(screen.getByRole('button',{name:`确认 · ${a.label}`}));expect(submit).toHaveBeenCalledExactlyOnceWith(a);m.unmount();
});
