import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent,cleanup} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import { kernel as k } from './testKernel';
import {ReadModal} from './ReadModal';
const catalog=JSON.parse(k.catalog());
const fixture=JSON.parse(readFileSync(resolve('src/game/jc030Test.fixture.json'),'utf8'));

it('checks JC030 printed blue bat fields and pinned original',()=>{

 
 const card=catalog.cards.find(c=>c.id==='JC030');
 expect(card).toMatchObject({name:'巨型蝙蝠',kind:'character',color:'蓝',unique:false,cost:3,loyalty:['蓝色','蓝色'],magic:'',subtypes:['蝙蝠'],defense:1,permanentIcons:{investigation:0,combat:2,influence:0},temporaryIcons:{investigation:0,combat:0,influence:0},ruleTraits:{cannot_be_equipped:true}});
 expect(card.abilities).toEqual(catalog.cards.find(c=>c.id==='JC029').abilities);
 const hash='e0627c1eec97eab04d8cbafb54c0d25ce496ab89e10e19a81485d84ad2915542';
 const scans=JSON.parse(readFileSync(resolve('src/game/cardScansV035.fixture.json'),'utf8'));expect(Object.keys(scans)).toHaveLength(107);expect(scans.JC030).toEqual({url:'/cards/JC030.jpg',source:'resource/ymsj-fun.github.io/cards/JC030 巨型蝙蝠.jpg',sha256:hash});
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC030.jpg'))).digest('hex')).toBe(hash);
 for(const id of ['XQ11'])expect(catalog.cards.some(c=>c.id===id)).toBe(false);expect(catalog.societies.some(c=>c.id==='MSJC03')).toBe(true);
});

it('renders the recorded JC030 effective subtype alongside its immutable original face',()=>{
 const row=fixture.fixtures.find(f=>f.kind==='positive'),view=row.views[0],card=view.regions[2].characters.find(c=>c.cardId==='JC030');
 expect(card.currentSubtypes).toEqual(['蝙蝠','吸血鬼']);expect(card.icons.investigation).toBe(1);
 const {container}=render(<ReadModal card={card} viewerId="p0" onClose={vi.fn()}/>);expect(container.textContent).toContain('吸血鬼');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'巨型蝙蝠原始牌面'})).toHaveAttribute('src','/cards/JC030.jpg');expect(screen.getByRole('link',{name:'打开巨型蝙蝠原始牌面全图'})).toHaveAttribute('href','/cards/JC030.jpg');
});

it('JC030 authorizes hidden borrowed originals only for the current controller across four independent views',()=>{
 for(const row of fixture.fixtures.filter(f=>f.kind==='hidden'))for(let seat=0;seat<4;seat++){
  cleanup();const view=row.views[seat],card=view.regions[2].characters[0];expect(card.owner).toBe('p'+row.owner);expect(card.controller).toBe('p'+row.controller);expect(card.currentSubtypes).toBeUndefined();
  render(<ReadModal card={card} viewerId={'p'+seat} onClose={vi.fn()}/>);
  if(seat===row.controller){fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'巨型蝙蝠原始牌面'})).toHaveAttribute('src','/cards/JC030.jpg');}
  else {expect(card.cardId).toBeUndefined();expect(screen.queryByRole('button',{name:'原始牌面'})).not.toBeInTheDocument();expect(screen.queryByRole('img')).not.toBeInTheDocument();expect(screen.queryByText('巨型蝙蝠')).not.toBeInTheDocument();}
 }
});
