import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, fireEvent } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import { ReadModal } from './ReadModal';
import { CardTile, visibleCard } from './CardTile';
import { Table } from './Table';
import { testView } from './testFixtures';
kernel.initSync({ module: readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm')) });
const catalog=JSON.parse(kernel.catalog());
for(const[id,sha,fields]of[
 ['JZ58','4cb1892b435cc14fd3f9d8310b428a16eaf5cc5afe1c870760a3dcc457d4dd95',{name:'噩梦残像',cost:0,loyalty:['紫色'],color:'紫',magic:'星辰',kind:'character',subtypes:['梦魔'],unique:false,defense:1,permanentIcons:{investigation:0,combat:0,influence:0},temporaryIcons:{investigation:0,combat:0,influence:1},keywords:['灵体'],ruleTraits:{spirit:true},abilities:[{key:'reveal-repress',costs:[],triggered:true,timing:'fast'}]}],
 ['JZ61','5da40b475c306d3a295a6582576b9b720c8707050bdedfea3005b92578e4e37f',{name:'暗夜访客',cost:2,loyalty:['紫色'],color:'紫',magic:'',kind:'character',subtypes:['人类','宿主'],unique:false,defense:1,permanentIcons:{investigation:0,combat:1,influence:1},temporaryIcons:{investigation:0,combat:0,influence:0},abilities:[{key:'death-find-nightmare',costs:[],triggered:true,timing:'fast'}]}],
])it(`reads ${id} unchanged original and actual compiled fields`,()=>{
 const d=catalog.cards.find(c=>c.id===id);expect(d).toMatchObject(fields);expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
 const m=render(<ReadModal card={{...d,cardId:id,instanceId:id,owner:'p0',controller:'p0',exhausted:false,faceDown:false}} definition={d} viewerId="p0" onClose={vi.fn()}/>);fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:`${d.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);m.unmount();
});
const d=catalog.cards.find(c=>c.id==='JZ58');const nightmare={...d,cardId:'JZ58',instanceId:'nightmare',owner:'p3',controller:'p0',exhausted:false,faceDown:false,region:0,defense:1};
it.each(['p0','p1','p2','p3'])('updates public spirit protection with the latest domain projection for %s',you=>{
 const m=render(<CardTile card={{...nightmare,currentSpiritProtection:true}} definition={d} viewerId={you}/>);expect(screen.getByText('灵体：当前防止伤害')).toBeInTheDocument();expect(screen.queryByText('本回合防止伤害')).not.toBeInTheDocument();
 m.rerender(<CardTile card={{...nightmare,currentSpiritProtection:false}} definition={d} viewerId={you}/>);expect(screen.getByText('灵体：当前不防止伤害')).toBeInTheDocument();expect(screen.queryByText('灵体：当前防止伤害')).not.toBeInTheDocument();m.unmount();
});
it.each(['p0','p1','p2','p3'])('withholds stale spirit protection when the instance is concealed for %s',you=>{
 const c={...nightmare,faceDown:true,currentSpiritProtection:true};const m=render(<CardTile card={c} definition={d} viewerId={you}/>);expect(screen.queryByText('灵体：当前防止伤害')).not.toBeInTheDocument();expect(screen.queryByText('灵体：当前不防止伤害')).not.toBeInTheDocument();if(you!=='p0'){expect(visibleCard(c,you).currentSpiritProtection).toBeUndefined();expect(visibleCard(c,you).cardId).toBeUndefined();expect(screen.getByRole('button',{name:'查看暗藏者'})).toBeInTheDocument();}m.unmount();
});
const players=[0,1,2,3].map(s=>({...testView.players[0],id:`p${s}`,seat:s,name:`玩家${s}`,team:Math.floor(s/2)}));
const base={...testView,status:'playing',mode:'teams',players,you:'p0',hand:[],regions:[],legalActions:[]};
const opponent={id:'repress-enemy',kind:'trigger',title:'噩梦残像：现身触发',description:'选择敌方玩家',playerId:'p0',min:0,max:1,allowDecline:true,options:[{id:'p2',label:'玩家2'},{id:'p3',label:'玩家3'}]};
const region={id:'repress-region',kind:'target',title:'遏制1：选择移除本方势力的地区',description:'选择一处本方势力所在地区',playerId:'p2',min:1,max:1,allowDecline:false,options:[{id:'region:2',label:'地区3：本方势力2'},{id:'region:4',label:'地区5：本方势力1'}]};
it('forwards opponent selection and the target player region selection as distinct exact choice payloads',()=>{
 const submit=vi.fn();const a={id:'opponent-choice',kind:'choose',choiceId:opponent.id,label:'确认选择'};const b={id:'region-choice',kind:'choose',choiceId:region.id,label:'确认选择'};
 const m=render(<Table view={{...base,pendingChoice:opponent,legalActions:[a]}} catalog={catalog} busy={false} onAction={submit}/>);fireEvent.click(m.container.querySelector('[data-choice-option="p2"]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenLastCalledWith({...a,selected:['p2']});expect(m.container.querySelector('[data-choice-option="p1"]')).toBeNull();
 m.rerender(<Table view={{...base,you:'p2',version:2,pendingChoice:region,legalActions:[b]}} catalog={catalog} busy={false} onAction={submit}/>);expect(screen.queryByRole('button',{name:'跳过此选择'})).not.toBeInTheDocument();fireEvent.click(m.container.querySelector('[data-choice-option="region:4"]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenLastCalledWith({...b,selected:['region:4']});m.unmount();
});
it.each(['p0','p1','p3'])('does not show region-choice options to non-target projection %s',you=>{
 const m=render(<Table view={{...base,you,pendingChoice:null,waitingChoice:{playerId:region.playerId,title:region.title,kind:region.kind}}} catalog={catalog} busy={false} onAction={vi.fn()}/>);expect(m.container.querySelector('[data-choice-option]')).toBeNull();expect(screen.queryByRole('button',{name:'确认选择'})).not.toBeInTheDocument();m.unmount();
});
it('offers real Amnesia optional shuffle confirmation and decline without inventing additional costs',()=>{
 const c={id:'dream-shuffle',kind:'optional-shuffle',title:'是否令目标玩家洗牌？',description:'可以洗牌或跳过',playerId:'p0',min:0,max:1,allowDecline:true,options:[{id:'shuffle',label:'洗牌'}]};const a={id:'shuffle-choice',kind:'choose',choiceId:c.id,label:'确认选择'};const submit=vi.fn();const m=render(<Table view={{...base,pendingChoice:c,legalActions:[a]}} catalog={catalog} busy={false} onAction={submit}/>);fireEvent.click(m.container.querySelector('[data-choice-option="shuffle"]'));fireEvent.click(screen.getByRole('button',{name:'确认选择'}));expect(submit).toHaveBeenLastCalledWith({...a,selected:['shuffle']});fireEvent.click(screen.getByRole('button',{name:'跳过此选择'}));expect(submit).toHaveBeenLastCalledWith({...a,selected:[]});m.unmount();
});
