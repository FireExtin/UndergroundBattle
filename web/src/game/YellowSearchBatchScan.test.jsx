import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import {ReadModal} from './ReadModal';
kernel.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(kernel.catalog());
const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));
it('admits only the two approved original yellow unique cards with their printed numbers and loyalty',()=>{
 expect(catalog.cards).toHaveLength(79);expect(Object.keys(scans)).toHaveLength(81);
 expect(catalog.cards.filter(c=>c.color==='黄'&&c.unique).map(c=>c.id)).toEqual(['WM003','LC01']);
 expect(catalog.cards.find(c=>c.id==='WM003')).toMatchObject({name:'千机庙离',color:'黄',unique:true,cost:1,defense:1,magic:'心灵',loyalty:['黄色','星辰','星辰'],subtypes:['人类','法师','学生'],permanentIcons:{investigation:0,combat:0,influence:0},temporaryIcons:{investigation:1,combat:0,influence:1},abilities:[{key:'search-any-private',timing:'fast'}]});
 expect(catalog.cards.find(c=>c.id==='LC01')).toMatchObject({name:'西比尔',color:'黄',unique:true,cost:5,defense:4,magic:'神圣',loyalty:['黄色','黄色'],subtypes:['人类','法师'],permanentIcons:{investigation:2,combat:0,influence:0},temporaryIcons:{investigation:1,combat:0,influence:0},abilities:[]});
 expect(catalog.privateDeckTop).toBeUndefined();
});
for(const[id,hash]of[['WM003','2409ef206e3af93d1eece3ec0bf4ba00a9e3d0c12a8e70443877cc8a28cc9bff'],['LC01','a0defb07b43463cc882e15b993c73eda22ea77ca39062da71e6518a365cc5bd6']])it(`reads the actual unmodified ${id} artwork through the existing reader`,()=>{
 const definition=catalog.cards.find(c=>c.id===id);expect(scans[id].sha256).toBe(hash);expect(createHash('sha256').update(readFileSync(resolve(`public/cards/${id}.jpg`))).digest('hex')).toBe(hash);
 const rendered=render(<ReadModal card={{...definition,cardId:id,instanceId:`private-${id}`,owner:'p0',controller:'p0',exhausted:false,faceDown:false}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:`${definition.name}原始牌面`})).toHaveAttribute('src',`/cards/${id}.jpg`);rendered.unmount();
});
