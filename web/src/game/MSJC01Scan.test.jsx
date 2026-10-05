import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import * as kernel from '../../../rust-game-wasm/pkg/hegemony_wasm.js';
import {actionForRoom,actionPayload} from './api';
import {CardContent} from './CardTile';
import {ReadModal} from './ReadModal';
import {validateDeckDraft} from './deckLibrary';
kernel.initSync({module:readFileSync(resolve('../rust-game-wasm/pkg/hegemony_wasm_bg.wasm'))});
const catalog=JSON.parse(kernel.catalog());
const definition=catalog.societies.find(c=>c.id==='MSJC01');
const yellow=['JC001','JC002','JC003','JC004','JC005','JC006','JC007','XQ03','JC008'];
function draft(n=25,total=50){let left=n;const cards=yellow.flatMap(cardId=>{const count=Math.min(3,left);left-=count;return count?[{cardId,count}]:[];});if(total>n)cards.push({cardId:'JC125',count:total-n});return{id:'msjc01-reader',name:'MSJC01本地正常牌组',description:'',societyId:'MSJC01',cards,rulesVersion:catalog.rulesVersion,cardPoolVersion:catalog.cardPoolVersion,engineVersion:catalog.engineVersion,updatedAt:''};}
function localRoom(){
 let room=JSON.parse(kernel.newGameWithDeck('msjc01-reader','LOCAL','duel','P0',JSON.stringify(draft()),'12'));
 room=JSON.parse(kernel.joinGameWithDeck(room.state,'P1',JSON.stringify(draft())));let count=0;
 const view=seat=>JSON.parse(kernel.view(room.state,seat));
 const session=(seat,action)=>{const r=JSON.parse(kernel.applyRoom(room.state,seat,JSON.stringify({commandId:`msjc01-reader-${count++}`,expectedVersion:room.version,action}),String(1000+count*10)));expect(r.outcome,r.errorMessage).toBe('accepted');room=r;};
 const game=(seat,action)=>{let v=view(seat);if(v.responseWindow&&!v.pendingChoice&&!v.waitingChoice&&action.kind!=='pass'){const member=v.responseWindow.members.find(m=>m.playerId===v.you);if(member?.status==='undecided'){session(seat,{kind:'beginResponse',windowId:v.responseWindow.id,intentId:`intent-${count}`});v=view(seat);}}session(seat,actionForRoom(v,actionPayload(action)));};
 const advance=done=>{for(let n=0;n<600;n++){const views=[view(0),view(1)];if(done(views))return;const chooser=views.findIndex(v=>v.pendingChoice);if(chooser>=0){const p=views[chooser].pendingChoice;game(chooser,{kind:'choose',choiceId:p.id,selected:p.allowDecline?[]:p.options.slice(0,p.min??1).map(o=>o.id)});}else{const passer=views.findIndex(v=>v.legalActions.some(a=>a.kind==='pass'));expect(passer).toBeGreaterThanOrEqual(0);game(passer,{kind:'pass'});}}throw Error('bounded normal Room progress failed');};
 for(const[seat,kind]of[[0,'ready'],[1,'ready'],[0,'start']])game(seat,{kind});return{view,game,advance};
}
it('retains actual MSJC01 original, metadata, two abilities and two real admitted yellow unique cards',()=>{
 expect(catalog.engineVersion).toBe('rust-v0.2.27-white-wound-society-candidate');expect(catalog.cardPoolVersion).toBe('limited-v2.24-white-wound-society-candidate');expect(catalog.cards).toHaveLength(88);expect(catalog.societies.map(c=>c.id)).toEqual(['MSJC09','MSJC01','MSJC07','MSJC06']);
 expect(definition).toMatchObject({name:'帷幕守望',subtitle:'世界守护者',kind:'society',type:'秘社/法师结社',subtypes:['法师结社'],color:'黄',unique:true,startingHand:6,printedCost:null,deckConstraints:[{kind:'minimumColor',color:'黄',count:25}]});expect(definition.abilities.map(a=>a.key)).toEqual(['drawWithInitiative','search-yellow-unique']);expect(catalog.cards.filter(c=>c.color==='黄'&&c.unique)).toHaveLength(2);
 const scans=JSON.parse(readFileSync(resolve('public/card-scans.json'),'utf8'));const hash='c61c887558cb184a532758eac5db766234c32a3a3605162a38c0899993e7bdf2';expect(Object.keys(scans)).toHaveLength(92);expect(scans.MSJC01).toEqual({url:'/cards/MSJC01.jpg',source:'resource/ymsj-fun.github.io/cards/MSJC01 帷幕守望.jpg',sha256:hash});expect(createHash('sha256').update(readFileSync(resolve('public/cards/MSJC01.jpg'))).digest('hex')).toBe(hash);
});
it('agrees with actual WASM on 24/25/27 yellow and excludes the society from 50 cards',()=>{
 for(const n of[24,25,27]){const d=draft(n);expect(validateDeckDraft(d,catalog).valid).toBe(n>=25);const create=()=>kernel.newGameWithDeck('local','LOCAL','duel','P0',JSON.stringify(d),'1');if(n>=25)expect(create).not.toThrow();else expect(create).toThrow();}const small=draft(25,49);expect(validateDeckDraft(small,catalog).valid).toBe(false);expect(()=>kernel.newGameWithDeck('local','LOCAL','duel','P0',JSON.stringify(small),'1')).toThrow();
});
it('uses normal paid empty search and shows the server usage fact after a normal next-turn reset',()=>{
 const local=localRoom();
 for(let n=0;n<4;n++){local.advance(vs=>vs[0].legalActions.some(a=>a.kind==='asset'));const v=local.view(0);const a=v.legalActions.find(a=>a.kind==='asset'&&v.hand.find(c=>c.instanceId===a.cardId)?.cardId==='JC125')||v.legalActions.find(a=>a.kind==='asset');local.game(0,a);}
 local.advance(vs=>vs[0].legalActions.some(a=>a.abilityId==='search-yellow-unique'));let v=local.view(0);const a=v.legalActions.find(a=>a.abilityId==='search-yellow-unique');const handBefore=v.hand.length;const deckBefore=v.players[0].deckCount;const turn=v.turn;local.game(0,a);
 let card=local.view(0).societyZones[0].card;expect(card.usedOncePerGame).toEqual(['search-yellow-unique']);expect(card.exhausted).toBe(true);local.advance(vs=>vs[0].stack.length===0);v=local.view(0);expect(v.hand).toHaveLength(handBefore);expect(v.players[0].deckCount).toBe(deckBefore);expect(v.log.some(e=>e.text.includes('没有符合条件的牌'))).toBe(true);
 local.advance(vs=>vs[0].turn>turn&&vs[0].legalActions.some(a=>a.abilityId==='drawWithInitiative'));v=local.view(0);card=v.societyZones[0].card;expect(card.exhausted).toBe(false);expect(card.usedOncePerGame).toEqual(['search-yellow-unique']);expect(v.legalActions.some(a=>a.abilityId==='search-yellow-unique')).toBe(false);expect(v.legalActions.some(a=>a.abilityId==='drawWithInitiative')).toBe(true);expect(local.view(1).societyZones[0].card.usedOncePerGame).toEqual(card.usedOncePerGame);
 let rendered=render(<CardContent card={card} definition={definition} viewerId="p0"/>);expect(screen.getByText('黄色独有检索（每局一次） · 本局已使用')).toBeInTheDocument();rendered.unmount();
 rendered=render(<ReadModal card={card} definition={definition} viewerId="p0" onClose={vi.fn()}/>);expect(screen.getByRole('dialog')).toHaveTextContent('本局已使用');expect(screen.getByRole('dialog')).not.toHaveTextContent('防御');fireEvent.click(screen.getByRole('button',{name:'原始牌面'}));expect(screen.getByRole('img',{name:'帷幕守望原始牌面'})).toHaveAttribute('src','/cards/MSJC01.jpg');rendered.unmount();
});
it('omits game-use labels for older society projections with no new field',()=>{
 const local=localRoom();const card=local.view(0).societyZones[0].card;expect(card.usedOncePerGame).toBeUndefined();const rendered=render(<CardContent card={card} definition={definition} viewerId="p0"/>);expect(screen.queryByText(/本局已使用/)).not.toBeInTheDocument();rendered.unmount();
});
