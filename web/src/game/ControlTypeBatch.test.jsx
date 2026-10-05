import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import {CardTile,visibleCard} from './CardTile';
import {ReadModal} from './ReadModal';
kernel.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(kernel.catalog());
const definition=catalog.cards.find(c=>c.id==='LC01');
const card={...definition,cardId:'LC01',instanceId:'live-LC01',region:0,owner:'p2',controller:'p0',exhausted:false,faceDown:false,currentSubtypes:['法师','吸血鬼','奴仆']};
it('shows authoritative current types while keeping printed types and original artwork unchanged',()=>{
 const rendered=render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByText('角色 · 法师 · 吸血鬼 · 奴仆')).toBeInTheDocument();
 expect(definition.subtypes).toEqual(['人类','法师']);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));
 expect(screen.getByRole('img',{name:'西比尔原始牌面'})).toHaveAttribute('src','/cards/LC01.jpg');
 rendered.unmount();
});
it('erases stale type overlays and print from every unauthorized concealed seat',()=>{
 const hidden={...card,faceDown:true,kind:'hidden'};
 const rendered=render(<CardTile card={hidden} definition={definition} viewerId="p0"/>);
 expect(screen.getByText('西比尔')).toBeInTheDocument();
 for(const viewer of ['p1','p2','p3']) {
  expect(visibleCard(hidden,viewer).currentSubtypes).toBeUndefined();
  rendered.rerender(<CardTile card={hidden} definition={definition} viewerId={viewer}/>);
  expect(screen.getByRole('button',{name:'查看暗藏者'})).toBeInTheDocument();
  expect(screen.queryByText('角色 · 法师 · 吸血鬼 · 奴仆')).not.toBeInTheDocument();
  expect(screen.queryByText('西比尔')).not.toBeInTheDocument();
 }
 rendered.unmount();
});
it('preserves the character subtitle and its original permanent versus initiative icons',()=>{
 const d=catalog.cards.find(c=>c.id==='JZ27');
 expect(d).toMatchObject({subtitle:'猩红凝视',cost:6,loyalty:['蓝色','蓝色','蓝色'],subtypes:['吸血鬼','法师'],unique:true,defense:3,permanentIcons:{investigation:1,combat:2,influence:0},temporaryIcons:{investigation:0,combat:0,influence:1}});
 const rendered=render(<CardTile card={{...d,cardId:d.id,instanceId:'source-JZ27',owner:'p0',controller:'p0',exhausted:false,faceDown:false}} definition={d} viewerId="p0"/>);
 expect(screen.getByText('猩红凝视')).toBeInTheDocument();rendered.unmount();
});
for(const[id,sha]of[['JC129','0c99776d13e342b85c6d6f6cc476459ec6463e6bbdc146046460518c738a0b59'],['JC036','017507e6a2a229f6541a4690b681ecd71fc09b35962b9673da70f5f035ddba83'],['JZ27','33fbdd4a062962b546cdb0d4f368f35b6c4c019908db73a37cfa09aaa237630e']])it(`reads unchanged original ${id} artwork`,()=>{
 const d=catalog.cards.find(c=>c.id===id);
 expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(sha);
 const rendered=render(<ReadModal card={{...d,cardId:id,instanceId:`original-${id}`,owner:'p0',controller:'p0',exhausted:false,faceDown:false}} definition={d} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:`${d.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);rendered.unmount();
});
