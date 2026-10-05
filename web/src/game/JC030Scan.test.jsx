import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent,cleanup} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as k from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import * as accepted31 from '../../../rust-game-wasm/legacy-v0.2.31/hegemony_wasm.js';
import {ReadModal} from './ReadModal';
k.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
accepted31.initSync({module:readFileSync(resolve('../rust-game-wasm/legacy-v0.2.31/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(k.catalog()),old=JSON.parse(accepted31.catalog());
const fixture=JSON.parse(readFileSync(resolve('src/game/jc030Test.fixture.json'),'utf8'));

it('JC030 prints its whole blue bat fields and original and leaves accepted31 definitions unchanged',()=>{
 expect(catalog.engineVersion).toBe('rust-v0.2.32-jc030-blue-bat-candidate');expect(catalog.cardPoolVersion).toBe('limited-v2.29-jc030-blue-bat-candidate');
 expect(catalog.cards).toHaveLength(94);expect(catalog.societies).toHaveLength(7);
 expect(catalog.cards.filter(c=>c.id!=='JC030')).toEqual(old.cards);expect(catalog.societies).toEqual(old.societies);expect(catalog.decks).toEqual(old.decks);
 const card=catalog.cards.find(c=>c.id==='JC030');
 expect(card).toMatchObject({name:'巨型蝙蝠',kind:'character',color:'蓝',unique:false,cost:3,loyalty:['蓝色','蓝色'],magic:'',subtypes:['蝙蝠'],defense:1,permanentIcons:{investigation:0,combat:2,influence:0},temporaryIcons:{investigation:0,combat:0,influence:0},ruleTraits:{cannot_be_equipped:true}});
 expect(card.abilities).toEqual(catalog.cards.find(c=>c.id==='JC029').abilities);
 const hash='e0627c1eec97eab04d8cbafb54c0d25ce496ab89e10e19a81485d84ad2915542';
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));expect(Object.keys(scans)).toHaveLength(101);expect(scans.JC030).toEqual({url:'/cards/JC030.jpg',source:'resource/ymsj-fun.github.io/cards/JC030 巨型蝙蝠.jpg',sha256:hash});
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC030.jpg'))).digest('hex')).toBe(hash);
 for(const id of ['JC032','JZ24','XQ11'])expect(catalog.cards.some(c=>c.id===id)).toBe(false);expect(catalog.societies.some(c=>c.id==='MSJC03')).toBe(false);
});

it('JC030 restores all actual32 whole views and exposes the current subtype with an immutable original face',()=>{
 for(const row of fixture.fixtures)for(let seat=0;seat<4;seat++)expect(JSON.parse(k.view(row.state,seat))).toEqual(row.views[seat]);
 const row=fixture.fixtures.find(f=>f.kind==='positive'),view=JSON.parse(k.view(row.state,0)),card=view.regions[2].characters.find(c=>c.cardId==='JC030');
 expect(card.currentSubtypes).toEqual(['蝙蝠','吸血鬼']);expect(card.icons.investigation).toBe(1);
 const {container}=render(<ReadModal card={card} viewerId="p0" onClose={vi.fn()}/>);expect(container.textContent).toContain('吸血鬼');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'巨型蝙蝠原始牌面'})).toHaveAttribute('src','/cards/JC030.jpg');expect(screen.getByRole('link',{name:'打开巨型蝙蝠原始牌面全图'})).toHaveAttribute('href','/cards/JC030.jpg');
 const lobby=JSON.parse(accepted31.newGame('old31','LOCAL','teams','P0','watchers','1'));expect(()=>k.view(lobby.state,0)).toThrow();expect(JSON.parse(accepted31.view(lobby.state,0))).toEqual(lobby.view);
});

it('JC030 authorizes hidden borrowed originals only for the current controller across four independent views',()=>{
 for(const row of fixture.fixtures.filter(f=>f.kind==='hidden'))for(let seat=0;seat<4;seat++){
  cleanup();const view=JSON.parse(k.view(row.state,seat)),card=view.regions[2].characters[0];expect(card.owner).toBe('p'+row.owner);expect(card.controller).toBe('p'+row.controller);expect(card.currentSubtypes).toBeUndefined();
  render(<ReadModal card={card} viewerId={'p'+seat} onClose={vi.fn()}/>);
  if(seat===row.controller){fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'巨型蝙蝠原始牌面'})).toHaveAttribute('src','/cards/JC030.jpg');}
  else {expect(card.cardId).toBeUndefined();expect(screen.queryByRole('button',{name:'原始牌面'})).not.toBeInTheDocument();expect(screen.queryByRole('img')).not.toBeInTheDocument();expect(screen.queryByText('巨型蝙蝠')).not.toBeInTheDocument();}
 }
});
