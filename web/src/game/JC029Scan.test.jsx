import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import { kernel as k } from './testKernel';
import {ReadModal} from './ReadModal';
import {validateDeckDraft} from './deckLibrary';
const catalog=JSON.parse(k.catalog()),d=catalog.cards.find(c=>c.id==='JC029');
const draft=(count=3,societyId=null)=>({id:'jc029-unit',name:'新生血族50',description:'',societyId,cards:[{cardId:'JC029',count},{cardId:'JC125',count:50-count}],rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''});
it('keeps the whole original and exactly one existing reveal ability alongside the finite green admission',()=>{
 expect(d).toMatchObject({name:'新生血族',kind:'character',subtypes:['吸血鬼'],cost:2,loyalty:['蓝色','蓝色'],color:'蓝',magic:'鲜血',defense:1,unique:false,deckCopyLimit:3,keywords:['袭击1'],permanentIcons:{investigation:0,combat:0,influence:1},temporaryIcons:{investigation:0,combat:1,influence:0}});
 expect(d.abilities).toEqual([{key:'raid-1',label:'袭击1',timing:'fast',costs:[],triggered:true}]);
 for(const id of ['LC30','JC018','JC015'])expect(catalog.cards.some(c=>c.id===id)).toBe(true);expect(catalog.cards.some(c=>c.id==='XQ11')).toBe(false);
 const scan=JSON.parse(readFileSync(resolve('src/game/cardScansV034.fixture.json'),'utf8')).JC029;
 expect(scan.sha256).toBe('b3da8f1546e2f8bbbaae8d3c4d4c79dd712cfa7f8d133fe1723efaa1fa152785');
 expect(createHash('sha256').update(readFileSync(resolve('public/cards/JC029.jpg'))).digest('hex')).toBe(scan.sha256);
 expect(validateDeckDraft(draft(),catalog).valid).toBe(true);expect(validateDeckDraft(draft(4),catalog).valid).toBe(false);
 expect(validateDeckDraft(draft(3,'MSJC03'),catalog).valid).toBe(false);
});
it('uses an actual WASM own-hand projection and opens the original card without replacing the damage rule',()=>{
 let room=JSON.parse(k.newGameWithDeck('jc029-reader','LOCAL','duel','P0',JSON.stringify(draft()),'1'));
 room=JSON.parse(k.joinGameWithDeck(room.state,'P1',JSON.stringify(draft())));
 for(const [seat,kind] of [[0,'ready'],[1,'ready'],[0,'start']])room=JSON.parse(k.applyRoom(room.state,seat,JSON.stringify({commandId:'reader-'+seat+'-'+kind,expectedVersion:room.version,action:{kind:'game',action:{kind}}}),'1000'));
 const card=JSON.parse(k.view(room.state,0)).hand.find(c=>c.cardId==='JC029');expect(card).toMatchObject({owner:'p0',controller:'p0',cost:2});
 render(<ReadModal card={card} definition={d} viewerId="p0" onClose={vi.fn()}/>);
 expect(screen.getByRole('dialog')).toHaveTextContent('袭击1');expect(screen.getByRole('dialog')).toHaveTextContent('1点伤害');
 fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'新生血族原始牌面'})).toHaveAttribute('src','/cards/JC029.jpg');
});
