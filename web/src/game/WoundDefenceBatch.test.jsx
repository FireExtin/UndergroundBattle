import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { CardTile } from './CardTile';
import { Table } from './Table';
import { testView, testCard } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog = JSON.parse(kernel.catalog());
for (const [id, sha, fields] of [
 ['JZ59','b43bf4fc74654c089cf10429306980c2af12743d57d60b603efae03b28eae5f3',{name:'骇人畸变体',cost:2,loyalty:['紫色'],color:'紫',kind:'character',subtypes:['人类','宿主'],magic:'',unique:false,defense:1,permanentIcons:{investigation:0,combat:1,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0},abilities:[{key:'death-local-wound',timing:'fast',costs:[],triggered:true}]}],
 ['LC12','2a13b5e0bf5fa11f320bcf31141cf0e68915a949f9031cd112b5c650420285cf',{name:'阿瓦隆隐士',cost:2,loyalty:['白色'],color:'白',kind:'character',subtypes:['人类','法师'],magic:'星辰',unique:false,defense:1,permanentIcons:{investigation:0,combat:0,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0},abilities:[{key:'protect-local-character',timing:'fast',costs:['ExhaustSource'],triggered:false}]}],
]) it(`reads unchanged ${id} original bytes and actual compiled printed fields`,()=>{
 const d=catalog.cards.find(c=>c.id===id);expect(d).toMatchObject(fields);
 expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
 const m=render(<ReadModal card={{...d,cardId:id,instanceId:id,owner:'p0',controller:'p0',exhausted:false,faceDown:false}} definition={d} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:`${d.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);m.unmount();
});
const hermit=catalog.cards.find(c=>c.id==='LC12');
const h={...hermit,instanceId:'hermit',cardId:'LC12',owner:'p0',controller:'p0',exhausted:false,faceDown:false,region:0};
const own={...testCard,instanceId:'controlled',owner:'p3',controller:'p0',region:0};
const teammate={...testCard,instanceId:'teammate',owner:'p1',controller:'p1',region:0};
const outside={...testCard,instanceId:'outside',region:1};
const players=[0,1,2,3].map(s=>({...testView.players[0],id:`p${s}`,seat:s,name:`玩家${s}`,team:Math.floor(s/2)}));
const region={id:'r0',index:0,cardId:'DQJC115',name:'死者之城',threshold:3,points:2,influence:[0,0],characters:[h,own,teammate]};
const a={id:'hermit-protect',kind:'activate',cardId:'hermit',abilityId:'protect-local-character',targetId:'controlled',label:'阿瓦隆隐士：快速行动 → 无知路人 [controlled]'};
const base={...testView,status:'playing',mode:'teams',you:'p0',players,hand:[],regions:[region,{...region,id:'r1',index:1,characters:[outside]}],legalActions:[a]};
it('selects the current controller target from the server list and forwards exact exhaust-only activation',()=>{
 const submit=vi.fn();const m=render(<Table view={base} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看阿瓦隆隐士'}));fireEvent.click(screen.getByRole('button',{name:'阿瓦隆隐士：快速行动'}));
 expect(m.container.querySelector('[data-card-instance="controlled"]')).toHaveAttribute('data-card-targeted','true');
 for(const id of ['hermit','teammate','outside'])expect(m.container.querySelector(`[data-card-instance="${id}"]`)).toHaveAttribute('data-card-targeted','false');
 fireEvent.click(m.container.querySelector('[data-card-instance="controlled"]'));
 fireEvent.click(screen.getByRole('button',{name:`确认 · ${a.label}`}));expect(submit).toHaveBeenCalledExactlyOnceWith(a);m.unmount();
});
it('invalidates a selected target when a new server projection removes its legal action',()=>{
 const submit=vi.fn();const m=render(<Table view={base} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(screen.getByRole('button',{name:'查看阿瓦隆隐士'}));fireEvent.click(screen.getByRole('button',{name:'阿瓦隆隐士：快速行动'}));fireEvent.click(m.container.querySelector('[data-card-instance="controlled"]'));
 m.rerender(<Table view={{...base,version:2,legalActions:[]}} catalog={catalog} busy={false} onAction={submit}/>);
 expect(screen.queryByRole('button',{name:`确认 · ${a.label}`})).not.toBeInTheDocument();expect(submit).not.toHaveBeenCalled();m.unmount();
});
it.each(['p0','p1','p2','p3'])('shows current defense, printed defense and wounds separately then updates expiry for %s',you=>{
 const d=catalog.cards.find(c=>c.id==='LC01');const c={...d,cardId:'LC01',instanceId:'wounded',owner:'p2',controller:'p2',exhausted:false,faceDown:false,region:0,defense:2,wounds:1,damage:0};
 const m=render(<CardTile card={c} definition={d} viewerId={you}/>);
 expect(screen.getByText(`当前防御 2 · 印刷防御 ${d.defense}`)).toBeInTheDocument();expect(screen.getByText('创伤 1')).toBeInTheDocument();expect(m.container.querySelector('.hg-damage-marker')).toBeNull();
 m.rerender(<CardTile card={{...c,defense:1}} definition={d} viewerId={you}/>);
 expect(screen.getByText(`当前防御 1 · 印刷防御 ${d.defense}`)).toBeInTheDocument();expect(screen.getByText('创伤 1')).toBeInTheDocument();m.unmount();
});
const choice={id:'death-wound',kind:'target',title:'骇人畸变体：死亡触发',description:'选择本地区角色造成1点创伤',playerId:'p0',min:1,max:1,allowDecline:true,options:[{id:own.instanceId,label:own.name,card:own}]};
const choose={id:'choose-wound',kind:'choose',choiceId:choice.id,label:'确认选择'};
it('submits the death source controller choice with its exact instance and permits free decline',()=>{
 const submit=vi.fn();const v={...base,pendingChoice:choice,legalActions:[choose]};const m=render(<Table view={v} catalog={catalog} busy={false} onAction={submit}/>);
 fireEvent.click(m.container.querySelector('[data-choice-option="controlled"]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));
 expect(submit).toHaveBeenLastCalledWith({...choose,choiceId:choice.id,selected:['controlled']});
 fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(submit).toHaveBeenLastCalledWith({...choose,choiceId:choice.id,selected:[]});m.unmount();
});
it.each(['p1','p2','p3'])('renders only waiting status for the non-controller death-choice projection %s',you=>{
 const submit=vi.fn();const m=render(<Table view={{...base,you,pendingChoice:null,waitingChoice:{playerId:choice.playerId,title:choice.title,kind:choice.kind},legalActions:[]}} catalog={catalog} busy={false} onAction={submit}/>);
 expect(screen.queryByRole('dialog',{name:'待完成的选择'})).not.toBeInTheDocument();expect(m.container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('button',{name:'确认选择'})).not.toBeInTheDocument();expect(submit).not.toHaveBeenCalled();m.unmount();
});
