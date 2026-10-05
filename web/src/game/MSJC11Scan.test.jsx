import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import {CardContent,visibleCard} from './CardTile';
import {ReadModal} from './ReadModal';
import {validateDeckDraft} from './deckLibrary';
kernel.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(kernel.catalog());
const definition=catalog.societies.find(c=>c.id==='MSJC11');
const fifty=cards=>({id:'msjc11-reader',name:'执行部双色50',description:'',societyId:'MSJC11',cards,rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''});
it('reads the exact whole original society and finite two standard abilities',()=>{
 expect(definition).toMatchObject({name:'S.P.T.执行部',subtitle:'直属特遣队',kind:'society',subtypes:['企业','部门'],color:'绿',unique:true,startingHand:6,printedCost:null,deckConstraints:[{kind:'greenNeutralOrPrintedHumanCombat'}]});
 expect(definition.abilities.map(a=>a.key)).toEqual(['grant-kill','grant-region-retreat']);expect(catalog.cards).toHaveLength(94);expect(catalog.societies).toHaveLength(7);
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));expect(scans.MSJC11.sha256).toBe('df9964832e48bfef57bcaa0d5f4db67ce2ab9fe0e371ffecc2736c1728b1b853');expect(createHash('sha256').update(readFileSync(resolve('public/cards/MSJC11.jpg'))).digest('hex')).toBe(scans.MSJC11.sha256);
 render(<ReadModal card={{...definition,cardId:definition.id,instanceId:"msjc11-reader",owner:"p0",controller:"p0",faceDown:false,exhausted:false}} definition={definition} viewerId="p0" onClose={vi.fn()}/>);expect(screen.getByRole('dialog')).toHaveTextContent('直属特遣队');expect(screen.getByRole('dialog')).toHaveTextContent('起手 6 张');fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'S.P.T.执行部原始牌面'})).toHaveAttribute('src','/cards/MSJC11.jpg');
});
it('matches printed mixed-deck exception instead of using runtime buffs or magic',()=>{
 for(const cardId of ['JC056','JC058','JC042','JC084','JC086','JC088','JZ59','JZ61','BQ022','LC20'])expect(validateDeckDraft(fifty([{cardId,count:3},{cardId:'JC125',count:47}]),catalog).valid,cardId).toBe(true);
 for(const cardId of ['JZ27','JC104','LC12','JC093','XQ43','JC004'])expect(validateDeckDraft(fifty([{cardId,count:3},{cardId:'JC125',count:47}]),catalog).issues.some(x=>x.code==='society-combat-human'),cardId).toBe(true);
 expect(validateDeckDraft(fifty([{cardId:'JC016',count:4},{cardId:'JC125',count:46}]),catalog).valid).toBe(false);
 expect(validateDeckDraft(fifty([{cardId:'JC125',count:49}]),catalog).valid).toBe(false);
});
it('shows current granted kill and retreat but strips both from every concealed fallback',()=>{
 const card={instanceId:'same-instance',cardId:'JC016',name:'执行部精锐',kind:'character',owner:'p2',controller:'p0',region:1,faceDown:false,exhausted:false,currentKill:2,currentRetreat:true};
 const shown=render(<CardContent card={card} definition={catalog.cards.find(c=>c.id==='JC016')} viewerId="p1"/>);expect(screen.getByText('本回合杀伤 2')).toBeInTheDocument();expect(screen.getByText('本回合撤回')).toBeInTheDocument();shown.unmount();
 for(const viewerId of ['p0','p1','p2','p3']){const hidden=visibleCard({...card,faceDown:true},viewerId);expect(hidden.currentKill).toBeUndefined();expect(hidden.currentRetreat).toBeUndefined();const view=render(<CardContent card={{...card,faceDown:true}} definition={catalog.cards.find(c=>c.id==='JC016')} viewerId={viewerId}/>);expect(screen.queryByText('本回合杀伤 2')).toBeNull();expect(screen.queryByText('本回合撤回')).toBeNull();view.unmount();}
});
